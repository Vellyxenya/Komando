FROM mcr.microsoft.com/devcontainers/base:ubuntu-22.04

# Install Rust and other dependencies
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    build-essential \
    curl \
    git \
    libssl-dev \
    pkg-config \
    ca-certificates

# Install Rust for the vscode user
USER vscode
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/home/vscode/.cargo/bin:${PATH}"

# Pre-install some common components
RUN rustup component add rust-analyzer rustfmt clippy

USER root
# Ensure the vscode user has permissions to common cargo directories
RUN mkdir -p /home/vscode/.cargo/registry && chown -R vscode:vscode /home/vscode/.cargo
