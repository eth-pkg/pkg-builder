# Pkg-builder

Pkg-builder simplifies the process of creating packages for Linux distributions. It automates the packaging process based on a configuration file, leveraging debcrafter, a framework for building Debian packages. By specifying package metadata, dependencies, and other configuration details in a structured format, developers can easily generate packages ready for Linux distributions. Pkg-builder abstracts away much of the complexity involved in packaging, allowing developers to focus on their software rather than packaging intricacies.

## Prerequisites

If you are using Debian, install sbuild, and various dependencies:

```bash
sudo apt install sbuild libssl-dev pkg-config quilt debhelper
sudo sbuild-adduser `whoami`

# if chroot not exists create it, TODO other cases 
sudo mkdir /srv/chroot 
sudo chown :sbuild /srv/chroot 
```

For non-Debian derived distributions, you'll need to either install VirtualBox or a similar tool to package for Debian-based distros. Alternatively, set up a QEMU virtual environment to build for Bookworm. Please note that sbuild is only available on Debian.

To set up QEMU virtual environment:

```bash
bash scripts/qemu-setup.sh # Install only for the first time
virsh start sbuild
virsh console sbuild # username: debian, password: debian 
```

## Getting Started

```bash
cargo build 
cargo install . 
pkg-builder package --config=examples/bookworm/virtual-package/pkg-builder.toml
```


This will build the package using the provided configuration file.


