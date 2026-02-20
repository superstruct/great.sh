#!/usr/bin/env bash
# Cross-platform test script for great.sh Docker containers.
#
# Runs inside the container with /workspace mounted read-only.
# Copies source to /build, compiles, and runs the full test suite.
set -euo pipefail

PLATFORM="${PLATFORM:-unknown}"
echo "============================================"
echo "  great.sh test run — ${PLATFORM}"
echo "============================================"
echo ""

# Copy source (workspace is read-only mounted)
echo "[1/5] Copying source..."
cp -r /workspace/src /build/src
cp -r /workspace/tests /build/tests
cp -r /workspace/templates /build/templates
cp /workspace/Cargo.toml /build/Cargo.toml
[ -f /workspace/Cargo.lock ] && cp /workspace/Cargo.lock /build/Cargo.lock

# Build
echo "[2/5] Building (release)..."
cargo build --release 2>&1

# Run unit tests
echo "[3/5] Running unit tests..."
cargo test --release -- --nocapture 2>&1

# Run clippy
echo "[4/5] Running clippy..."
cargo clippy --release 2>&1

# Smoke test the binary
echo "[5/5] Smoke testing binary..."
BIN="./target/release/great"

${BIN} --version
${BIN} --help > /dev/null
${BIN} doctor 2>&1 || true
${BIN} template list 2>&1

echo ""
echo "============================================"
echo "  All tests passed — ${PLATFORM}"
echo "============================================"
