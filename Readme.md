# pkg-builder

[![Tests](https://github.com/eth-pkg/pkg-builder/actions/workflows/tests.yml/badge.svg?branch=main)](https://github.com/eth-pkg/pkg-builder/actions/workflows/tests.yml)

A tool to create reproducible builds for Debian-based systems (Ubuntu Jammy, Noble, and Debian 12) using a TOML configuration file.

## Overview

pkg-builder uses debcrafter to generate Debian packages from a TOML config, streamlining reproducible packaging for developers.

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
pkg-builder env create path/to/pkg-builder.toml
pkg-builder package path/to/pkg-builder.toml
```

If no config file path is provided, `pkg-builder.toml` in the current directory is used.

## Commands

```bash
pkg-builder package path/to/pkg-builder.toml  # Build package
pkg-builder env create path/to/pkg-builder.toml  # Create build environment
pkg-builder env clean path/to/pkg-builder.toml  # Clean build environment
pkg-builder piuparts path/to/pkg-builder.toml  # Run piuparts tests
pkg-builder autopkgtests path/to/pkg-builder.toml  # Run autopkgtests
pkg-builder lintian path/to/pkg-builder.toml  # Run lintian checks
pkg-builder verify path/to/pkg-builder.toml  # Verify package hashes
pkg-builder version  # Show version
```

If no config file path is provided, `pkg-builder.toml` in the current directory is used.

## Testing

```bash
pkg-builder piuparts path/to/pkg-builder.toml  # Run piuparts tests
pkg-builder autopkgtests path/to/pkg-builder.toml  # Run autopkgtests
pkg-builder lintian path/to/pkg-builder.toml  # Run lintian checks
pkg-builder verify path/to/pkg-builder.toml  # Verify package hashes
```

If no config file path is provided, `pkg-builder.toml` in the current directory is used.

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
