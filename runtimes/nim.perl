# Runtime: Nim
# Note: binary_checksum includes the filename (e.g. "hash  nim-VER-linux_x64.tar.xz")
my $binary_url = '{{binary_url}}';
my $binary_checksum = '{{binary_checksum}}';
my $nim_version = '{{nim_version}}';

my $nim_archive = "nim-${nim_version}-linux_x64.tar.xz";

my @runtime_commands = (
    'apt install -y wget',
    "wget -q -O /tmp/$nim_archive $binary_url",
    "cd /tmp && echo \"$binary_checksum\" > hash_file.txt && sha256sum -c hash_file.txt",
    'mkdir -p /opt/lib/nim',
    "mkdir -p /tmp/nim-extract && tar xJf /tmp/$nim_archive -C /tmp/nim-extract --strip-components=1 && mv /tmp/nim-extract /opt/lib/nim/nim-${nim_version} && rm -f /tmp/$nim_archive /tmp/hash_file.txt",
    "ln -s /opt/lib/nim/nim-${nim_version}/bin/nim /usr/bin/nim",
    'apt remove -y wget',
    'nim --version',
);
