# Cross-compilation for Linux aarch64 (ARM64)
#
# Usage:
#   docker build -f docker/cross-linux-aarch64.Dockerfile -t great-cross-aarch64 .
#   docker run --rm -v $(pwd):/workspace great-cross-aarch64
FROM rust:1.88-slim

RUN apt-get update && apt-get install -y \
    gcc-aarch64-linux-gnu \
    libc6-dev-arm64-cross \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add aarch64-unknown-linux-gnu

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc

WORKDIR /build

CMD ["bash", "/workspace/docker/cross-test-linux-aarch64.sh"]
