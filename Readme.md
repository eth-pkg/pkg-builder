# pkg-builder

A tool that simplifies building packages for Linux distributions by automating the packaging process using a configuration file approach.

## Overview

pkg-builder leverages debcrafter to generate Debian packages from a structured configuration file. This abstraction allows developers to focus on their software rather than packaging complexities.

## Quick Start

### Prerequisites

For Debian systems:
```bash
sudo apt install libssl-dev pkg-config quilt debhelper tar wget autopkgtest vmdb2 qemu-system-x86 git-lfs uidmap
sudo sbuild-adduser `whoami`
```

For sbuild installation and setup, see [installation docs](INSTALL.md).

### Basic Usage

```bash
# Install pkg-builder
cargo install --path .

# Create environment and build package
pkg-builder env create path/to/pkg-builder.toml
pkg-builder package path/to/pkg-builder.toml
```

### Testing

```bash
# Run piuparts tests only
pkg-builder piuparts path/to/pkg-builder.toml

# Run autopkgtests only
pkg-builder autopkgtests path/to/pkg-builder.toml
```

## Examples

See [examples documentation](EXAMPLES.md) for sample configurations for various languages:
- Virtual Packages
- Rust
- TypeScript/JavaScript
- Nim
- .NET
- Java

## Documentation

- [Installation Guide](INSTALL.md)
- [Configuration Reference](CONFIG.md)
- [Examples](EXAMPLES.md)