#!/usr/bin/env bash
# Generate changelog from conventional commits since the last tag.
#
# Usage: scripts/changelog.sh [version]
#   version: e.g. "v1.0.0" (optional, defaults to "Unreleased")
#
# Reads commits since the last git tag and groups them by type.
# Output goes to stdout — redirect to a file if needed.

set -euo pipefail

VERSION="${1:-Unreleased}"
DATE=$(date +%Y-%m-%d)

# Find the last tag, or use initial commit if none exists
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")

if [ -n "$LAST_TAG" ]; then
    RANGE="${LAST_TAG}..HEAD"
    COMPARE_NOTE="Full diff: compare ${LAST_TAG}...${VERSION}"
else
    RANGE="HEAD"
    COMPARE_NOTE="Initial release"
fi

# Collect commits by type
declare -A SECTIONS=(
    ["feat"]="Features"
    ["fix"]="Bug Fixes"
    ["docs"]="Documentation"
    ["refactor"]="Refactoring"
    ["perf"]="Performance"
    ["test"]="Tests"
    ["ci"]="CI/CD"
    ["build"]="Build"
    ["chore"]="Chores"
)

# Ordered keys for consistent output
ORDERED_KEYS=("feat" "fix" "docs" "refactor" "perf" "test" "ci" "build" "chore")

echo "## ${VERSION} (${DATE})"
echo ""

HAS_CONTENT=false

for key in "${ORDERED_KEYS[@]}"; do
    title="${SECTIONS[$key]}"
    # Match commits starting with "type:" or "type(scope):"
    commits=$(git log --oneline --no-merges "$RANGE" --grep="^${key}" --format="- %s (%h)" 2>/dev/null || true)

    if [ -n "$commits" ]; then
        HAS_CONTENT=true
        echo "### ${title}"
        echo ""
        echo "$commits"
        echo ""
    fi
done

# Catch any commits that don't match conventional format
other=$(git log --oneline --no-merges "$RANGE" \
    --format="- %s (%h)" 2>/dev/null | \
    grep -v -E "^- (feat|fix|docs|refactor|perf|test|ci|build|chore)" || true)

if [ -n "$other" ]; then
    HAS_CONTENT=true
    echo "### Other"
    echo ""
    echo "$other"
    echo ""
fi

if [ "$HAS_CONTENT" = false ]; then
    echo "No changes."
    echo ""
fi

echo "---"
echo "${COMPARE_NOTE}"
