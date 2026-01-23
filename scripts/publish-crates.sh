#!/bin/bash
# Publish AOF crates to crates.io in dependency order
#
# Prerequisites:
#   1. cargo login with your crates.io token
#   2. All tests passing: cargo test --workspace
#   3. Clean git status (all changes committed)
#
# Usage:
#   ./scripts/publish-crates.sh           # Dry run (default)
#   ./scripts/publish-crates.sh --publish # Actually publish

set -e

DRY_RUN=true
if [[ "$1" == "--publish" ]]; then
    DRY_RUN=false
    echo "🚀 Publishing crates to crates.io..."
else
    echo "🔍 Dry run mode (use --publish to actually publish)"
fi

# Crates in dependency order (leaf dependencies first)
CRATES=(
    "aof-core"
    "aof-mcp"
    "aof-llm"
    "aof-memory"
    "aof-tools"
    "aof-runtime"
    "aof-triggers"
    "aofctl"
)

# Wait time between publishes to allow crates.io index to update
WAIT_SECONDS=30

publish_crate() {
    local crate=$1
    echo ""
    echo "📦 Publishing $crate..."

    if $DRY_RUN; then
        cargo publish -p "$crate" --dry-run --allow-dirty
    else
        cargo publish -p "$crate"
        echo "⏳ Waiting ${WAIT_SECONDS}s for crates.io index to update..."
        sleep $WAIT_SECONDS
    fi
}

# Verify we're logged in
if ! cargo login --help > /dev/null 2>&1; then
    echo "❌ cargo not found. Please install Rust."
    exit 1
fi

# Check for uncommitted changes
if ! git diff --quiet; then
    if $DRY_RUN; then
        echo "⚠️  Uncommitted changes detected (allowed in dry-run mode)"
    else
        echo "❌ Uncommitted changes detected. Please commit or stash before publishing."
        git status --short
        exit 1
    fi
fi

# Run tests first
echo "🧪 Running tests..."
cargo test --workspace --lib 2>&1 | tail -5

echo ""
echo "Publishing order:"
for i in "${!CRATES[@]}"; do
    echo "  $((i+1)). ${CRATES[$i]}"
done

# Publish each crate
for crate in "${CRATES[@]}"; do
    publish_crate "$crate"
done

echo ""
if $DRY_RUN; then
    echo "✅ Dry run completed successfully!"
    echo ""
    echo "To actually publish, run:"
    echo "  ./scripts/publish-crates.sh --publish"
else
    echo "✅ All crates published successfully!"
    echo ""
    echo "Users can now install with:"
    echo "  cargo install aofctl"
fi
