# Runtime: Go
my $binary_url = '{{binary_url}}';
my $binary_checksum = '{{binary_checksum}}';

my @runtime_commands = (
    'apt install -y wget',
    "wget -q -O /tmp/go.tar.gz $binary_url",
    "echo \"$binary_checksum  /tmp/go.tar.gz\" > /tmp/hash_file.txt && sha256sum -c /tmp/hash_file.txt",
    'rm -rf /usr/local/go && tar -C /usr/local -xzf /tmp/go.tar.gz && rm -f /tmp/go.tar.gz /tmp/hash_file.txt',
    'ln -s /usr/local/go/bin/go /usr/bin/go',
    'chmod -R a+rwx /usr/local/go/pkg',
    'apt remove -y wget',
    'go version',
);
