# Fedora 39 test environment for great.sh
FROM fedora:39

# System dependencies
RUN dnf install -y \
    curl \
    git \
    gcc \
    make \
    openssl-devel \
    dbus-devel \
    libsecret-devel \
    && dnf clean all

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Pre-fetch dependencies
WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && \
    cargo fetch && \
    rm -rf src

LABEL description="great.sh test environment - Fedora 39"
