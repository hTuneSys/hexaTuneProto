#!/usr/bin/env bash
set -euo pipefail

WORKSPACE_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$WORKSPACE_ROOT"

FFI_DIR="crates/hexa-tune-proto-ffi"
OUT_DIR="${FFI_DIR}/include"

mkdir -p "$OUT_DIR"

echo "Generating C header via cbindgen..."
cargo build -p hexa-tune-proto-ffi

HEADER="${OUT_DIR}/hexa_tune_proto.h"
if [ -f "$HEADER" ]; then
    echo "Header generated: ${HEADER}"
    echo "Size: $(wc -c < "$HEADER") bytes"
else
    echo "ERROR: Header not found at ${HEADER}"
    exit 1
fi
