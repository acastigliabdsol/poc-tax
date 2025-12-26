use anyhow::{Context, Result};
use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
use dotenvy::dotenv;
use futures_util::{AsyncReadExt, FutureExt};
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};


use tax_manager::domain::calculators::IVACalculator;
use tax_manager::infra::db::ProfileRepository;
use tax_manager::infra::cache::ProfileCache;
use tax_manager::app::resolver::ProfileResolver;
use tax_manager::app::orchestrator::Orchestrator;
use tax_manager::app::rpc::TaxEngineImpl;
use tax_manager::schema_capnp::tax_engine;


// Also need Clone for ProfileResolver and trait objects if possible, 
// or I'll just use Arc for them inside the implementation.
// Let's check how they are defined.
// Actually, let's wrap the whole Orchestrator in an Arc if it's easier.

// Wait, Orchestrator has private fields. I might need to make them public or provide a clone.
// Let's check src/app/orchestrator.rs

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "impuestos=info,debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Tax Engine RPC Server...");

    let addr_str = env::var("RPC_ADDR").unwrap_or_else(|_| "0.0.0.0:50051".to_string());
    let addr = addr_str.to_socket_addrs()?.next().context("Invalid address")?;

    // Infrastructure setup
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let redis_url = env::var("REDIS_URL").context("REDIS_URL must be set")?;

    info!("Connecting to PostgreSQL...");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .context("Failed to connect to PostgreSQL")?;

    info!("Connecting to Redis...");
    let redis_client = Arc::new(redis::Client::open(redis_url)?);
    
    // Wire layers
    let db_repo = ProfileRepository::new(db_pool);
    let cache_repo = ProfileCache::new(redis_client);
    let profile_resolver = ProfileResolver::new(db_repo, cache_repo);
    let iva_calculator = IVACalculator;
    
    let orchestrator = Orchestrator::new(profile_resolver, iva_calculator);

    let tax_engine_impl = TaxEngineImpl { orchestrator };
    let client: tax_engine::Client = capnp_rpc::new_client::<tax_engine::Client, _>(tax_engine_impl);

    let listener = TcpListener::bind(&addr).await?;
    info!("RPC Server listening on {}", addr);

    let local = tokio::task::LocalSet::new();
    local.run_until(async move {
        loop {
            let (stream, _) = listener.accept().await?;
            stream.set_nodelay(true)?;
            let (reader, writer) = tokio_util::compat::TokioAsyncReadCompatExt::compat(stream).split();
            let network = twoparty::VatNetwork::new(
                reader,
                writer,
                rpc_twoparty_capnp::Side::Server,
                Default::default(),
            );

            let rpc_system = RpcSystem::new(Box::new(network), Some(client.clone().client));
            tokio::task::spawn_local(rpc_system.map(|_| ()));
        }
    }).await
}
