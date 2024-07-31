# Use the official Rust image
FROM rust:latest

# Create a new empty shell project
RUN USER=root cargo new --bin prover
WORKDIR /prover

# Copy our manifests and source code
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src

# Build the application
RUN cargo build --release

# Copy the build artifact from the target folder to the current directory
RUN cp target/release/prover-node .

# Set the binary as the entrypoint of the container
ENTRYPOINT ["./prover-node"]

# Expose the port the server is running on
EXPOSE 8080