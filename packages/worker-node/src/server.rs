use actix_web::{web, App, HttpResponse, HttpServer, Responder};

use crate::prove::{prove, ProofRequest};

// Basic endpoint to check if the server is running
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello, API!")
}

async fn prove_api(program: web::Json<ProofRequest>) -> impl Responder {
    let proving_seconds = prove(program.into_inner()).await;
    match proving_seconds {
        Ok(proving_seconds) => HttpResponse::Ok().json(proving_seconds),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

pub async fn start_server() -> std::io::Result<()> {
    println!("Starting server");

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/prove", web::post().to(prove_api))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
