#!/usr/bin/env bash
#
# Updates the pkg_builder version in all example pkg-builder.toml files
# to match the current CLI crate version.
#
# Usage: ./scripts/bump-example-versions.sh [version]
#   If no version is given, reads it from crates/cli/Cargo.toml

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

if [ $# -ge 1 ]; then
    VERSION="$1"
else
    VERSION=$(grep '^version' "$REPO_ROOT/crates/cli/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')
fi

if [ -z "$VERSION" ]; then
    echo "Error: could not determine version" >&2
    exit 1
fi

echo "Bumping pkg_builder version to $VERSION in example TOML files..."

count=0
while IFS= read -r -d '' f; do
    sed -i "s/^pkg_builder = \".*\"/pkg_builder = \"$VERSION\"/" "$f"
    count=$((count + 1))
done < <(find "$REPO_ROOT/examples" -name "pkg-builder.toml" -print0)

echo "Updated $count example files to pkg_builder = \"$VERSION\""
