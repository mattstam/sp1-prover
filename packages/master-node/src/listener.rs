use anyhow::Result;
use log::info;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sp1_sdk::{network::client::NetworkClient, proto::network::ProofStatus};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProofRequest {
    pub proof_id: String,
    pub program_artifact_id: String,
    pub stdin_artifact_id: String,
    pub proof_artifact_id: String,
}

/// Listener function to listen for proof requests.
/// This function will be called every second to check for new proof requests.
/// If a proof request is found, it will claim the proof and send the proof request to a worker API endpoint to generate the proof.
pub async fn listener() -> Result<()> {
    // Create a new HTTP client and network client.
    let http_client = Client::new();
    let network_client = NetworkClient::new(&std::env::var("SP1_PRIVATE_KEY").unwrap());

    // Get proof requests with status ProofRequested.
    let proof_requests = network_client
        .get_proof_requests(ProofStatus::ProofRequested)
        .await?;

    // If there are proof requests, claim the first one and send it to the worker API endpoint.
    if proof_requests.proofs.len() > 0 {
        let proof_request = proof_requests.proofs[0].clone();
        let claim_proof_res = network_client.claim_proof(&proof_request.proof_id).await?;
        info!(
            "Proof with ID '{}' has been successfully claimed.",
            proof_request.proof_id
        );

        // Create ProofRequest object.
        let proof_request = ProofRequest {
            proof_id: proof_request.proof_id,
            program_artifact_id: claim_proof_res.program_artifact_id,
            stdin_artifact_id: claim_proof_res.stdin_artifact_id,
            proof_artifact_id: claim_proof_res.proof_artifact_id,
        };

        let proof_request_json = serde_json::to_string(&proof_request)?;

        info!("Sending proof request to worker API endpoint.");

        // Send the proof request to the worker API endpoint.
        let response = http_client
            .post(std::env::var("WORKER_NODE_ENDPOINT")? + "/prove")
            .header("Content-Type", "application/json")
            .body(proof_request_json)
            .send()
            .await?;

        // Get the proving time from the response.
        let proving_seconds = response.text().await?;

        info!(
            "Proof with ID '{}' has been successfully generated in {} seconds.",
            proof_request.proof_id, proving_seconds
        );
    }

    Ok(())
}
