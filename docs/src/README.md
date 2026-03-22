# Introduction

pkg-builder is a command-line tool for creating reproducible Debian packages from TOML configuration files. It integrates with [debcrafter](https://github.com/Kixunil/debcrafter) to simplify the Debian packaging workflow, handling the entire pipeline from environment setup and source acquisition to package building and verification.

## Why pkg-builder?

Debian packaging is powerful but notoriously difficult to get right. A typical packaging workflow involves writing and maintaining `debian/rules`, `debian/control`, `debian/changelog`, source format files, and more — each with its own syntax and subtle interactions. Reproducibility is hard: builds depend on the exact state of upstream archives, toolchain versions, and build environments.

pkg-builder solves this by letting you define your entire package in a single `pkg-builder.toml` file. Instead of hand-writing debian control files, you declare what you want — source location, build distribution, runtime toolchain — and pkg-builder handles everything else. It pins snapshot archives, normalizes timestamps, and verifies output hashes so that the same config always produces the same `.deb`.

## Features

- **TOML-driven configuration** — define your package in a single `pkg-builder.toml` file
- **Reproducible builds** — timestamp normalization, snapshot pinning, and hash verification
- **Multiple source types** — tarballs, git repositories (with submodule pinning), and virtual/meta-packages
- **Multi-language support** — C, Rust, Go, Node.js (JavaScript/TypeScript), Java (with Gradle), .NET, Nim
- **Multi-distribution** — Debian (bookworm, trixie) and Ubuntu (noble)
- **Package verification** — SHA-256 hash checking for built `.dsc` and `.deb` files
- **Interactive init wizard** — scaffold new packages with auto-detection of runtime toolchains

## Quick start

```bash
# Install prerequisites and build pkg-builder
sudo apt install libssl-dev pkg-config quilt debhelper tar git-lfs uidmap
cargo install --path crates/cli

# Create the build environment
pkg-builder --config pkg-builder.toml env create

# Build the package
pkg-builder --config pkg-builder.toml build
```

Or scaffold a new package interactively:

```bash
pkg-builder init --output ./my-package
```

See [Installation](getting-started/installation.md) for full setup instructions and [Your First Package](getting-started/first-package.md) for a step-by-step tutorial.

## Architecture

pkg-builder is a Rust workspace with five crates:

| Crate | Purpose |
|-------|---------|
| `crates/cli` | Command-line interface and argument parsing |
| `crates/config` | TOML parsing, validation, and configuration types |
| `crates/init` | Interactive project initialization wizard |
| `crates/executor` | Build pipeline executor (acquire, extract, debcrafter, sbuild) |
| `crates/update` | Package version update tool |

The build pipeline directly executes four steps: acquire source, extract tarball, generate debian packaging via debcrafter, and invoke sbuild.
