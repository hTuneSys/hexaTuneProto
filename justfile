# hexaTuneProto — Task Runner
# All commands delegate to scripts/ directory

# Default: show available commands
default:
    @just --list

# Build workspace (dev|release|ffi)
build mode="dev":
    bash scripts/build.sh {{mode}}

# Run tests (all|proto|embedded|ffi)
test scope="all":
    bash scripts/test.sh {{scope}}

# Lint: clippy + rustfmt check (all|clippy|fmt|fix)
lint action="all":
    bash scripts/lint.sh {{action}}

# Check no_std compilation
check-nostd:
    bash scripts/check-nostd.sh

# Generate C header via cbindgen
gen-header:
    bash scripts/gen-header.sh

# Run full CI pipeline (lint + build + test + no_std)
ci:
    bash scripts/ci.sh

# Clean build artifacts
clean:
    bash scripts/clean.sh
