use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_util::compat::TokioAsyncReadCompatExt;
use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
use futures_util::{FutureExt, AsyncReadExt};

use tax_manager::app::orchestrator::Orchestrator;
use tax_manager::app::resolver::ProfileResolver;
use tax_manager::app::rpc::TaxEngineImpl;
use tax_manager::domain::calculators::IVACalculator;
use tax_manager::infra::cache::ProfileCache;
use tax_manager::infra::db::ProfileRepository;
use tax_manager::schema_capnp::tax_engine;

use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::redis::Redis;
use testcontainers::runners::AsyncRunner;

#[tokio::test]
async fn test_e2e_rpc_calculation() {
    let local = tokio::task::LocalSet::new();

    local.run_until(async move {
        // 1. Setup Postgres
        let pg_node = Postgres::default().start().await.expect("Failed to start Postgres");
        let pg_host = pg_node.get_host().await.expect("Failed to get PG host");
        let pg_port = pg_node.get_host_port_ipv4(5432).await.expect("Failed to get PG port");
        let db_url = format!("postgres://postgres:postgres@{}:{}/postgres", pg_host, pg_port);
        let db_pool = PgPool::connect(&db_url).await.expect("Failed to connect to test PG");

        sqlx::query("CREATE TABLE profiles (client_id TEXT PRIMARY KEY, fiscal_category TEXT NOT NULL, config JSONB NOT NULL DEFAULT '{}')")
            .execute(&db_pool).await.unwrap();
        sqlx::query("CREATE TABLE iva_rates (jurisdiction TEXT PRIMARY KEY, rate FLOAT8 NOT NULL)")
            .execute(&db_pool).await.unwrap();
        sqlx::query("INSERT INTO iva_rates (jurisdiction, rate) VALUES ('TEST_J', 0.15)")
            .execute(&db_pool).await.unwrap();
        sqlx::query("INSERT INTO profiles (client_id, fiscal_category, config) VALUES ('client_test', 'RESPONSABLE_INSCRIPTO', '{}')")
            .execute(&db_pool).await.unwrap();

        // 2. Setup Redis
        let redis_node = Redis::default().start().await.expect("Failed to start Redis");
        let redis_host = redis_node.get_host().await.expect("Failed to get Redis host");
        let redis_port = redis_node.get_host_port_ipv4(6379).await.expect("Failed to get Redis port");
        let redis_url = format!("redis://{}:{}", redis_host, redis_port);
        let redis_client = Arc::new(redis::Client::open(redis_url).expect("Failed to create redis client"));

        // 3. Initialize Server
        let db_repo = ProfileRepository::new(db_pool);
        let cache_repo = ProfileCache::new(redis_client);
        let resolver = ProfileResolver::new(db_repo, cache_repo);
        let orchestrator = Orchestrator::new(resolver, IVACalculator);
        let tax_engine_impl = TaxEngineImpl::new(orchestrator);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Start server in background
        tokio::task::spawn_local(async move {
            let (stream, _) = listener.accept().await.unwrap();
            stream.set_nodelay(true).unwrap();
            let (reader, writer) = stream.compat().split();
            let network = twoparty::VatNetwork::new(
                reader,
                writer,
                rpc_twoparty_capnp::Side::Server,
                Default::default(),
            );
            
            let client: tax_engine::Client = capnp_rpc::new_client::<tax_engine::Client, _>(tax_engine_impl);

            let rpc_system = RpcSystem::new(Box::new(network), Some(client.client));
            rpc_system.await.map(|_| ()).unwrap();
        });

        // 4. Client Request
        let stream = tokio::net::TcpStream::connect(&addr).await.unwrap();
        stream.set_nodelay(true).unwrap();
        let (reader, writer) = stream.compat().split();
        let network = twoparty::VatNetwork::new(
            reader,
            writer,
            rpc_twoparty_capnp::Side::Client,
            Default::default(),
        );

        let mut rpc_system = RpcSystem::new(Box::new(network), None);
        let client: tax_engine::Client = rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);
        tokio::task::spawn_local(rpc_system.map(|_| ()));

        let mut request = client.calculate_request();
        {
            let mut tx_req = request.get().init_tx();
            tx_req.set_client_id("client_test");
            tx_req.set_amount(1000.0);
            tx_req.set_jurisdiction("TEST_J");
            tx_req.set_product("TEST_PROD");
        }

        let response = request.send().promise.await.expect("RPC failed");
        let results = response.get().expect("Failed to get results");
        let resp_data = results.get_response().expect("Failed to get response");

        assert_eq!(resp_data.get_total_amount(), 150.0);
        let breakdown = resp_data.get_breakdown().expect("Failed to get breakdown");
        assert_eq!(breakdown.len(), 1);
        let detail = breakdown.get(0);
        assert_eq!(detail.get_tax_type().unwrap().to_str().unwrap(), "IVA");
        assert_eq!(detail.get_rate(), 0.15);
        assert_eq!(detail.get_amount(), 150.0);
    }).await;
}
