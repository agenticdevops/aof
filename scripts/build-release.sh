#!/bin/bash
# Build AOF release binaries for multiple platforms
# Usage: ./scripts/build-release.sh [VERSION]

set -e

# Configuration
PROJECT_NAME="aofctl"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="${REPO_ROOT}/target/release"
DIST_DIR="${REPO_ROOT}/dist"
VERSION="${1:-0.1.0}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🔨 AOF Release Builder${NC}"
echo "Version: $VERSION"
echo "Repository: $REPO_ROOT"
echo ""

# Create dist directory
mkdir -p "$DIST_DIR"

# Supported platforms
declare -A TARGETS=(
    ["x86_64-unknown-linux-gnu"]="linux-x86_64"
    ["aarch64-unknown-linux-gnu"]="linux-aarch64"
    ["x86_64-apple-darwin"]="macos-x86_64"
    ["aarch64-apple-darwin"]="macos-aarch64"
    ["x86_64-pc-windows-gnu"]="windows-x86_64"
)

echo -e "${YELLOW}📦 Building for platforms:${NC}"
for target in "${!TARGETS[@]}"; do
    echo "  - $target"
done
echo ""

# Build for each platform
for target in "${!TARGETS[@]}"; do
    platform="${TARGETS[$target]}"
    echo -e "${YELLOW}Building for $platform...${NC}"

    # Check if target is installed
    rustup target add "$target" 2>/dev/null || true

    # Build
    cd "$REPO_ROOT/aof"
    cargo build --release --target "$target" --package aofctl 2>&1 | grep -E "error|warning|Finished" || true

    # Check if build succeeded
    binary_path="${REPO_ROOT}/target/${target}/release/${PROJECT_NAME}"
    if [ ! -f "$binary_path" ]; then
        echo -e "${RED}❌ Build failed for $target${NC}"
        continue
    fi

    # Copy to dist with version suffix
    dist_file="${DIST_DIR}/aofctl-${VERSION}-${platform}"
    cp "$binary_path" "$dist_file"
    chmod +x "$dist_file"

    # Create checksums
    sha256sum "$dist_file" > "${dist_file}.sha256"

    echo -e "${GREEN}✅ Built: $dist_file${NC}"
done

echo ""
echo -e "${GREEN}✅ Build complete!${NC}"
echo -e "${YELLOW}📁 Artifacts in: $DIST_DIR${NC}"
echo ""
echo "Files created:"
ls -lh "$DIST_DIR" | tail -n +2 | awk '{print "  " $9 " (" $5 ")"}'
