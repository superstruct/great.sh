#!/usr/bin/env bash
set -euo pipefail

# great.sh installer
# Usage: curl -sSL https://great.sh/install.sh | bash

REPO="superstruct/great.sh"
INSTALL_DIR="${GREAT_INSTALL_DIR:-$HOME/.local/bin}"

main() {
    echo "Installing great.sh..."
    echo

    # Detect platform
    local os arch target
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)  os="unknown-linux-gnu" ;;
        Darwin) os="apple-darwin" ;;
        *)
            echo "Error: Unsupported OS: $os"
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *)
            echo "Error: Unsupported architecture: $arch"
            exit 1
            ;;
    esac

    target="${arch}-${os}"
    echo "  Platform: ${target}"

    # Get latest release
    local latest_url
    latest_url="https://github.com/${REPO}/releases/latest/download/great-${target}.tar.gz"
    echo "  Downloading from: ${latest_url}"

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Download and extract
    local tmp
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT

    if command -v curl >/dev/null 2>&1; then
        curl -sSL "$latest_url" -o "$tmp/great.tar.gz"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$tmp/great.tar.gz" "$latest_url"
    else
        echo "Error: curl or wget required"
        exit 1
    fi

    tar xzf "$tmp/great.tar.gz" -C "$tmp"
    install -m 755 "$tmp/great" "$INSTALL_DIR/great"

    echo
    echo "  Installed to: $INSTALL_DIR/great"

    # Check if install dir is in PATH
    case ":$PATH:" in
        *":$INSTALL_DIR:"*) ;;
        *)
            echo
            echo "  Warning: $INSTALL_DIR is not in your PATH."
            echo "  Add this to your shell profile:"
            echo "    export PATH=\"$INSTALL_DIR:\$PATH\""
            ;;
    esac

    echo
    echo "  Run 'great init' to get started!"
}

main "$@"
