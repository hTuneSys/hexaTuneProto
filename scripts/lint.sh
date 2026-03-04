#!/usr/bin/env bash
set -euo pipefail

WORKSPACE_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$WORKSPACE_ROOT"

ACTION="${1:-all}"

case "$ACTION" in
    all)
        echo "Running clippy..."
        cargo clippy --workspace -- -D warnings
        echo ""
        echo "Checking formatting..."
        cargo fmt --all -- --check
        ;;
    clippy)
        echo "Running clippy..."
        cargo clippy --workspace -- -D warnings
        ;;
    fmt)
        echo "Checking formatting..."
        cargo fmt --all -- --check
        ;;
    fix)
        echo "Applying rustfmt..."
        cargo fmt --all
        echo "Formatting applied."
        ;;
    *)
        echo "Usage: $0 [all|clippy|fmt|fix]"
        exit 1
        ;;
esac

echo "Lint complete."
