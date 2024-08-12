//! Worker node for the proof generation.

extern crate dotenv;

mod artifact;
mod prove;
mod s3;
mod server;
mod statics;

use std::sync::{atomic::AtomicBool, Arc};

use dotenv::dotenv;
use sp1_sdk::ProverClient;
use tokio::signal;

use crate::server::start_server;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    // Create ProverClient
    let prover = Arc::new(std::thread::spawn(ProverClient::new).join().unwrap());

    // Download plonk artifacts in the background
    let plonk_available = Arc::new(AtomicBool::new(false));
    let plonk_available_clone = plonk_available.clone();
    tokio::task::spawn_blocking(move || {
        let start_time = std::time::Instant::now();
        // TODO: detect if it finished correctly or not.
        sp1_sdk::install::try_install_plonk_bn254_artifacts();
        plonk_available_clone.store(true, std::sync::atomic::Ordering::Relaxed);
        let elapsed = start_time.elapsed();
        log::info!(
            "plonk artifacts ready after {:.1} min",
            elapsed.as_secs() / 60
        );
    });

    // Start the server
    let (server, addr) = start_server(prover.clone())
        .await
        .expect("Failed to start server");

    log::info!("Server running on {}. Press Ctrl-C to stop.", addr);

    // Spawn the server on a new task
    let server_task = tokio::spawn(server);

    // Wait for a ctrl-c signal
    signal::ctrl_c().await.expect("Failed to listen for ctrl-c");

    log::info!("Shutting down server...");

    // Stop the server
    server_task.abort();

    // Wait for the server to finish shutting down
    let _ = server_task.await;

    log::info!("Server stopped.");
}
