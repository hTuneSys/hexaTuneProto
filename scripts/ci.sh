#!/usr/bin/env bash
set -euo pipefail

WORKSPACE_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$WORKSPACE_ROOT"

echo "=== CI Pipeline ==="
echo ""

echo "[1/4] Lint..."
bash scripts/lint.sh all
echo ""

echo "[2/4] Build..."
bash scripts/build.sh dev
echo ""

echo "[3/4] Test..."
bash scripts/test.sh all
echo ""

echo "[4/4] no_std check..."
bash scripts/check-nostd.sh
echo ""

echo "=== CI Pipeline PASSED ==="
