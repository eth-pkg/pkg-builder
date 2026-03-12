# pkg-builder

[![CI](https://github.com/eth-pkg/pkg-builder/actions/workflows/tests.yml/badge.svg)](https://github.com/eth-pkg/pkg-builder/actions/workflows/tests.yml)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://github.com/eth-pkg/pkg-builder/blob/main/LICENSE)

A command-line tool for creating reproducible Debian packages from TOML configuration files. It integrates with [debcrafter](https://github.com/Kixunil/debcrafter) to simplify the Debian packaging workflow, handling the entire pipeline from environment setup and source acquisition to package building and verification.

## Why pkg-builder?

Debian packaging is powerful but notoriously difficult to get right. A typical packaging workflow involves writing and maintaining `debian/rules`, `debian/control`, `debian/changelog`, source format files, and more — each with its own syntax and subtle interactions. Reproducibility is hard: builds depend on the exact state of upstream archives, toolchain versions, and build environments.

pkg-builder solves this by letting you define your entire package in a single `pkg-builder.toml` file. Instead of hand-writing Makefiles and debian control files, you declare what you want — source location, build distribution, runtime toolchain, test settings — and pkg-builder generates everything else. It pins snapshot archives, normalizes timestamps, and verifies output hashes so that the same config always produces the same `.deb`.

It works alongside [debcrafter](https://github.com/Kixunil/debcrafter), which generates the Debian specification files (`.sss` files) that describe package metadata and relationships. Together, they provide a fully declarative packaging pipeline: debcrafter handles *what* the package is, pkg-builder handles *how* it gets built.

## Quick Start

**1. Write a config file** — create `pkg-builder.toml` in your project directory:

```toml
[package]
name = "hello-world-c"
version = "1.0.0"
revision = "1"
homepage = "https://github.com/eth-pkg/pkg-builder#examples"
spec = "hello-world-c.sss"

[source]
type = "tarball"
url = "hello-world-1.0.0.tar.gz"
hash = "c93bdd829eca65af1e..."

[build]
distribution = "bookworm"
arch = "amd64"
workdir = "~/.pkg-builder/packages/bookworm"
```

**2. Create the build environment:**

```bash
pkg-builder env create pkg-builder.toml
```

**3. Build the package:**

```bash
pkg-builder package pkg-builder.toml
```

That's it — your `.deb` is in the workdir. Add `[testing]` and `[verify]` sections to enable lintian checks and hash verification.

## Features

- **TOML-driven configuration** — define your package in a single `pkg-builder.toml` file
- **Reproducible builds** — timestamp normalization, snapshot pinning, and hash verification
- **Multiple source types** — tarballs, git repositories (with submodule pinning), and virtual/meta-packages
- **Multi-language support** — C, Rust, Go, Node.js (JavaScript/TypeScript), Java (with Gradle), .NET, Nim
- **Multi-distribution** — Debian (bookworm, trixie) and Ubuntu (noble)
- **Integrated testing** — lintian, piuparts, and autopkgtest
- **Package verification** — SHA-1 hash checking for built `.dsc` and `.deb` files

## Installation

### Prerequisites

```bash
sudo apt install libssl-dev pkg-config quilt debhelper tar wget autopkgtest \
                 vmdb2 qemu-system-x86 git-lfs uidmap
sudo sbuild-adduser $(whoami)
```

### Build from source

```bash
cargo install --path .
```

## Usage

### Build a package

```bash
pkg-builder package path/to/pkg-builder.toml
```

If no config path is provided, pkg-builder looks for `pkg-builder.toml` in the current directory.

### Other commands

```bash
pkg-builder env create [CONFIG]    # Create the sbuild environment
pkg-builder env clean [CONFIG]     # Clean up the build environment
pkg-builder generate [CONFIG]      # Generate Makefile without building
pkg-builder lintian [CONFIG]       # Run lintian quality checks
pkg-builder piuparts [CONFIG]      # Run piuparts install/remove tests
pkg-builder autopkgtest [CONFIG]   # Run autopkgtest functional tests
pkg-builder verify [CONFIG]        # Verify package file hashes
pkg-builder clean [CONFIG]         # Clean build artifacts
```

The `package` command also accepts runtime flags:

```bash
pkg-builder package [CONFIG] --run-lintian true --run-piuparts --run-autopkgtest
```

## Configuration

A `pkg-builder.toml` file defines everything needed to build a package. Here's a full example for a C project:

```toml
[package]
name = "hello-world-c"
version = "1.0.0"
revision = "1"
homepage = "https://github.com/eth-pkg/pkg-builder#examples"
spec = "hello-world-c.sss"

[source]
type = "tarball"
url = "hello-world-1.0.0.tar.gz"
hash = "c93bdd829eca65af1e..."

[build]
distribution = "bookworm"
arch = "amd64"
workdir = "~/.pkg-builder/packages/bookworm"

[testing]
lintian = true
piuparts = true
autopkgtest = true

[tools]
pkg_builder = "0.3.1"
debcrafter = "8189263"
sbuild = "0.85.6"
lintian = "2.116.3"
piuparts = "1.1.7"
autopkgtest = "5.28"

[verify]
package_hash = [
    { name = "hello-world-c_1.0.0-1.dsc", hash = "0843091f9e3cff991450c9687935e67644f6b032" },
    { name = "hello-world-c_1.0.0-1_amd64.deb", hash = "acd656c4554f41eb2a0ad570ef99ff19db51273d" },
]
```

### Source types

**Tarball** — download or use a local source archive:

```toml
[source]
type = "tarball"
url = "https://example.com/source-1.0.0.tar.gz"
hash = "sha256_or_sha512_hash"
```

**Git** — clone a repository with optional submodule pinning:

```toml
[source]
type = "git"
url = "https://github.com/org/repo.git"
tag = "v1.0.0"
submodules = [
    { commit = "72523ee3f865...", path = "vendor/lib1" },
    { commit = "ab3ff9fad45f...", path = "vendor/lib2" },
]
```

**Virtual** — meta-packages with no source code:

```toml
[source]
type = "virtual"
```

### Runtime recipes

For languages that need external toolchains, add a `[runtime]` section:

```toml
[runtime]
recipe = "rust"
binary_url = "https://static.rust-lang.org/dist/rust-1.77.2-x86_64-unknown-linux-gnu.tar.xz"
binary_gpg_asc = "..."
```

Available recipes: `c`, `rust`, `go`, `node`, `java`, `java-gradle`, `nim`, `dotnet-noble`, `dotnet-debian`, `dotnet-backup`

### Snapshot pinning

For reproducible builds against frozen Debian package archives:

```toml
[build]
distribution = "trixie"
arch = "amd64"
workdir = "~/.pkg-builder/packages/trixie"
snapshot_date = "20250101"
snapshot_security_date = "20250115T000000Z"
```

Snapshot pinning is only available for Debian distributions.

## Examples

The `examples/` directory contains working configurations for all supported language and distribution combinations:

```
examples/
  bookworm/       — Debian stable (C, git-package)
  noble/          — Ubuntu 24.04 (C, .NET, Go, Java, JavaScript, Nim, Rust, TypeScript, virtual, git-package)
  trixie/         — Debian testing (C, .NET, Go, Java, JavaScript, Nim, Rust, TypeScript, virtual, git-package)
```

Each example includes a `pkg-builder.toml` and any supporting files needed for a complete build. They serve as both documentation and integration tests.

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
