#!/usr/bin/env bash
# Setup development environment for hexaTuneProto.
#
# Configures git to use .githooks/ for commit-msg and pre-push hooks.
#
# Usage: scripts/setup.sh

set -euo pipefail

WORKSPACE_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$WORKSPACE_ROOT"

echo "=== hexaTuneProto Setup ==="
echo ""

# Configure git hooks path
echo "Configuring git hooks..."
git config core.hooksPath .githooks
chmod +x .githooks/commit-msg
chmod +x .githooks/pre-push
echo "  ✓ Git hooks path set to .githooks/"
echo "  ✓ commit-msg hook: validates conventional commit format"
echo "  ✓ pre-push hook: runs full CI (lint + build + test + no_std)"

echo ""
echo "=== Setup Complete ==="
echo ""
echo "Hooks active:"
echo "  commit  → commit message format check (docs/COMMIT_STRATEGY.md)"
echo "  push    → full CI pipeline (scripts/ci.sh)"
