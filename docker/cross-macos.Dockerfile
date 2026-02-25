# Cross-compilation for macOS x86_64 and aarch64 via osxcross
#
# Usage:
#   docker compose build macos-cross
#   docker compose run macos-cross
#   docker compose run macos-cross cargo build --release --target x86_64-apple-darwin
#
# The osxcross image is FROM scratch upstream (no shell, no OS userland).
# We use it as a named stage and COPY the toolchain into a real Ubuntu base.

# Stage 1: pinned osxcross toolchain source (FROM scratch -- not runnable)
FROM crazymax/osxcross:26.1-r0-ubuntu AS osxcross

# Stage 2: real Ubuntu base with shell and package manager
FROM ubuntu:24.04

# Copy the osxcross toolchain from the source stage
COPY --from=osxcross /osxcross /osxcross

# Install system dependencies needed for Rust compilation
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    clang \
    lld \
    pkg-config \
    ca-certificates \
    file \
    && rm -rf /var/lib/apt/lists/*

# Make osxcross binaries available on PATH
ENV PATH="/osxcross/bin:${PATH}"
ENV LD_LIBRARY_PATH="/osxcross/lib:${LD_LIBRARY_PATH}"

# Install Rust toolchain with both macOS targets
ENV RUSTUP_HOME=/opt/rust
ENV CARGO_HOME=/opt/rust
ENV PATH="/opt/rust/bin:${PATH}"

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- \
    -y \
    --default-toolchain 1.88.0 \
    --profile minimal \
    --no-modify-path

RUN rustup target add x86_64-apple-darwin aarch64-apple-darwin

# Auto-detect osxcross tool paths and generate cargo config + env vars.
# This avoids hardcoding SDK version strings like "darwin24.5".
RUN set -e && \
    X86_CLANG=$(find /osxcross/bin -name 'x86_64-apple-darwin*-clang' -not -name '*++' | head -1) && \
    X86_AR=$(find /osxcross/bin -name 'x86_64-apple-darwin*-ar' | head -1) && \
    ARM_CLANG=$(find /osxcross/bin -name 'aarch64-apple-darwin*-clang' -not -name '*++' | head -1) && \
    ARM_AR=$(find /osxcross/bin -name 'aarch64-apple-darwin*-ar' | head -1) && \
    test -n "$X86_CLANG" || { echo "FATAL: x86_64 clang not found in /osxcross/bin"; exit 1; } && \
    test -n "$ARM_CLANG" || { echo "FATAL: aarch64 clang not found in /osxcross/bin"; exit 1; } && \
    echo "Detected x86_64 clang: $X86_CLANG" && \
    echo "Detected aarch64 clang: $ARM_CLANG" && \
    mkdir -p /root/.cargo && \
    printf '[target.x86_64-apple-darwin]\nlinker = "%s"\nar = "%s"\n\n' "$X86_CLANG" "$X86_AR" > /root/.cargo/config.toml && \
    printf '[target.aarch64-apple-darwin]\nlinker = "%s"\nar = "%s"\n' "$ARM_CLANG" "$ARM_AR" >> /root/.cargo/config.toml && \
    echo "export CC_x86_64_apple_darwin=\"$X86_CLANG\"" >> /etc/profile.d/osxcross-env.sh && \
    echo "export CXX_x86_64_apple_darwin=\"${X86_CLANG}++\"" >> /etc/profile.d/osxcross-env.sh && \
    echo "export AR_x86_64_apple_darwin=\"$X86_AR\"" >> /etc/profile.d/osxcross-env.sh && \
    echo "export CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER=\"$X86_CLANG\"" >> /etc/profile.d/osxcross-env.sh && \
    echo "export CC_aarch64_apple_darwin=\"$ARM_CLANG\"" >> /etc/profile.d/osxcross-env.sh && \
    echo "export CXX_aarch64_apple_darwin=\"${ARM_CLANG}++\"" >> /etc/profile.d/osxcross-env.sh && \
    echo "export AR_aarch64_apple_darwin=\"$ARM_AR\"" >> /etc/profile.d/osxcross-env.sh && \
    echo "export CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER=\"$ARM_CLANG\"" >> /etc/profile.d/osxcross-env.sh

# Source env vars at build time too (for the pre-fetch step)
SHELL ["/bin/bash", "-c"]
RUN source /etc/profile.d/osxcross-env.sh && env | grep -E '^(CC_|CXX_|AR_|CARGO_TARGET_)' >> /etc/environment

# Pre-fetch dependencies using real Cargo.toml + Cargo.lock
WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && \
    cargo fetch && \
    rm -rf src

WORKDIR /workspace

CMD ["bash", "/workspace/docker/cross-test-macos.sh"]
