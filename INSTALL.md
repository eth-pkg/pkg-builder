# Installation Guide

## Debian Prerequisites

```bash
sudo apt install libssl-dev pkg-config quilt debhelper tar wget autopkgtest vmdb2 qemu-system-x86 git-lfs uidmap
sudo sbuild-adduser `whoami`
```

## Installing sbuild

```bash
# Clone repository
git clone https://github.com/eth-pkg/sbuild.git 
cd sbuild  

# Install dependencies
sudo apt-get install -y dh-python dh-sequence-python3 libyaml-tiny-perl python3-all genisoimage

# Build the package
dpkg-buildpackage -us -uc 

# Install the newly built package 
cd .. && sudo dpkg -i sbuild_0.85.6_all.deb libsbuild-perl_0.85.6_all.deb
```

## Setting up chroot

```bash
# Create chroot directory if it doesn't exist
sudo mkdir /srv/chroot 
sudo chown :sbuild /srv/chroot

# For noble builds
sudo ln -s /usr/share/debootstrap/scripts/gutsy /usr/share/debootstrap/scripts/noble
```

## Ubuntu-specific Setup

If building for Ubuntu on Bookworm, manually download the ubuntu-archive-keyring:
1. Get the keyring from [ubuntu-archive-keyring](https://salsa.debian.org/debian/ubuntu-keyring/-/raw/master/keyrings/ubuntu-archive-keyring.gpg?ref_type=heads)
2. Copy it into `/usr/share/keyrings`