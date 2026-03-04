#!/usr/bin/env bash
set -euo pipefail

WORKSPACE_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$WORKSPACE_ROOT"

echo "Checking no_std build (proto core)..."
cargo build -p hexa-tune-proto --no-default-features

echo "Checking no_std build (embedded adapter)..."
cargo build -p hexa-tune-proto-embedded --no-default-features

echo "no_std check complete."
