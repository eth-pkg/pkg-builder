# Runtime: Node.js
my $binary_url = '{{binary_url}}';
my $binary_checksum = '{{binary_checksum}}';

my @runtime_commands = (
    'apt install -y wget',
    "wget -q -O /tmp/node.tar.gz $binary_url",
    "echo \"$binary_checksum  /tmp/node.tar.gz\" > /tmp/hash_file.txt && sha256sum -c /tmp/hash_file.txt",
    'rm -rf /usr/share/node && mkdir -p /usr/share/node && tar -C /usr/share/node -xzf /tmp/node.tar.gz --strip-components=1 && rm -f /tmp/node.tar.gz /tmp/hash_file.txt',
    'ln -s /usr/share/node/bin/node /usr/bin/node',
    'ln -s /usr/share/node/bin/npm /usr/bin/npm',
    'ln -s /usr/share/node/bin/npx /usr/bin/npx',
    'ln -s /usr/share/node/bin/corepack /usr/bin/corepack',
    'apt remove -y wget',
    'node --version',
    'npm --version',
);
