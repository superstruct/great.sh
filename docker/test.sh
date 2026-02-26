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

# Print toolchain version for build log traceability
echo "Toolchain: $(rustc --version)"
echo ""

# Copy source (workspace is read-only mounted)
echo "[1/5] Copying source..."
cp -r /workspace/src /build/src
cp -r /workspace/tests /build/tests
cp -r /workspace/templates /build/templates
cp -r /workspace/loop /build/loop
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
doctor_rc=0
${BIN} doctor 2>&1 || doctor_rc=$?
if [ "$doctor_rc" -ne 0 ]; then
    echo "[WARN] great doctor exited non-zero (exit ${doctor_rc})"
fi
${BIN} template list 2>&1

echo ""
echo "============================================"
echo "  All tests passed — ${PLATFORM}"
echo "============================================"
