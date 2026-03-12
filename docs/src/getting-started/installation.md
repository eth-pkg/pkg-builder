# Installation

## Prerequisites

Install the required Debian packages:

```bash
sudo apt install libssl-dev pkg-config quilt debhelper tar wget autopkgtest \
                 vmdb2 qemu-system-x86 git-lfs uidmap
```

Add your user to the sbuild group:

```bash
sudo sbuild-adduser $(whoami)
```

## Installing sbuild

pkg-builder requires a patched version of sbuild from the eth-pkg fork:

```bash
# Clone the fork
git clone https://github.com/eth-pkg/sbuild.git
cd sbuild

# Install build dependencies
sudo apt-get install -y dh-python dh-sequence-python3 libyaml-tiny-perl python3-all genisoimage

# Build and install
dpkg-buildpackage -us -uc
cd .. && sudo dpkg -i sbuild_0.85.6_all.deb libsbuild-perl_0.85.6_all.deb
```

## Setting up chroot

Create the chroot directory used by sbuild:

```bash
sudo mkdir -p /srv/chroot
sudo chown :sbuild /srv/chroot
```

For Ubuntu noble builds, add the debootstrap symlink:

```bash
sudo ln -s /usr/share/debootstrap/scripts/gutsy /usr/share/debootstrap/scripts/noble
```

## Building pkg-builder

```bash
git clone https://github.com/eth-pkg/pkg-builder.git
cd pkg-builder
cargo install --path .
```

## Ubuntu-specific setup

If building for Ubuntu on a Debian host, manually install the Ubuntu archive keyring:

1. Download [ubuntu-archive-keyring.gpg](https://salsa.debian.org/debian/ubuntu-keyring/-/raw/master/keyrings/ubuntu-archive-keyring.gpg?ref_type=heads)
2. Copy it to `/usr/share/keyrings/`
