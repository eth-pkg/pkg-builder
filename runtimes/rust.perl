# Runtime: Rust (GPG-verified)
my $binary_url = '{{binary_url}}';
my $binary_gpg_asc = '{{binary_gpg_asc}}';

my @runtime_commands = (
    'apt install -y wget gpg gpg-agent',
    "wget -q -O /tmp/rust.tar.xz $binary_url",
    "echo \"$binary_gpg_asc\" > /tmp/rust.tar.xz.asc",
    'wget -qO- https://keybase.io/rust/pgp_keys.asc | gpg --import',
    'gpg --verify /tmp/rust.tar.xz.asc /tmp/rust.tar.xz',
    'mkdir -p /tmp/rust-install && tar xJf /tmp/rust.tar.xz -C /tmp/rust-install --strip-components=1 && rm -f /tmp/rust.tar.xz /tmp/rust.tar.xz.asc',
    '/bin/bash /tmp/rust-install/install.sh --components=rustc,cargo,rust-std-x86_64-unknown-linux-gnu',
    'rm -rf /tmp/rust-install',
    'apt remove -y wget gpg gpg-agent',
    'rustc --version',
    'cargo --version',
);
