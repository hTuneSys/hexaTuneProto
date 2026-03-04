#!/usr/bin/env bash
# Build hexa-tune-proto-ffi for iOS targets.
#
# Prerequisites:
#   - macOS with Xcode command line tools
#   - Rust targets: rustup target add aarch64-apple-ios aarch64-apple-ios-sim
#
# Usage: scripts/build-ios.sh [debug|release]

set -euo pipefail

PROFILE="${1:-release}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
OUT_DIR="$ROOT_DIR/target/ios"

TARGETS=(
    "aarch64-apple-ios"
    "aarch64-apple-ios-sim"
)

# Validate platform
if [[ "$(uname)" != "Darwin" ]]; then
    echo "ERROR: iOS builds require macOS."
    exit 1
fi

# Check targets
for target in "${TARGETS[@]}"; do
    if ! rustup target list --installed | grep -q "$target"; then
        echo "Installing target: $target"
        rustup target add "$target"
    fi
done

BUILD_FLAGS=""
if [ "$PROFILE" = "release" ]; then
    BUILD_FLAGS="--release"
fi

echo "Building hexa-tune-proto-ffi for iOS ($PROFILE)..."

for target in "${TARGETS[@]}"; do
    echo "  → $target"
    cargo build -p hexa-tune-proto-ffi --target "$target" $BUILD_FLAGS
done

# Collect outputs
mkdir -p "$OUT_DIR"

for target in "${TARGETS[@]}"; do
    if [ "$PROFILE" = "release" ]; then
        src_a="$ROOT_DIR/target/$target/release/libhexa_tune_proto_ffi.a"
    else
        src_a="$ROOT_DIR/target/$target/debug/libhexa_tune_proto_ffi.a"
    fi

    if [ -f "$src_a" ]; then
        cp "$src_a" "$OUT_DIR/libhexa_tune_proto_ffi_${target}.a"
        echo "    ✓ $OUT_DIR/libhexa_tune_proto_ffi_${target}.a"
    else
        echo "    ✗ not found: $src_a"
    fi
done

# Create universal binary for simulator if both arch exist
DEVICE_LIB="$OUT_DIR/libhexa_tune_proto_ffi_aarch64-apple-ios.a"
SIM_LIB="$OUT_DIR/libhexa_tune_proto_ffi_aarch64-apple-ios-sim.a"
XCFRAMEWORK_DIR="$OUT_DIR/HexaTuneProto.xcframework"

if [ -f "$DEVICE_LIB" ] && [ -f "$SIM_LIB" ]; then
    echo ""
    echo "Creating XCFramework..."
    rm -rf "$XCFRAMEWORK_DIR"

    HEADER_DIR="$ROOT_DIR/crates/hexa-tune-proto-ffi/include"

    xcodebuild -create-xcframework \
        -library "$DEVICE_LIB" -headers "$HEADER_DIR" \
        -library "$SIM_LIB" -headers "$HEADER_DIR" \
        -output "$XCFRAMEWORK_DIR"

    echo "  ✓ $XCFRAMEWORK_DIR"
fi

echo ""
echo "iOS build complete. Output: $OUT_DIR"
