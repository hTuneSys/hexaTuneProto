#!/usr/bin/env bash
set -euo pipefail

WORKSPACE_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$WORKSPACE_ROOT"

echo "Cleaning build artifacts..."
cargo clean

echo "Removing generated header..."
rm -f crates/hexa-tune-proto-ffi/include/hexa_tune_proto.h

echo "Clean complete."
