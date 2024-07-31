use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sp1_sdk::{ProverClient, SP1ProofWithPublicValues, SP1Stdin};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Program {
    pub elf: Vec<u8>,
    pub stdin: SP1Stdin,
}

// Basic endpoint to check if the server is running
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello, API!")
}

fn prove(program: Program) -> SP1ProofWithPublicValues {
    println!("Proving program");

    let client = ProverClient::new();
    let (pk, _) = client.setup(&program.elf);
    let proof = client.prove(&pk, program.stdin).run().unwrap();

    println!("Proof generated");
    proof
}

async fn prove_endpoint(program: web::Json<Program>) -> impl Responder {
    let proof = prove(program.into_inner());
    HttpResponse::Ok().json(proof) // Respond with the proof as JSON
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server");

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/prove", web::post().to(prove_endpoint))
    })
    .bind("0.0.0.0:8080")? // Listen on localhost port 8080
    .run()
    .await
}
