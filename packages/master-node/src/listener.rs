use anyhow::Result;
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

pub async fn listener() -> Result<()> {
    let http_client = Client::new();

    let network_client = NetworkClient::new(&std::env::var("SP1_PRIVATE_KEY").unwrap());

    let proof_requests = network_client
        .get_proof_requests(ProofStatus::ProofRequested)
        .await?;

    if proof_requests.proofs.len() > 0 {
        let proof_request = proof_requests.proofs[0].clone();
        let claim_proof_res = network_client.claim_proof(&proof_request.proof_id).await?;
        println!("Claimed proof: {:?}", claim_proof_res);

        // Create ProofRequest object and send to an API endpoint.
        let proof_request = ProofRequest {
            proof_id: proof_request.proof_id,
            program_artifact_id: claim_proof_res.program_artifact_id,
            stdin_artifact_id: claim_proof_res.stdin_artifact_id,
            proof_artifact_id: claim_proof_res.proof_artifact_id,
        };

        let proof_request_json = serde_json::to_string(&proof_request)?;

        // Send the proof request to the API endpoint.
        let response = http_client
            .post(std::env::var("WORKER_NODE_ENDPOINT")? + "/prove")
            .header("Content-Type", "application/json")
            .body(proof_request_json)
            .send()
            .await?;

        let proving_seconds = response.text().await?;

        println!("Proving time: {:?}", proving_seconds);
    }

    Ok(())
}
