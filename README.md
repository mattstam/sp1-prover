# SP1 Prover

## Introduction
SP1 Prover is a cluster prover that fetches proof requests from Succinct Prover Network, proves them, and sends the proofs back to the network. It has a master-slave architecture, where the master node is responsible for managing the proof requests and distributing them to the slave nodes for processing. The slave nodes are responsible for proving the theorems and fulfilling the proof requests. Currently, there is only one slave node, but the system is designed to be scalable and can be easily extended to support multiple slave nodes.

## Usage
To get started with SP1 Prover, clone the repository and follow the setup instructions.

1. Clone the repository:
    ```
    git clone https://github.com/Bisht13/SP1-Prover.git
    cd SP1-Prover
    ```

2. Set up the environment variables for both the master node and worker node by creating `.env` files in their respective directories from the provided `.env.example` files:

    For the master node:
    ```
    cd packages/master-node
    cp .env.example .env
    # Edit the .env file with the appropriate values
    ```

    For the worker node:
    ```
    cd packages/worker-node
    cp .env.example .env
    # Edit the .env file with the appropriate values
    ```

    Replace the placeholder values in both `.env` files with the actual values required for your application.

3. After setting up your `.env` files, navigate back to the root directory and start the application using Docker:
    ```
    cd ../..
    docker-compose up --build
    ```

## Downloading Artifacts
The prover cluster requires integration with a prover network to process proof requests and submit proofs. This necessitates AWS credentials, whether you are accessing the Succinct Prover Network or running your own network instance. Ensure you provide the necessary AWS credentials by updating the environment variables in `packages/worker-node/.env`:
```plaintext
S3_BUCKET=<your-s3-bucket-name>
AWS_REGION=<your-aws-region>
AWS_ACCESS_KEY_ID=<your-access-key-id>
AWS_SECRET_ACCESS_KEY=<your-secret-access-key>
```
Please replace the placeholder values with your actual AWS credentials. The current system mandates these credentials to facilitate the download of required artifacts. Succinct is considering more streamlined approaches for artifact acquisition in future updates.

## Docker Build Time
The build time for the Docker image can be quite long due to the installation of various dependencies, mainly due to the `native-gnark` feature in `sp1-sdk`. This is a one-time process and subsequent builds will be faster due to caching.

