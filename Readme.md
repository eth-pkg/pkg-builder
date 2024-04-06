# Pkg-builder

Pkg-builder simplifies the process of creating packages for Linux distributions. It automates the packaging process based on a configuration file, leveraging debcrafter, a framework for building Debian packages. By specifying package metadata, dependencies, and other configuration details in a structured format, developers can easily generate packages ready for Linux distributions. Pkg-builder abstracts away much of the complexity involved in packaging, allowing developers to focus on their software rather than packaging intricacies.

## Prerequisites

If you are using Debian, install sbuild, and various dependencies:

```bash
sudo apt install sbuild libssl-dev pkg-config quilt debhelper tar wget
sudo sbuild-adduser `whoami`

# if chroot not exists create it, TODO other cases 
sudo mkdir /srv/chroot 
sudo chown :sbuild /srv/chroot 
```


## Getting Started

### Example virtual package
```bash
cargo build 
cargo install . 
pkg-builder build-env create examples/bookworm/virtual-package/pkg-builder.toml
pkg-builder build-env create examples/bookworm/virtual-package/pkg-builder.toml
pkg-builder package examples/bookworm/virtual-package/pkg-builder.toml
```

This will build the package using the provided configuration file.

### Example rust package

```bash
cargo build 
cargo install . 
pkg-builder build-env create examples/bookworm/rust/hello-world/pkg-builder.toml
pkg-builder pkg-builder build-env create examples/bookworm/rust/hello-world/pkg-builder.toml
pkg-builder package examples/bookworm/rust/hello-world/pkg-builder.toml
```

### Example typescript package

```bash
cargo build 
cargo install . 
pkg-builder build-env create examples/bookworm/rust/hello-world/pkg-builder.toml
pkg-builder build-env create examples/bookworm/rust/hello-world/pkg-builder.toml
pkg-builder package examples/bookworm/rust/hello-world/pkg-builder.toml
```

### Example javascript package

```bash
cargo build 
cargo install . 
pkg-builder build-env create examples/bookworm/javascript/hello-world/pkg-builder.toml
pkg-builder build-env create examples/bookworm/javascript/hello-world/pkg-builder.toml
pkg-builder package examples/bookworm/javascript/hello-world/pkg-builder.toml
```

### Example nim package

```bash
cargo build 
cargo install . 
pkg-builder build-env create examples/bookworm/nim/hello-world/pkg-builder.toml
pkg-builder build-env create examples/bookworm/nim/hello-world/pkg-builder.toml
pkg-builder package examples/bookworm/nim/hello-world/pkg-builder.toml
```

### Example csharp package

```bash
cargo build 
cargo install . 
pkg-builder build-env create examples/bookworm/csharp/hello-world/pkg-builder.toml
pkg-builder build-env create examples/bookworm/csharp/hello-world/pkg-builder.toml
pkg-builder package examples/bookworm/csharp/hello-world/pkg-builder.toml
```

### Example java package

```bash
cargo build 
cargo install . 
pkg-builder build-env create examples/bookworm/java/hello-world/pkg-builder.toml
pkg-builder build-env create examples/bookworm/java/hello-world/pkg-builder.toml
pkg-builder package examples/bookworm/java/hello-world/pkg-builder.toml
```