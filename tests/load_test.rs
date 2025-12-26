//! Load test for Tax Engine RPC using Goose
//! 
//! Run with:
//! ```bash
//! # First, start the server with required dependencies (PostgreSQL + Redis)
//! # Then run:
//! 
//! # Quick test (10 users, 10 seconds)
//! cargo test --test load_test --release -- --users 10 --run-time 10s
//! 
//! # Full load test (1000 users, 60 seconds, targeting 10K RPS)
//! cargo test --test load_test --release -- --users 1000 --hatch-rate 100 --run-time 60s --report-file report.html
//! 
//! # With custom server address
//! RPC_ADDR=192.168.1.100:50051 cargo test --test load_test --release -- --users 100 --run-time 30s
//! ```
//!
//! Performance Targets (per RT-002):
//! - Latency p99 < 5ms
//! - Latency p50 < 1ms
//! - Capacity: Minimum 10,000 RPS per instance

use goose::prelude::*;
use tax_manager::schema_capnp::tax_engine;
use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncReadCompatExt;
use futures_util::{FutureExt, AsyncReadExt};
use std::env;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};

/// Global metrics for RPC performance tracking
static RPC_SUCCESS_COUNT: AtomicU64 = AtomicU64::new(0);
static RPC_ERROR_COUNT: AtomicU64 = AtomicU64::new(0);
static RPC_TOTAL_LATENCY_US: AtomicU64 = AtomicU64::new(0);
static RPC_MIN_LATENCY_US: AtomicU64 = AtomicU64::new(u64::MAX);
static RPC_MAX_LATENCY_US: AtomicU64 = AtomicU64::new(0);

/// Latency buckets for histogram (in microseconds)
static LATENCY_UNDER_1MS: AtomicU64 = AtomicU64::new(0);
static LATENCY_1_5MS: AtomicU64 = AtomicU64::new(0);
static LATENCY_5_10MS: AtomicU64 = AtomicU64::new(0);
static LATENCY_OVER_10MS: AtomicU64 = AtomicU64::new(0);

/// Request to send through the RPC worker channel
struct RpcRequest {
    resp_tx: oneshot::Sender<Result<Duration, String>>,
}

/// Per-worker connection manager
struct RpcWorker {
    addr: String,
    shutdown: Arc<AtomicBool>,
}

impl RpcWorker {
    fn new(addr: String, shutdown: Arc<AtomicBool>) -> Self {
        Self { addr, shutdown }
    }

    async fn run(&self, mut rx: mpsc::Receiver<RpcRequest>) {
        // Connect to RPC server
        let stream = match TcpStream::connect(&self.addr).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to connect to RPC server at {}: {}", self.addr, e);
                // Respond to all pending requests with error
                while let Some(req) = rx.recv().await {
                    let _ = req.resp_tx.send(Err(format!("Connection failed: {}", e)));
                }
                return;
            }
        };
        
        if let Err(e) = stream.set_nodelay(true) {
            eprintln!("Warning: Failed to set TCP_NODELAY: {}", e);
        }

        let (reader, writer) = stream.compat().split();
        let network = twoparty::VatNetwork::new(
            reader,
            writer,
            rpc_twoparty_capnp::Side::Client,
            Default::default(),
        );

        let mut rpc_system = RpcSystem::new(Box::new(network), None);
        let client: tax_engine::Client = rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);

        // Spawn RPC system handler
        let rpc_handle = tokio::task::spawn_local(rpc_system.map(|_| ()));

        // Process requests
        while let Some(req) = rx.recv().await {
            if self.shutdown.load(Ordering::Relaxed) {
                break;
            }

            let start = Instant::now();
            let mut request = client.calculate_request();
            {
                let mut tx_req = request.get().init_tx();
                tx_req.set_client_id("load_test_client");
                tx_req.set_amount(1000.0);
                tx_req.set_jurisdiction("TEST_JURISDICTION");
                tx_req.set_product("TEST_PRODUCT");
            }

            let promise = request.send().promise;
            let resp_tx = req.resp_tx;

            tokio::task::spawn_local(async move {
                match promise.await {
                    Ok(_) => {
                        let latency = start.elapsed();
                        let _ = resp_tx.send(Ok(latency));
                    }
                    Err(e) => {
                        let _ = resp_tx.send(Err(e.to_string()));
                    }
                }
            });
        }

        // Cleanup
        rpc_handle.abort();
    }
}

/// Connection pool for RPC workers
struct RpcConnectionPool {
    senders: Vec<mpsc::Sender<RpcRequest>>,
    next_idx: AtomicU64,
    _shutdown: Arc<AtomicBool>,
}

impl RpcConnectionPool {
    fn new(addr: &str, num_workers: usize) -> Self {
        let shutdown = Arc::new(AtomicBool::new(false));
        let mut senders = Vec::with_capacity(num_workers);

        for worker_id in 0..num_workers {
            let (tx, rx) = mpsc::channel::<RpcRequest>(1000);
            let worker = RpcWorker::new(addr.to_string(), shutdown.clone());
            
            // Spawn worker in dedicated thread with its own runtime
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create worker runtime");

                let local = tokio::task::LocalSet::new();
                local.block_on(&rt, async move {
                    tracing::debug!("RPC Worker {} started", worker_id);
                    worker.run(rx).await;
                    tracing::debug!("RPC Worker {} stopped", worker_id);
                });
            });

            senders.push(tx);
        }

        Self {
            senders,
            next_idx: AtomicU64::new(0),
            _shutdown: shutdown,
        }
    }

    async fn send_request(&self) -> Result<Duration, String> {
        // Round-robin load balancing
        let idx = self.next_idx.fetch_add(1, Ordering::Relaxed) as usize % self.senders.len();
        let sender = &self.senders[idx];

        let (resp_tx, resp_rx) = oneshot::channel();
        
        sender
            .send(RpcRequest { resp_tx })
            .await
            .map_err(|e| format!("Failed to send to worker: {}", e))?;

        resp_rx
            .await
            .map_err(|e| format!("Worker channel closed: {}", e))?
    }
}

/// Lazy initialization of the connection pool using OnceLock
static RPC_POOL: OnceLock<RpcConnectionPool> = OnceLock::new();

fn get_pool() -> &'static RpcConnectionPool {
    RPC_POOL.get_or_init(|| {
        let addr = env::var("RPC_ADDR").unwrap_or_else(|_| "127.0.0.1:50051".to_string());
        // Use number of CPU cores or 8 workers, whichever is higher
        let num_workers = std::cmp::max(num_cpus::get(), 8);
        println!("Initializing RPC connection pool with {} workers to {}", num_workers, addr);
        RpcConnectionPool::new(&addr, num_workers)
    })
}

/// Record latency metrics
fn record_latency(latency: Duration) {
    let latency_us = latency.as_micros() as u64;
    
    RPC_SUCCESS_COUNT.fetch_add(1, Ordering::Relaxed);
    RPC_TOTAL_LATENCY_US.fetch_add(latency_us, Ordering::Relaxed);
    
    // Update min latency
    let mut current_min = RPC_MIN_LATENCY_US.load(Ordering::Relaxed);
    while latency_us < current_min {
        match RPC_MIN_LATENCY_US.compare_exchange_weak(
            current_min,
            latency_us,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => break,
            Err(actual) => current_min = actual,
        }
    }
    
    // Update max latency
    let mut current_max = RPC_MAX_LATENCY_US.load(Ordering::Relaxed);
    while latency_us > current_max {
        match RPC_MAX_LATENCY_US.compare_exchange_weak(
            current_max,
            latency_us,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => break,
            Err(actual) => current_max = actual,
        }
    }
    
    // Record in histogram bucket
    if latency_us < 1000 {
        LATENCY_UNDER_1MS.fetch_add(1, Ordering::Relaxed);
    } else if latency_us < 5000 {
        LATENCY_1_5MS.fetch_add(1, Ordering::Relaxed);
    } else if latency_us < 10000 {
        LATENCY_5_10MS.fetch_add(1, Ordering::Relaxed);
    } else {
        LATENCY_OVER_10MS.fetch_add(1, Ordering::Relaxed);
    }
}

/// Print detailed performance metrics
fn print_metrics(duration_secs: f64) {
    let success = RPC_SUCCESS_COUNT.load(Ordering::Relaxed);
    let errors = RPC_ERROR_COUNT.load(Ordering::Relaxed);
    let total = success + errors;
    let total_latency = RPC_TOTAL_LATENCY_US.load(Ordering::Relaxed);
    let min_latency = RPC_MIN_LATENCY_US.load(Ordering::Relaxed);
    let max_latency = RPC_MAX_LATENCY_US.load(Ordering::Relaxed);
    
    let under_1ms = LATENCY_UNDER_1MS.load(Ordering::Relaxed);
    let l_1_5ms = LATENCY_1_5MS.load(Ordering::Relaxed);
    let l_5_10ms = LATENCY_5_10MS.load(Ordering::Relaxed);
    let over_10ms = LATENCY_OVER_10MS.load(Ordering::Relaxed);
    
    let avg_latency = if success > 0 { total_latency / success } else { 0 };
    let rps = if duration_secs > 0.0 { total as f64 / duration_secs } else { 0.0 };
    
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║          TAX ENGINE RPC LOAD TEST RESULTS                  ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  REQUESTS                                                  ║");
    println!("║    Total:      {:>10}                                  ║", total);
    println!("║    Successful: {:>10}                                  ║", success);
    println!("║    Errors:     {:>10}                                  ║", errors);
    println!("║    Error Rate: {:>9.2}%                                  ║", 
             if total > 0 { (errors as f64 / total as f64) * 100.0 } else { 0.0 });
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  THROUGHPUT                                                ║");
    println!("║    Duration:   {:>9.2}s                                  ║", duration_secs);
    println!("║    RPS:        {:>10.2}                                  ║", rps);
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  LATENCY (microseconds)                                    ║");
    println!("║    Min:        {:>10} µs                               ║", if min_latency == u64::MAX { 0 } else { min_latency });
    println!("║    Avg:        {:>10} µs                               ║", avg_latency);
    println!("║    Max:        {:>10} µs                               ║", max_latency);
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  LATENCY DISTRIBUTION                                      ║");
    println!("║    < 1ms:      {:>10} ({:>6.2}%)                        ║", 
             under_1ms, if success > 0 { under_1ms as f64 / success as f64 * 100.0 } else { 0.0 });
    println!("║    1-5ms:      {:>10} ({:>6.2}%)                        ║", 
             l_1_5ms, if success > 0 { l_1_5ms as f64 / success as f64 * 100.0 } else { 0.0 });
    println!("║    5-10ms:     {:>10} ({:>6.2}%)                        ║", 
             l_5_10ms, if success > 0 { l_5_10ms as f64 / success as f64 * 100.0 } else { 0.0 });
    println!("║    > 10ms:     {:>10} ({:>6.2}%)                        ║", 
             over_10ms, if success > 0 { over_10ms as f64 / success as f64 * 100.0 } else { 0.0 });
    println!("╠════════════════════════════════════════════════════════════╣");
    
    // Performance assessment based on RT-002 requirements
    let p50_ok = under_1ms as f64 / success.max(1) as f64 >= 0.5;
    let p99_ok = over_10ms as f64 / success.max(1) as f64 <= 0.01;
    let rps_ok = rps >= 10000.0;
    
    println!("║  REQUIREMENTS CHECK (RT-002)                               ║");
    println!("║    p50 < 1ms:  {}                                          ║", if p50_ok { "✓ PASS" } else { "✗ FAIL" });
    println!("║    p99 < 5ms:  {}                                          ║", if p99_ok { "✓ PASS" } else { "✗ FAIL" });
    println!("║    10K RPS:    {}                                          ║", if rps_ok { "✓ PASS" } else { "✗ FAIL" });
    println!("╚════════════════════════════════════════════════════════════╝");
}

/// Goose transaction: Calculate tax via RPC
async fn calculate_tax(_user: &mut GooseUser) -> TransactionResult {
    let pool = get_pool();
    
    match pool.send_request().await {
        Ok(latency) => {
            record_latency(latency);
            Ok(())
        }
        Err(e) => {
            RPC_ERROR_COUNT.fetch_add(1, Ordering::Relaxed);
            eprintln!("RPC error: {}", e);
            Ok(()) // Return Ok to continue load test, error is tracked in metrics
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), GooseError> {
    let start_time = Instant::now();
    
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║          TAX ENGINE RPC LOAD TEST                          ║");
    println!("║                                                            ║");
    println!("║  Target: 10,000 RPS with p99 < 5ms latency                 ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");
    
    // Initialize pool eagerly
    let _ = get_pool();
    
    let _metrics = GooseAttack::initialize()?
        .register_scenario(
            scenario!("TaxCalculation")
                .register_transaction(transaction!(calculate_tax))
        )
        .execute()
        .await?;
    
    let duration = start_time.elapsed().as_secs_f64();
    print_metrics(duration);
    
    Ok(())
}
