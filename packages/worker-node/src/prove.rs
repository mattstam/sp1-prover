//! This module contains the logic for proving a proof request.

use std::sync::Arc;

use anyhow::Result;
use log::info;
use serde::{Deserialize, Serialize};
use sp1_sdk::proto::network::ProofMode;
use sp1_sdk::{network::client::NetworkClient, ProverClient, SP1Stdin};

use crate::artifact::Artifact;
use crate::statics::HTTP_CLIENT_WITH_MIDDLEWARE;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProofRequest {
    pub proof_id: String,
    pub mode: ProofMode,
    pub program_artifact_id: String,
    pub stdin_artifact_id: String,
    pub proof_artifact_id: String,
}

/// Fetch the program and stdin artifacts from S3.
async fn fetch_artifacts(
    program_artifact_id: String,
    stdin_artifact_id: String,
) -> Result<(Vec<u8>, SP1Stdin)> {
    let http_client = HTTP_CLIENT_WITH_MIDDLEWARE.lock().unwrap().clone();

    // Fetch the program artifact.
    let program_artifact = Artifact::new(&program_artifact_id, "program");
    let program: Vec<u8> = program_artifact
        .download(&http_client)
        .await
        .map_err(anyhow::Error::from)?;

    // Fetch the stdin artifact.
    let stdin_artifact = Artifact::new(&stdin_artifact_id, "stdin");
    let stdin = stdin_artifact
        .download::<SP1Stdin>(&http_client)
        .await
        .map_err(anyhow::Error::from)?;

    Ok((program, stdin))
}

/// Generate the proof for the proof request.
pub async fn generate_proof(proof_req: ProofRequest, client: Arc<ProverClient>) -> Result<u64> {
    info!(
        "Generating proof for proof with ID '{}'",
        proof_req.proof_id
    );

    // Fetch the program and stdin artifacts.
    let (program, stdin) =
        fetch_artifacts(proof_req.program_artifact_id, proof_req.stdin_artifact_id).await?;

    // Setup the proving key.
    let (pk, _) = client.setup(&program);

    // Generate the proof.
    let proof = tokio::task::spawn_blocking(move || match proof_req.mode {
        ProofMode::Unspecified => Err(anyhow::anyhow!("Unspecified proof mode is not valid")),
        ProofMode::Core => client.prove(&pk, stdin).run(),
        ProofMode::Compressed => client.prove(&pk, stdin).compressed().run(),
        ProofMode::Plonk => client.prove(&pk, stdin).plonk().run(),
        ProofMode::Groth16 => client.prove(&pk, stdin).groth16().run(),
    })
    .await??;

    // Upload the proof artifact to S3.
    let http_client = HTTP_CLIENT_WITH_MIDDLEWARE.lock().unwrap().clone();
    let proof_artifact = Artifact::new(&proof_req.proof_artifact_id, "proof");
    proof_artifact.upload(&http_client, &proof).await?;

    // Fulfill the proof request.
    let network_client = NetworkClient::new(&std::env::var("SP1_PRIVATE_KEY")?);
    let proving_seconds = network_client.fulfill_proof(&proof_req.proof_id).await?;

    info!(
        "Proof with ID '{}' has been successfully generated and fulfilled.",
        proof_req.proof_id
    );

    Ok(proving_seconds.proving_seconds)
}
