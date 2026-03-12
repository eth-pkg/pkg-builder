#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Skip these package types
SKIP_PACKAGES=("dotnet" "dotnet-9" "rust" "git-package")

# All available distros
ALL_DISTROS=(bookworm noble trixie)

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

usage() {
    echo "Usage: $0 [OPTIONS] [DISTRO...]"
    echo ""
    echo "Build, verify, and update hashes for all example packages."
    echo ""
    echo "Arguments:"
    echo "  DISTRO...       Distros to build (default: all: ${ALL_DISTROS[*]})"
    echo ""
    echo "Options:"
    echo "  --skip PKG      Skip a package type (can be repeated, default: ${SKIP_PACKAGES[*]})"
    echo "  --only PKG      Only build this package type (can be repeated)"
    echo "  --no-clean       Skip the clean step before building"
    echo "  --no-verify      Skip verify step"
    echo "  --replace        Update hashes in toml when verify finds mismatches"
    echo "  --dry-run        Show what would be built without building"
    echo "  -h, --help       Show this help"
}

ONLY_PACKAGES=()
NO_CLEAN=false
NO_VERIFY=false
REPLACE=false
DRY_RUN=false
DISTROS=()

while [[ $# -gt 0 ]]; do
    case $1 in
        --skip)
            SKIP_PACKAGES+=("$2")
            shift 2
            ;;
        --only)
            ONLY_PACKAGES+=("$2")
            shift 2
            ;;
        --no-clean)
            NO_CLEAN=true
            shift
            ;;
        --no-verify)
            NO_VERIFY=true
            shift
            ;;
        --replace)
            REPLACE=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        -*)
            echo "Unknown option: $1"
            usage
            exit 1
            ;;
        *)
            DISTROS+=("$1")
            shift
            ;;
    esac
done

# Default to all distros if none specified
if [[ ${#DISTROS[@]} -eq 0 ]]; then
    DISTROS=("${ALL_DISTROS[@]}")
fi

should_skip() {
    local pkg="$1"
    # If --only is set, skip everything not in the list
    if [[ ${#ONLY_PACKAGES[@]} -gt 0 ]]; then
        for only in "${ONLY_PACKAGES[@]}"; do
            if [[ "$pkg" == "$only" ]]; then
                return 1
            fi
        done
        return 0
    fi
    # Otherwise check skip list
    for skip in "${SKIP_PACKAGES[@]}"; do
        if [[ "$pkg" == "$skip" ]]; then
            return 0
        fi
    done
    return 1
}

# Collect all toml files to process
TOMLS=()
for distro in "${DISTROS[@]}"; do
    if [[ ! -d "examples/$distro" ]]; then
        echo -e "${RED}Distro not found: examples/$distro${NC}"
        continue
    fi
    for toml in examples/"$distro"/*/hello-world/pkg-builder.toml examples/"$distro"/*/nimbus/pkg-builder.toml; do
        [[ -f "$toml" ]] || continue
        # Extract package type: examples/distro/TYPE/...
        pkg_type=$(echo "$toml" | cut -d/ -f3)
        if should_skip "$pkg_type"; then
            echo -e "${YELLOW}Skipping ${pkg_type} (${distro})${NC}"
            continue
        fi
        TOMLS+=("$toml")
    done
done

if [[ ${#TOMLS[@]} -eq 0 ]]; then
    echo -e "${RED}No packages found to build.${NC}"
    exit 1
fi

echo -e "${BLUE}Will process ${#TOMLS[@]} package(s):${NC}"
for toml in "${TOMLS[@]}"; do
    echo "  $toml"
done
echo ""

if [[ "$DRY_RUN" == true ]]; then
    echo -e "${YELLOW}Dry run, exiting.${NC}"
    exit 0
fi

FAILED=()
SUCCEEDED=()

for toml in "${TOMLS[@]}"; do
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}Processing: ${toml}${NC}"
    echo -e "${BLUE}========================================${NC}"

    # Step 1: Clean
    if [[ "$NO_CLEAN" != true ]]; then
        echo -e "${YELLOW}Cleaning...${NC}"
        if ! cargo run --bin pkg-builder -- clean "$toml" 2>&1; then
            echo -e "${YELLOW}Clean failed (continuing anyway)${NC}"
        fi
    fi

    # Step 2: Remove tmp folder contents
    echo -e "${YELLOW}Removing tmp folder contents...${NC}"
    sudo rm -rf /tmp/* 2>/dev/null || true


    # Step 3: Build
    echo -e "${YELLOW}Building...${NC}"
    if ! cargo run --bin pkg-builder -- package "$toml" 2>&1; then
        echo -e "${RED}BUILD FAILED: ${toml}${NC}"
        FAILED+=("$toml")
        continue
    fi

    # Step 4: Verify (and optionally update hashes with --replace)
    if [[ "$NO_VERIFY" != true ]]; then
        echo -e "${YELLOW}Verifying...${NC}"
        verify_output=$(cargo run --bin pkg-builder -- verify "$toml" 2>&1) || true
        echo "$verify_output"

        if echo "$verify_output" | grep -q "SHA1 mismatch"; then
            if [[ "$REPLACE" == true ]]; then
                echo -e "${YELLOW}Hash mismatch detected, updating hashes in ${toml}...${NC}"

                # Split errors (may be joined by "; " on one line) and process each
                while IFS= read -r match; do
                    file_name=$(echo "$match" | sed 's/SHA1 mismatch for \(.*\): expected.*/\1/')
                    new_hash=$(echo "$match" | sed 's/.*, got //')
                    echo -e "  Updating hash for ${file_name} -> ${new_hash}"
                    # Replace by matching the file name in the toml
                    sed -i "/${file_name}/s/hash = \"[a-f0-9]*\"/hash = \"${new_hash}\"/" "$toml"
                done < <(echo "$verify_output" | grep -o 'SHA1 mismatch for [^;]*')

                # Re-verify
                echo -e "${YELLOW}Re-verifying...${NC}"
                if cargo run --bin pkg-builder -- verify "$toml" 2>&1; then
                    echo -e "${GREEN}Verification passed after hash update!${NC}"
                else
                    echo -e "${RED}Verification still failing after hash update!${NC}"
                    FAILED+=("$toml")
                    continue
                fi
            else
                echo -e "${RED}Verify failed for ${toml}${NC}"
                FAILED+=("$toml")
                continue
            fi
        elif echo "$verify_output" | grep -q "No \[verify\] section"; then
            echo -e "${RED}No [verify] section in ${toml}${NC}"
            FAILED+=("$toml")
            continue
        elif echo "$verify_output" | grep -q "Verification successful"; then
            echo -e "${GREEN}Verification passed!${NC}"
        else
            echo -e "${RED}Verify failed for ${toml}${NC}"
            FAILED+=("$toml")
            continue
        fi
    fi

    SUCCEEDED+=("$toml")
done

echo -e "\n${BLUE}========================================${NC}"
echo -e "${BLUE}Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}Succeeded: ${#SUCCEEDED[@]}${NC}"
for s in "${SUCCEEDED[@]}"; do
    echo -e "  ${GREEN}✓${NC} $s"
done

if [[ ${#FAILED[@]} -gt 0 ]]; then
    echo -e "${RED}Failed: ${#FAILED[@]}${NC}"
    for f in "${FAILED[@]}"; do
        echo -e "  ${RED}✗${NC} $f"
    done
    exit 1
fi
