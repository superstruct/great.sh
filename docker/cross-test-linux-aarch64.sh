#!/usr/bin/env bash
# Linux aarch64 cross-compilation build + validation script.
#
# Runs inside the cross-linux-aarch64 container. Builds for aarch64-unknown-linux-gnu,
# validates the output binary, and copies it to /build/test-files/
# for testing on an ARM64 host.
set -euo pipefail

TARGET="aarch64-unknown-linux-gnu"

echo "============================================"
echo "  great.sh Linux aarch64 cross-compilation"
echo "============================================"
echo ""

# Print toolchain version for build log traceability
echo "Toolchain: $(rustc --version)"
echo ""

# Copy source to writable build dir (workspace is read-only)
echo "[1/4] Copying source..."
cp -r /workspace/src /build/src
cp -r /workspace/tests /build/tests
[ -d /workspace/templates ] && cp -r /workspace/templates /build/templates
[ -d /workspace/loop ] && cp -r /workspace/loop /build/loop
cp /workspace/Cargo.toml /build/Cargo.toml
[ -f /workspace/Cargo.lock ] && cp /workspace/Cargo.lock /build/Cargo.lock
cd /build

# Build
echo "[2/4] Building for ${TARGET}..."
cargo build --release --target "${TARGET}"

# Validate binary
echo "[3/4] Validating binary..."
bin="target/${TARGET}/release/great"
if [ ! -f "$bin" ]; then
    echo "FATAL: missing binary: ${bin}"
    exit 1
fi

file_output=$(file "$bin")
echo "  ${TARGET}: ${file_output}"

if ! echo "$file_output" | grep -q "ELF 64-bit.*ARM aarch64"; then
    echo "FATAL: ${TARGET} binary is not an ELF ARM aarch64 executable"
    exit 1
fi

# Export binary to shared volume.
# Output goes to /build/test-files/ (writable); /workspace is read-only.
echo "[4/4] Exporting binary..."
mkdir -p /build/test-files
dest="/build/test-files/great-${TARGET}"
cp "$bin" "$dest"
size=$(du -h "$dest" | cut -f1)
echo "  ${dest} (${size})"

echo ""
echo "============================================"
echo "  Cross-compilation complete"
echo "  Binary in /build/test-files/"
echo "============================================"
