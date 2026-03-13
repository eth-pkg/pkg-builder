# pkg-builder

[![Tests](https://github.com/eth-pkg/pkg-builder/actions/workflows/tests.yml/badge.svg?branch=main)](https://github.com/eth-pkg/pkg-builder/actions/workflows/tests.yml)

A tool to create reproducible builds for Debian-based systems (Ubuntu Jammy, Noble, and Debian 12) using a TOML configuration file.

## Overview

pkg-builder uses debcrafter to generate Debian packages from a TOML config, simplifying reproducible packaging for developers.

## Key Features

- TOML-based configuration
- Package types: default (tarballs), Git-based, virtual
- Build support: C/C++, Rust, Go, Python, TypeScript/JavaScript, Java, .NET, Nim
- Testing: piuparts (install/remove), autopkgtest (functionality), lintian (quality)
- Package verification with hashes
- Flexible build environments
- Reproducible builds for Ubuntu Jammy, Noble, and Debian 12

## Quick Start

### Prerequisites (Debian/Ubuntu)

```bash
sudo apt install libssl-dev pkg-config quilt debhelper tar wget autopkgtest vmdb2 qemu-system-x86 git-lfs uidmap
sudo sbuild-adduser `whoami`
```

See [installation docs](docs/install.md) for sbuild setup.

### Basic Usage

```bash
# Install pkg-builder
cargo install --path .

# Create environment and build package
pkg-builder --config path/to/pkg-builder.toml env create
pkg-builder --config path/to/pkg-builder.toml build
```

If `--config` is omitted, pkg-builder looks for `pkg-builder.toml` in the current directory.

## Commands

```bash
pkg-builder build                              # Build package
pkg-builder build --with-tests                 # Build and run enabled tests
pkg-builder env create                         # Create build environment
pkg-builder env clean                          # Remove build environment
pkg-builder test                               # Run all enabled tests
pkg-builder test lintian                       # Run lintian checks on host
pkg-builder test piuparts                      # Run piuparts tests
pkg-builder test autopkgtest                   # Run autopkgtest tests
pkg-builder verify                             # Verify package hashes
pkg-builder clean                              # Clean build artifacts
pkg-builder --version                          # Show version
```

Use `--config path/to/pkg-builder.toml` to specify a config file. If omitted, the current directory is used.

## Testing

```bash
pkg-builder test                               # Run all enabled tests
pkg-builder test lintian                       # Run lintian checks on host
pkg-builder test piuparts                      # Run piuparts tests
pkg-builder test autopkgtest                   # Run autopkgtest tests
pkg-builder verify                             # Verify package hashes
```

Use `--config path/to/pkg-builder.toml` to specify a config file. If omitted, the current directory is used.

## Examples

See [examples documentation](EXAMPLES.md) for sample configs:
- Virtual packages, Rust, TypeScript/JavaScript, Nim, .NET, Java, Python, Go

## Documentation

- [Installation Guide](docs/install.md)
- [Configuration Reference](docs/config.md)
- [Examples](docs/examples.md)
- [Packaging FAQ](docs/packaging.md)

## License

Apache License, Version 2.0. See [LICENSE](http://www.apache.org/licenses/LICENSE-2.0).
