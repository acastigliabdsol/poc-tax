use sqlx::PgPool;
use std::sync::Arc;
use tax_manager::app::orchestrator::Orchestrator;
use tax_manager::app::resolver::ProfileResolver;
use tax_manager::domain::calculators::IVACalculator;
use tax_manager::domain::models::Transaction;
use tax_manager::infra::cache::ProfileCache;
use tax_manager::infra::db::ProfileRepository;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::redis::Redis;
use testcontainers::runners::AsyncRunner;

#[tokio::test]
async fn test_integration_full_flow() {
    // 1. Setup Postgres container
    let pg_node = Postgres::default().start().await.expect("Failed to start Postgres");
    let pg_host = pg_node.get_host().await.expect("Failed to get PG host");
    let pg_port = pg_node.get_host_port_ipv4(5432).await.expect("Failed to get PG port");
    let db_url = format!("postgres://postgres:postgres@{}:{}/postgres", pg_host, pg_port);
    
    let db_pool = PgPool::connect(&db_url).await.expect("Failed to connect to test PG");

    // Run migrations manually for the test
    sqlx::query("CREATE TABLE profiles (client_id TEXT PRIMARY KEY, fiscal_category TEXT NOT NULL, config JSONB NOT NULL DEFAULT '{}')")
        .execute(&db_pool).await.unwrap();
    sqlx::query("CREATE TABLE iva_rates (jurisdiction TEXT PRIMARY KEY, rate FLOAT8 NOT NULL)")
        .execute(&db_pool).await.unwrap();
    sqlx::query("INSERT INTO iva_rates (jurisdiction, rate) VALUES ('TEST_J', 0.15), ('DEFAULT', 0.21)")
        .execute(&db_pool).await.unwrap();
    sqlx::query("INSERT INTO profiles (client_id, fiscal_category, config) VALUES ('client_test', 'RESPONSABLE_INSCRIPTO', '{}')")
        .execute(&db_pool).await.unwrap();

    // 2. Setup Redis container
    let redis_node = Redis::default().start().await.expect("Failed to start Redis");
    let redis_host = redis_node.get_host().await.expect("Failed to get Redis host");
    let redis_port = redis_node.get_host_port_ipv4(6379).await.expect("Failed to get Redis port");
    let redis_url = format!("redis://{}:{}", redis_host, redis_port);
    
    let redis_client = Arc::new(redis::Client::open(redis_url).expect("Failed to create redis client"));

    // 3. Initialize layers
    let db_repo = ProfileRepository::new(db_pool.clone());
    let cache_repo = ProfileCache::new(redis_client);
    let resolver = ProfileResolver::new(db_repo, cache_repo);
    let orchestrator = Orchestrator::new(resolver, IVACalculator);

    // 4. Run calculation
    let tx = Transaction {
        amount: 1000.0,
        product: "TEST".to_string(),
        jurisdiction: "TEST_J".to_string(),
        client_id: "client_test".to_string(),
        date: chrono::Local::now().date_naive(),
    };

    // First call (Populate cache)
    let res1 = orchestrator.process_calculation(tx.clone()).await.expect("Calculation failed");
    assert_eq!(res1.len(), 1);
    assert_eq!(res1[0].rate, 0.15);
    assert_eq!(res1[0].amount, 150.0);

    // Change DB value to verify cache hit
    sqlx::query("UPDATE iva_rates SET rate = 0.50 WHERE jurisdiction = 'TEST_J'")
        .execute(&db_pool).await.unwrap();

    // Second call (Should hit cache and still be 0.15)
    let res2 = orchestrator.process_calculation(tx).await.expect("Calculation failed");
    assert_eq!(res2[0].rate, 0.15);
    assert_eq!(res2[0].amount, 150.0);
}
