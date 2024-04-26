# Pkg-builder

Pkg-builder simplifies the process of creating packages for Linux distributions. It automates the packaging process based on a configuration file, leveraging debcrafter, a framework for building Debian packages. By specifying package metadata, dependencies, and other configuration details in a structured format, developers can easily generate packages ready for Linux distributions. Pkg-builder abstracts away much of the complexity involved in packaging, allowing developers to focus on their software rather than packaging intricacies.

## Prerequisites

If you are using Debian, install sbuild, and various dependencies:

```bash
sudo apt install libssl-dev pkg-config quilt debhelper tar wget vmdb2
sudo sbuild-adduser `whoami`

# Install sbuild
git clone https://github.com/eth-pkg/sbuild.git ~/<DIR>/sbuild 
cd ~/<DIR>/sbuild  
# Install dependencies
sudo apt-get install dh-python dh-sequence-python3 libyaml-tiny-perl python3-all 
# Build the package
dpkg-buildpackage -us -uc  
# Install the newly built package 
cd .. && sudo dpkg -i sbuild_0.85.6_all.deb libsbuild-perl_0.85.6_all.deb

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
pkg-builder package examples/bookworm/virtual-package/pkg-builder.toml
```

This will build the package using the provided configuration file.

### Example rust package

```bash
cargo build 
cargo install . 
pkg-builder build-env create examples/bookworm/rust/hello-world/pkg-builder.toml
pkg-builder package examples/bookworm/rust/hello-world/pkg-builder.toml
```

### Example typescript package

```bash
cargo build 
cargo install . 
pkg-builder build-env create examples/bookworm/rust/hello-world/pkg-builder.toml
pkg-builder package examples/bookworm/rust/hello-world/pkg-builder.toml
```

### Example javascript package

```bash
cargo build 
cargo install . 
pkg-builder build-env create examples/bookworm/javascript/hello-world/pkg-builder.toml
pkg-builder package examples/bookworm/javascript/hello-world/pkg-builder.toml
```

### Example nim package

```bash
cargo build 
cargo install . 
pkg-builder build-env create examples/bookworm/nim/hello-world/pkg-builder.toml
pkg-builder package examples/bookworm/nim/hello-world/pkg-builder.toml
```

### Example dotnet package

```bash
cargo build 
cargo install . 
pkg-builder build-env create examples/bookworm/dotnet/hello-world/pkg-builder.toml
pkg-builder package examples/bookworm/dotnet/hello-world/pkg-builder.toml
```

### Example java package

```bash
cargo build 
cargo install . 
pkg-builder build-env create examples/bookworm/java/hello-world/pkg-builder.toml
pkg-builder package examples/bookworm/java/hello-world/pkg-builder.toml
```

### Piuparts only 

Assuming that you already packaged your source before as such 
```bash
cargo build 
cargo install . 
pkg-builder build-env create examples/bookworm/virtual-package/pkg-builder.toml
pkg-builder package examples/bookworm/virtual-package/pkg-builder.toml
```

you can run only piuparts 
```bash
pkg-builder piuparts examples/bookworm/virtual-package/pkg-builder.toml
```

### Autopkgtest only

Assuming that you already packaged your source before as such
```bash
cargo build 
cargo install . 
pkg-builder build-env create examples/bookworm/virtual-package/pkg-builder.toml
pkg-builder package examples/bookworm/virtual-package/pkg-builder.toml
```

you can run only piuparts
```bash
pkg-builder autopkgtests examples/bookworm/virtual-package/pkg-builder.toml
```