#!/bin/bash

set -e

# Color output for clarity
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Check if version argument is provided
if [ -z "$1" ]; then
  echo -e "${RED}Error: version argument required${NC}"
  echo "Usage: ./scripts/release.sh <version>"
  echo "Example: ./scripts/release.sh 0.1.0"
  exit 1
fi

VERSION="$1"

# Validate semver format (X.Y.Z)
if ! [[ $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo -e "${RED}Error: invalid semver format${NC}"
  echo "Version must match X.Y.Z pattern (e.g., 0.1.0)"
  exit 1
fi

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

echo -e "${GREEN}Updating version to $VERSION...${NC}"

# Update package.json
echo "  → package.json"
sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" package.json

# Update src-tauri/Cargo.toml (within [package] section)
echo "  → src-tauri/Cargo.toml"
sed -i '' "/^\[package\]/,/^\[/ s/^version = \"[^\"]*\"/version = \"$VERSION\"/" src-tauri/Cargo.toml

# Update src-tauri/tauri.conf.json
echo "  → src-tauri/tauri.conf.json"
sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" src-tauri/tauri.conf.json

echo -e "${GREEN}Building release binary...${NC}"

# Setup environment and build
source ~/.cargo/env
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && . "$NVM_DIR/nvm.sh"
nvm use 22
pnpm tauri build

# Find and display DMG path
DMG_PATH=$(find src-tauri/target/release/bundle/dmg -name "*.dmg" -type f | head -1)

if [ -n "$DMG_PATH" ]; then
  echo -e "${GREEN}Release built successfully!${NC}"
  echo -e "DMG location: ${GREEN}$DMG_PATH${NC}"
else
  echo -e "${RED}Error: DMG file not found${NC}"
  exit 1
fi
