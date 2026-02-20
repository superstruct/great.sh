#!/usr/bin/env bash
# macOS cross-compilation build + validation script.
#
# Runs inside the cross-macos container. Builds for both x86_64 and aarch64,
# validates the output binaries, and copies them to /workspace/test-files/
# for the macOS VM to pick up.
set -euo pipefail

# Source auto-detected osxcross env vars
source /etc/profile.d/osxcross-env.sh

TARGETS=(x86_64-apple-darwin aarch64-apple-darwin)

echo "============================================"
echo "  great.sh macOS cross-compilation"
echo "============================================"
echo ""

# Copy source to writable build dir (workspace is read-only)
echo "[1/4] Copying source..."
cp -r /workspace/src /build/src
cp -r /workspace/tests /build/tests
[ -d /workspace/templates ] && cp -r /workspace/templates /build/templates
cp /workspace/Cargo.toml /build/Cargo.toml
[ -f /workspace/Cargo.lock ] && cp /workspace/Cargo.lock /build/Cargo.lock
cd /build

# Build both targets
echo "[2/4] Building for ${#TARGETS[@]} targets..."
for target in "${TARGETS[@]}"; do
    echo "  -> Building ${target}..."
    cargo build --release --target "${target}"
done

# Validate binaries
echo "[3/4] Validating binaries..."
for target in "${TARGETS[@]}"; do
    bin="target/${target}/release/great"
    if [ ! -f "$bin" ]; then
        echo "FATAL: missing binary for ${target}: ${bin}"
        exit 1
    fi

    file_output=$(file "$bin")
    echo "  ${target}: ${file_output}"

    case "$target" in
        x86_64-apple-darwin)
            if ! echo "$file_output" | grep -q "Mach-O 64-bit.*x86_64"; then
                echo "FATAL: ${target} binary is not Mach-O x86_64"
                exit 1
            fi
            ;;
        aarch64-apple-darwin)
            if ! echo "$file_output" | grep -q "Mach-O 64-bit.*arm64"; then
                echo "FATAL: ${target} binary is not Mach-O arm64"
                exit 1
            fi
            ;;
    esac
done

# Export binaries to shared volume
echo "[4/4] Exporting binaries..."
mkdir -p /workspace/test-files
for target in "${TARGETS[@]}"; do
    src="target/${target}/release/great"
    # Name like great-x86_64-apple-darwin
    dest="/workspace/test-files/great-${target}"
    cp "$src" "$dest"
    size=$(du -h "$dest" | cut -f1)
    echo "  ${dest} (${size})"
done

echo ""
echo "============================================"
echo "  Cross-compilation complete"
echo "  Binaries in /workspace/test-files/"
echo "============================================"
