use crate::statics::HTTP_CLIENT_WITH_MIDDLEWARE;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sp1_sdk::{network::client::NetworkClient, ProverClient, SP1Stdin};

use crate::artifact::Artifact;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProofRequest {
    pub proof_id: String,
    pub program_artifact_id: String,
    pub stdin_artifact_id: String,
    pub proof_artifact_id: String,
}

async fn fetch_artifacts(proof_req: ProofRequest) -> Result<(Vec<u8>, SP1Stdin)> {
    let http_client = HTTP_CLIENT_WITH_MIDDLEWARE.lock().unwrap().clone();
    let program_artifact = Artifact::new(&proof_req.program_artifact_id, "program");
    let program: Vec<u8> = program_artifact
        .download(&http_client)
        .await
        .map_err(anyhow::Error::from)?;

    let stdin_artifact = Artifact::new(&proof_req.stdin_artifact_id, "stdin");
    let stdin = stdin_artifact
        .download::<SP1Stdin>(&http_client)
        .await
        .map_err(anyhow::Error::from)?;

    Ok((program, stdin))
}

pub async fn prove(proof_req: ProofRequest) -> Result<u64> {
    println!("Proving proof");

    let (program, stdin) = fetch_artifacts(proof_req.clone()).await?;

    let client = ProverClient::new();
    let (pk, _) = client.setup(&program);
    let proof = client.prove(&pk, stdin).run()?;

    let http_client = HTTP_CLIENT_WITH_MIDDLEWARE.lock().unwrap().clone();
    let proof_artifact = Artifact::new(&proof_req.proof_artifact_id, "proof");
    proof_artifact.upload(&http_client, &proof).await?;

    let network_client = NetworkClient::new(&std::env::var("SP1_PRIVATE_KEY")?);
    let proving_seconds = network_client.fulfill_proof(&proof_req.proof_id).await?;

    println!("Proof fulfilled");
    Ok(proving_seconds.proving_seconds)
}
