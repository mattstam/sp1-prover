FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /sp1-prover-master

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

RUN apt-get update && apt-get install -y clang libclang-dev wget && \
    wget https://dl.google.com/go/go1.22.5.linux-amd64.tar.gz && \
    tar -xvf go1.22.5.linux-amd64.tar.gz && \
    mv go /usr/local && \
    rm go1.22.5.linux-amd64.tar.gz
ENV PATH="/usr/local/go/bin:${PATH}"
RUN go version

COPY --from=planner /sp1-prover-master/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin sp1-prover-master

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y clang libclang-dev wget && \
    wget https://dl.google.com/go/go1.22.5.linux-amd64.tar.gz && \
    tar -xvf go1.22.5.linux-amd64.tar.gz && \
    mv go /usr/local && \
    rm go1.22.5.linux-amd64.tar.gz
ENV PATH="/usr/local/go/bin:${PATH}"
RUN go version

WORKDIR /sp1-prover-master
COPY --from=builder /sp1-prover-master/target/release/sp1-prover-master /usr/local/bin
ENTRYPOINT ["/usr/local/bin/sp1-prover-master"]