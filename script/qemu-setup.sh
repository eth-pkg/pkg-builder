#!/usr/bin/env bash

debian_image="debian-12.5.0-amd64-netinst.iso"
sbuild_host="sbuild"
username=$(whoami)
pkgbuilder_dir="/home/$username/.pkg-builder/qemu"
username=$(whoami)
qemu_img="debian12.img"

print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  -f, --shared-folder   Specify the shared folder path on the host system (default: /home/username/workspace/pkg-builder)"
    echo "  -r, --recreate-env    Recreate the environment if the virtual machine already exists"
    echo "  -s, --disk-size       Specify the size of the disk in GB (default: 20)"
    echo "  -m, --memory-size     Specify the memory size in MB (default: 2048)"
    echo "  -h, --help            Display this help message"
}

while [[ "$#" -gt 0 ]]; do
    case $1 in
        -f|--shared-folder) shared_folder="$2"; shift ;;
        -r|--recreate-env) recreate_env=true ;;
        -s|--disk-size) qemu_disk_size="$2"; shift ;;
        -m|--memory-size) memory_size="$2"; shift ;;
        -h|--help) print_usage; exit 0 ;;
        *) echo "Error: Unknown option $1"; print_usage; exit 1 ;;
    esac
    shift
done

shared_folder="${shared_folder:-/home/$(whoami)/workspace/pkg-builder}"
recreate_env="${recreate_env:-false}"
qemu_disk_size="${qemu_disk_size:-20}"
memory_size="${memory_size:-2048}"

echo "Shared folder: $shared_folder"
echo "Recreate environment: $recreate_env"
echo "QEMU disk size: $qemu_disk_size GB"
echo "Memory size: $memory_size MB"


echo_error() {
    local message="$1"
    local RED='\033[0;31m'
    local NC='\033[0m' # No Color
    echo -e "${RED}Error: ${message}${NC}"
}

echo_success() {
    local message="$1"
    local GREEN='\033[0;32m'
    local NC='\033[0m' # No Color
    echo -e "${GREEN}${message}${NC}"
}

qemu_setup() {
    local qemu_system="qemu-system-x86_64"
    
    if command -v "$qemu_system" &> /dev/null; then
        echo "$qemu_system is already installed."
    else
        echo "$qemu_system is not installed, installing it now"
        sudo dnf install "$qemu_system"
    fi
}

download_debian_image() {
    local debian_url="https://cdimage.debian.org/debian-cd/current/amd64/iso-cd/"
    local signature_file="SHA512SUMS.sign"
    local sha512sums_file="SHA512SUMS"
    local key_id="DF9B9C49EAA9298432589D76DA87E80D6294BE9B"
    
    
    
    for file in "$debian_image" "$signature_file" "$sha512sums_file"; do
        if [ -e "$pkgbuilder_dir/$file" ]; then
            echo "$file already exists, not downloading it again"
        else
            wget "${debian_url}$file" -P $pkgbuilder_dir || {
                echo "Download failed. Exiting with status 1."
                exit 1
            }
        fi
    done
    
    if gpg --list-keys "$key_id" &> /dev/null; then
        echo "Key already exists in the keyring."
    else
        gpg --keyserver hkp://keys.gnupg.net --recv-keys "$key_id"
    fi
    
    gpg --verify "$pkgbuilder_dir/$signature_file" "$pkgbuilder_dir/$sha512sums_file"
    
    local calculated_checksum=$(sha512sum "$pkgbuilder_dir/$debian_image" | awk '{print $1}')
    local expected_checksum=$(grep "$debian_image" "$pkgbuilder_dir/$sha512sums_file" | awk '{print $1}')
    
    [ "$calculated_checksum" == "$expected_checksum" ] && \
    echo "Checksums match. The Debian image is valid." || \
    echo "Checksums do not match. The Debian image may be corrupted."
}

setup_sbuild_env() {
    local original_size=$(stty -g)
    
    if [ -e "$pkgbuilder_dir/$qemu_img" ]; then
        echo "Virtual machine image already exists."
        if $recreate_env; then
            # Destroy VM
            virsh destroy "$sbuild_host" > /dev/null 2>&1
            virsh managedsave-remove "$sbuild_host" > /dev/null 2>&1
            virsh undefine "$sbuild_host" > /dev/null 2>&1
            echo "Environment recreated."
        else
            echo "Skipping recreation of environment."
        fi
    else
        echo "Virtual machine image does not exist. Creating new image..."
        # Create new disk image
        qemu-img create -f qcow2 "$pkgbuilder_dir/$qemu_img" "${qemu_disk_size}G"
    fi
    # Check if the virtual machine already exists
    if virsh dominfo "$sbuild_host" > /dev/null 2>&1; then
        echo_info "Virtual machine '$sbuild_host' already exists."
        return 
    fi
    
    script_dir=$(dirname "$(readlink -f "$0")")
    echo "The current working directory is: $script_dir"
    
    if ! virt-install \
    --name="$sbuild_host" \
    --memory=$memory_size \
    --vcpus=2 \
    --disk size="$qemu_disk_size",path="$pkgbuilder_dir/$qemu_img",bus=virtio,format=qcow2 \
    --network user,model=virtio \
    --graphics=none \
    --location="$pkgbuilder_dir/$debian_image" \
    --os-variant=debian12 \
    --console pty,target_type=serial \
    --initrd-inject $script_dir/preseed.cfg \
    --extra-args="ks=file:/preseed.cfg console=tty0 console=ttyS0,115200n8 serial"; then
        echo_error "virt-install failed. Exiting with status 1."
        exit 1
    fi
    
    stty "$original_size"
}


print_login_instructions() {
    cat <<EOF
After installation, log in to the machine:
  - Start the virtual machine: virsh start "$sbuild_host"
  - Access the console: virsh console "$sbuild_host"
  - Log in using the following credentials:
    - Username: debian
    - Password: debian
EOF
}

mkdir -p "$pkgbuilder_dir"
qemu_setup
download_debian_image
setup_sbuild_env
print_login_instructions