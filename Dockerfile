FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
  build-essential \
  gcc-arm-none-eabi \
  binutils-arm-none-eabi \
  make \
  python3 \
  git \
  curl \
  ca-certificates \
  && apt-get clean

# Install Rust (stable) + Cortex-M0 target + binutils helpers
RUN useradd -m builder
USER builder
WORKDIR /home/builder
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
ENV PATH="/home/builder/.cargo/bin:${PATH}"
RUN rustup target add thumbv6m-none-eabi \
 && rustup component add llvm-tools-preview \
 && cargo install cargo-binutils

# Workdir for your project (mounted at runtime)
WORKDIR /workdir
