#!/usr/bin/env bash
set -euo pipefail

WORKSPACE_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$WORKSPACE_ROOT"

MODE="${1:-dev}"

case "$MODE" in
    dev)
        echo "Building workspace (dev)..."
        cargo build --workspace
        ;;
    release)
        echo "Building workspace (release)..."
        cargo build --workspace --release
        ;;
    ffi)
        echo "Building FFI crate (release, cdylib)..."
        cargo build -p hexa-tune-proto-ffi --release
        ;;
    *)
        echo "Usage: $0 [dev|release|ffi]"
        exit 1
        ;;
esac

echo "Build complete."
