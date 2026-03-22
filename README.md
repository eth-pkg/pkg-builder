# pkg-builder

[![CI](https://github.com/eth-pkg/pkg-builder/actions/workflows/tests.yml/badge.svg)](https://github.com/eth-pkg/pkg-builder/actions/workflows/tests.yml)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://github.com/eth-pkg/pkg-builder/blob/main/LICENSE)

A command-line tool for creating reproducible Debian packages from TOML configuration files. It integrates with [debcrafter](https://github.com/Kixunil/debcrafter) to simplify the Debian packaging workflow, handling the entire pipeline from environment setup and source acquisition to package building and verification.

## Why pkg-builder?

Debian packaging is powerful but notoriously difficult to get right. pkg-builder solves this by letting you define your entire package in a single `pkg-builder.toml` file. Instead of hand-writing debian control files, you declare what you want — source location, build distribution, runtime toolchain — and pkg-builder handles everything else. It pins snapshot archives, normalizes timestamps, and verifies output hashes so that the same config always produces the same `.deb`.

It works alongside [debcrafter](https://github.com/Kixunil/debcrafter), which generates the Debian specification files (`.sss` files) that describe package metadata and relationships. Together, they provide a fully declarative packaging pipeline: debcrafter handles *what* the package is, pkg-builder handles *how* it gets built.

## Quick Start

**1. Install prerequisites and build:**

```bash
sudo apt install libssl-dev pkg-config quilt debhelper tar git-lfs uidmap
sudo sbuild-adduser $(whoami)
cargo install --path .
```

**2. Create the build environment:**

```bash
pkg-builder --config pkg-builder.toml env create
```

**3. Build the package:**

```bash
pkg-builder --config pkg-builder.toml build
```

That's it — your `.deb` is in the workdir. See the [tutorial](docs/src/getting-started/first-package.md) for a complete walkthrough.

## Features

- **TOML-driven configuration** — define your package in a single `pkg-builder.toml` file
- **Reproducible builds** — timestamp normalization, snapshot pinning, and hash verification
- **Multiple source types** — tarballs, git repositories (with submodule pinning), and virtual/meta-packages
- **Multi-language support** — C, Rust, Go, Node.js (JavaScript/TypeScript), Java (with Gradle), .NET, Nim
- **Multi-distribution** — Debian (bookworm, trixie) and Ubuntu (noble)
- **Package verification** — SHA-256 hash checking for built `.dsc` and `.deb` files

## Documentation

Full documentation is in the [`docs/`](docs/src/SUMMARY.md) directory, built with [mdBook](https://rust-lang.github.io/mdBook/):

- **Getting Started** — [Installation](docs/src/getting-started/installation.md) | [Tutorial](docs/src/getting-started/first-package.md) | [Concepts](docs/src/getting-started/concepts.md)
- **Guides** — [Source Types](docs/src/guides/source-types.md) | [Runtime Recipes](docs/src/guides/runtime-recipes.md) | [Snapshot Pinning](docs/src/guides/snapshot-pinning.md) | [Patching](docs/src/guides/patching.md) | [Caching](docs/src/guides/caching.md) | [Verification](docs/src/guides/verification.md)
- **Reference** — [Configuration](docs/src/reference/config.md) | [CLI](docs/src/reference/cli.md) | [Recipes](docs/src/reference/recipes.md)
- [Troubleshooting](docs/src/troubleshooting.md)

To build and view the docs locally:

```bash
cargo install mdbook
mdbook serve docs/
```

## Examples

The `examples/` directory contains working configurations for all supported language and distribution combinations:

```
examples/
  bookworm/       — Debian stable (C, git-package)
  noble/          — Ubuntu 24.04 (C, .NET, Go, Java, JavaScript, Nim, Rust, TypeScript, virtual, git-package)
  trixie/         — Debian testing (C, .NET, Go, Java, JavaScript, Nim, Rust, TypeScript, virtual, git-package)
```

Each example includes a `pkg-builder.toml` and any supporting files needed for a complete build. They serve as both documentation and integration tests.

## Contributing

```bash
# Clone and build
git clone https://github.com/eth-pkg/pkg-builder.git
cd pkg-builder
cargo build

# Run tests
cargo test
```

Bug reports and pull requests are welcome on [GitHub](https://github.com/eth-pkg/pkg-builder/issues).

## License

This project is licensed under the [Apache License 2.0](LICENSE).
