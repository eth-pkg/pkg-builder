# Runtime: .NET on Ubuntu Noble (PPA backports)
my @packages = (
{{packages_perl}}
);

my @runtime_commands = (
    'apt-get install -y software-properties-common',
    'add-apt-repository ppa:dotnet/backports',
    'apt-get update -y',
    'apt install -y wget',
    (map {
        my $p = $_;
        (
            "wget -q -O /tmp/$$p{name}.deb $$p{url}",
            "apt install -y $$p{apt_name}",
            "echo \"$$p{hash}  /tmp/$$p{name}.deb\" > /tmp/hash_file.txt && sha1sum -c /tmp/hash_file.txt",
            "rm -f /tmp/$$p{name}.deb /tmp/hash_file.txt",
        )
    } @packages),
    'apt remove -y wget',
    'dotnet --version',
);
