# Cross-compilation for Windows x86_64 (MinGW)
#
# Usage:
#   docker compose build windows-cross
#   docker compose run windows-cross
#   docker compose run windows-cross cargo build --release --target x86_64-pc-windows-gnu
FROM rust:1.88-slim

RUN apt-get update && apt-get install -y \
    gcc-mingw-w64-x86-64 \
    file \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-pc-windows-gnu

WORKDIR /workspace

CMD ["cargo", "build", "--release", "--target", "x86_64-pc-windows-gnu"]
