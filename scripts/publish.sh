#!/usr/bin/env bash
# scripts/publish.sh — Manual crate publishing helper
# Usage:
#   bash scripts/publish.sh bump <version>     Update all workspace versions
#   bash scripts/publish.sh proto              Publish hexa-tune-proto
#   bash scripts/publish.sh embedded           Publish hexa-tune-proto-embedded
set -euo pipefail

ROOT_CARGO="Cargo.toml"
ENV_FILE=".env"

load_token() {
    if [[ ! -f "$ENV_FILE" ]]; then
        echo "Error: $ENV_FILE not found. Create it with CARGO_REGISTRY_TOKEN=\"<token>\"" >&2
        exit 1
    fi
    # shellcheck source=/dev/null
    source "$ENV_FILE"
    if [[ -z "${CARGO_REGISTRY_TOKEN:-}" ]]; then
        echo "Error: CARGO_REGISTRY_TOKEN is not set in $ENV_FILE" >&2
        exit 1
    fi
    export CARGO_REGISTRY_TOKEN
}

usage() {
    echo "Usage: $0 <command> [args]"
    echo ""
    echo "Commands:"
    echo "  bump <version>   Update workspace.package and workspace.dependencies versions"
    echo "  proto            Publish hexa-tune-proto to crates.io"
    echo "  embedded         Publish hexa-tune-proto-embedded to crates.io"
    exit 1
}

cmd_bump() {
    local version="${1:?Version argument required (e.g. 0.1.2)}"

    echo "Bumping workspace version to ${version}..."

    # Update [workspace.package] version
    sed -i "s/^version = \".*\"/version = \"${version}\"/" "$ROOT_CARGO"

    # Update [workspace.dependencies] hexa-tune-proto version
    sed -i "s/\(hexa-tune-proto.*version = \"\)[^\"]*\"/\1${version}\"/" "$ROOT_CARGO"

    # Update lock file
    cargo update --workspace

    echo "Done. Updated versions:"
    grep 'version' "$ROOT_CARGO"
}

cmd_proto() {
    load_token
    echo "Publishing hexa-tune-proto..."
    cargo publish -p hexa-tune-proto
    echo "Published hexa-tune-proto successfully."
}

cmd_embedded() {
    load_token
    echo "Publishing hexa-tune-proto-embedded..."
    cargo publish -p hexa-tune-proto-embedded
    echo "Published hexa-tune-proto-embedded successfully."
}

case "${1:-}" in
    bump)     shift; cmd_bump "$@" ;;
    proto)    cmd_proto ;;
    embedded) cmd_embedded ;;
    *)        usage ;;
esac
