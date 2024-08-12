//! Server module for the worker node.

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use log::info;
use sp1_sdk::ProverClient;
use std::net::SocketAddr;
use std::{env, sync::Arc};

use crate::prove::{generate_proof, ProofRequest};

/// Basic endpoint to check if the server is running
async fn ping_api() -> impl Responder {
    HttpResponse::Ok().body("pong")
}

/// Proof generation endpoint.
async fn generate_proof_api(
    program: web::Json<ProofRequest>,
    prover_client: web::Data<Arc<ProverClient>>,
) -> impl Responder {
    // Generate the proof and return the proving time.
    let proving_seconds =
        generate_proof(program.into_inner(), prover_client.get_ref().clone()).await;

    match proving_seconds {
        Ok(proving_seconds) => HttpResponse::Ok().json(proving_seconds),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

/// Start the worker node server.
pub async fn start_server(
    prover_client: Arc<ProverClient>,
) -> std::io::Result<(actix_web::dev::Server, SocketAddr)> {
    info!("Starting worker node server.");

    let port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("SERVER_PORT must be a number");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(prover_client.clone()))
            .route("/ping", web::get().to(ping_api))
            .route("/prove", web::post().to(generate_proof_api))
    })
    .bind(addr)?
    .run();

    Ok((server, addr))
}
