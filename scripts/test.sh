#!/usr/bin/env bash
set -euo pipefail

WORKSPACE_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$WORKSPACE_ROOT"

SCOPE="${1:-all}"

case "$SCOPE" in
    all)
        echo "Running all workspace tests..."
        cargo test --workspace
        ;;
    proto)
        echo "Running proto core tests..."
        cargo test -p hexa-tune-proto
        ;;
    embedded)
        echo "Running embedded adapter tests..."
        cargo test -p hexa-tune-proto-embedded
        ;;
    ffi)
        echo "Running FFI adapter tests..."
        cargo test -p hexa-tune-proto-ffi
        ;;
    *)
        echo "Usage: $0 [all|proto|embedded|ffi]"
        exit 1
        ;;
esac

echo "Tests complete."
