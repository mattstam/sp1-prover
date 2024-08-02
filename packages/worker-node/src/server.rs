//! Server module for the worker node.

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use log::info;
use std::env;

use crate::prove::{generate_proof, ProofRequest};

/// Basic endpoint to check if the server is running
async fn ping_api() -> impl Responder {
    HttpResponse::Ok().body("pong")
}

/// Proof generation endpoint.
async fn generate_proof_api(program: web::Json<ProofRequest>) -> impl Responder {
    // Generate the proof and return the proving time.
    let proving_seconds = generate_proof(program.into_inner()).await;

    match proving_seconds {
        Ok(proving_seconds) => HttpResponse::Ok().json(proving_seconds),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

/// Start the worker node server.
pub async fn start_server() -> std::io::Result<()> {
    info!("Starting worker node server.");

    HttpServer::new(|| {
        App::new()
            .route("/ping", web::get().to(ping_api))
            .route("/prove", web::post().to(generate_proof_api))
    })
    .bind(
        &("0.0.0.0:".to_owned()
            + &env::var("SERVER_PORT")
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?),
    )?
    .run()
    .await
}
