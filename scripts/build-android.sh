#!/usr/bin/env bash
# Build hexa-tune-proto-ffi for Android targets using cargo-ndk.
#
# Prerequisites:
#   - Android NDK installed (ANDROID_NDK_HOME set)
#   - cargo-ndk: cargo install cargo-ndk
#   - Rust targets: rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
#
# Usage: scripts/build-android.sh [debug|release]

set -euo pipefail

PROFILE="${1:-release}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
FFI_DIR="$ROOT_DIR/crates/hexa-tune-proto-ffi"
OUT_DIR="$ROOT_DIR/target/android"

TARGETS=(
    "aarch64-linux-android"
    "armv7-linux-androideabi"
    "x86_64-linux-android"
)

# Map Rust target triples to Android ABI names for jniLibs
declare -A ABI_MAP=(
    ["aarch64-linux-android"]="arm64-v8a"
    ["armv7-linux-androideabi"]="armeabi-v7a"
    ["x86_64-linux-android"]="x86_64"
)

# Validate prerequisites
if ! command -v cargo-ndk &>/dev/null; then
    echo "ERROR: cargo-ndk not found. Install with: cargo install cargo-ndk"
    exit 1
fi

if [ -z "${ANDROID_NDK_HOME:-}" ]; then
    echo "ERROR: ANDROID_NDK_HOME not set."
    exit 1
fi

# Check targets
for target in "${TARGETS[@]}"; do
    if ! rustup target list --installed | grep -q "$target"; then
        echo "Installing target: $target"
        rustup target add "$target"
    fi
done

# Build flags
BUILD_FLAGS=""
if [ "$PROFILE" = "release" ]; then
    BUILD_FLAGS="--release"
fi

echo "Building hexa-tune-proto-ffi for Android ($PROFILE)..."

for target in "${TARGETS[@]}"; do
    abi="${ABI_MAP[$target]}"
    echo "  → $target ($abi)"
    cargo ndk --target "$target" build -p hexa-tune-proto-ffi $BUILD_FLAGS
done

# Collect outputs into jniLibs structure
JNILIBS_DIR="$OUT_DIR/jniLibs"
mkdir -p "$JNILIBS_DIR"

for target in "${TARGETS[@]}"; do
    abi="${ABI_MAP[$target]}"
    mkdir -p "$JNILIBS_DIR/$abi"

    if [ "$PROFILE" = "release" ]; then
        src="$ROOT_DIR/target/$target/release/libhexa_tune_proto_ffi.so"
    else
        src="$ROOT_DIR/target/$target/debug/libhexa_tune_proto_ffi.so"
    fi

    if [ -f "$src" ]; then
        cp "$src" "$JNILIBS_DIR/$abi/"
        echo "    ✓ $JNILIBS_DIR/$abi/libhexa_tune_proto_ffi.so"
    else
        echo "    ✗ not found: $src"
    fi
done

echo ""
echo "Android build complete. Output: $JNILIBS_DIR"
echo "Copy jniLibs/ to your Flutter project's android/app/src/main/"
