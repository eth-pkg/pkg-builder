# Introduction

pkg-builder is a command-line tool for creating reproducible Debian packages from TOML configuration files. It integrates with [debcrafter](https://github.com/Kixunil/debcrafter) to simplify the Debian packaging workflow, handling the entire pipeline from environment setup and source acquisition to package building and verification.

## Why pkg-builder?

Debian packaging is powerful but notoriously difficult to get right. A typical packaging workflow involves writing and maintaining `debian/rules`, `debian/control`, `debian/changelog`, source format files, and more — each with its own syntax and subtle interactions. Reproducibility is hard: builds depend on the exact state of upstream archives, toolchain versions, and build environments.

pkg-builder solves this by letting you define your entire package in a single `pkg-builder.toml` file. Instead of hand-writing Makefiles and debian control files, you declare what you want — source location, build distribution, runtime toolchain, test settings — and pkg-builder generates everything else. It pins snapshot archives, normalizes timestamps, and verifies output hashes so that the same config always produces the same `.deb`.

## Features

- **TOML-driven configuration** — define your package in a single `pkg-builder.toml` file
- **Reproducible builds** — timestamp normalization, snapshot pinning, and hash verification
- **Multiple source types** — tarballs, git repositories (with submodule pinning), and virtual/meta-packages
- **Multi-language support** — C, Rust, Go, Node.js (JavaScript/TypeScript), Java (with Gradle), .NET, Nim
- **Multi-distribution** — Debian (bookworm, trixie) and Ubuntu (noble)
- **Integrated testing** — lintian, piuparts, and autopkgtest
- **Package verification** — SHA-1 hash checking for built `.dsc` and `.deb` files

## Quick start

```bash
# Install prerequisites and build pkg-builder
sudo apt install libssl-dev pkg-config quilt debhelper tar wget autopkgtest \
                 vmdb2 qemu-system-x86 git-lfs uidmap
cargo install --path .

# Create the build environment
pkg-builder env create pkg-builder.toml

# Build the package
pkg-builder package pkg-builder.toml
```

See [Installation](getting-started/installation.md) for full setup instructions and [Your First Package](getting-started/first-package.md) for a step-by-step tutorial.

## Architecture

pkg-builder is a Rust workspace with five crates:

| Crate | Purpose |
|-------|---------|
| `crates/cli` | Command-line interface and argument parsing |
| `crates/config` | TOML parsing, validation, and configuration types |
| `crates/pipeline` | Build pipeline orchestration |
| `crates/tool` | Wrappers for external tools (git, tar, sbuild, lintian, etc.) |
| `crates/makefile` | Makefile generation from recipe templates |

The build pipeline works by generating a Makefile from distribution-specific and language-specific recipe templates, then executing it via `make`.
