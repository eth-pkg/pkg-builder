# pkg-builder

pkg-builder simplifies the process of creating packages for Linux distributions. It automates the packaging process based on a configuration file, leveraging debcrafter, a framework for building Debian packages. By specifying package metadata, dependencies, and other configuration details in a structured format, developers can easily generate packages ready for Linux distributions. Pkg-builder abstracts away much of the complexity involved in packaging, allowing developers to focus on their software rather than packaging intricacies.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Getting Started](#getting-started)
    - [Example Virtual Package](#example-virtual-package)
    - [Example Rust Package](#example-rust-package)
    - [Example TypeScript Package](#example-typescript-package)
    - [Example JavaScript Package](#example-javascript-package)
    - [Example Nim Package](#example-nim-package)
    - [Example .NET Package](#example-net-package)
    - [Example Java Package](#example-java-package)
3. [Piuparts Only](#piuparts-only)
4. [Autopkgtest Only](#autopkgtest-only)

## Prerequisites

If you are using Debian, install sbuild, and various dependencies:

```bash
sudo apt install libssl-dev pkg-config quilt debhelper tar wget autopkgtest vmdb2 qemu-system-x86 git-lfs uidmap
sudo sbuild-adduser `whoami`

# Install sbuild
git clone https://github.com/eth-pkg/sbuild.git 
cd sbuild  
# Install dependencies
sudo apt-get install -y dh-python dh-sequence-python3 libyaml-tiny-perl python3-all 
sudo apt-get install -y genisoimage
# Build the package
dpkg-buildpackage -us -uc 
# Install the newly built package 
cd .. && sudo dpkg -i sbuild_0.85.6_all.deb libsbuild-perl_0.85.6_all.deb

# if chroot not exists create it
sudo mkdir /srv/chroot 
sudo chown :sbuild /srv/chroot

# for noble builds
sudo ln -s /usr/share/debootstrap/scripts/gutsy /usr/share/debootstrap/scripts/noble
```

If you are building for Ubuntu on Bookworm, you need to manually download the ubuntu-archive-keyring:
[ubuntu-archive-keyring](https://salsa.debian.org/debian/ubuntu-keyring/-/raw/master/keyrings/ubuntu-archive-keyring.gpg?ref_type=heads)
and copy it into `/usr/share/keyrings`.

## Getting Started

### Example Virtual Package
<details>
<summary>Click to expand</summary>

```bash
cargo build && cargo install --path . 
pkg-builder env create examples/bookworm/virtual-package/pkg-builder.toml
pkg-builder package examples/bookworm/virtual-package/pkg-builder.toml
```

This will build the package using the provided configuration file.
</details>

### Example Rust Package
<details>
<summary>Click to expand</summary>

```bash
cargo build && cargo install --path . 
pkg-builder env create examples/bookworm/rust/hello-world/pkg-builder.toml
pkg-builder package examples/bookworm/rust/hello-world/pkg-builder.toml
```
</details>

### Example TypeScript Package
<details>
<summary>Click to expand</summary>

```bash
cargo build && cargo install --path . 
pkg-builder env create examples/bookworm/typescript/hello-world/pkg-builder.toml
pkg-builder package examples/bookworm/typescript/hello-world/pkg-builder.toml
```
</details>

### Example JavaScript Package
<details>
<summary>Click to expand</summary>

```bash
cargo build && cargo install --path . 
pkg-builder env create examples/bookworm/javascript/hello-world/pkg-builder.toml
pkg-builder package examples/bookworm/javascript/hello-world/pkg-builder.toml
```
</details>

### Example Nim Package
<details>
<summary>Click to expand</summary>

```bash
cargo build && cargo install --path . 
pkg-builder env create examples/bookworm/nim/hello-world/pkg-builder.toml
pkg-builder package examples/bookworm/nim/hello-world/pkg-builder.toml
```
</details>

### Example .NET Package
<details>
<summary>Click to expand</summary>

```bash
cargo build && cargo install --path . 
pkg-builder env create examples/bookworm/dotnet/hello-world/pkg-builder.toml
pkg-builder package examples/bookworm/dotnet/hello-world/pkg-builder.toml
```
</details>

### Example Java Package
<details>
<summary>Click to expand</summary>

```bash
cargo build && cargo install --path . 
pkg-builder env create examples/bookworm/java/hello-world/pkg-builder.toml
pkg-builder package examples/bookworm/java/hello-world/pkg-builder.toml
```
</details>

## Piuparts Only

Assuming that you already packaged your source before as such:

```bash
cargo build && cargo install --path . 
pkg-builder env create examples/bookworm/virtual-package/pkg-builder.toml
pkg-builder package examples/bookworm/virtual-package/pkg-builder.toml
```

you can run only piuparts:

```bash
pkg-builder piuparts examples/bookworm/virtual-package/pkg-builder.toml
```

## Autopkgtest Only

Assuming that you already packaged your source before as such:

```bash
cargo build && cargo install --path . 
pkg-builder env create examples/bookworm/virtual-package/pkg-builder.toml
pkg-builder package examples/bookworm/virtual-package/pkg-builder.toml
```

you can run only autopkgtests:

```bash
pkg-builder autopkgtests examples/bookworm/virtual-package/pkg-builder.toml
```
