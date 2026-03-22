# Runtime: .NET on Debian (Microsoft repo)
my @packages = (
{{packages_perl}}
);

my @runtime_commands = (
    'apt install -y wget',
    'wget -q -O /tmp/packages-microsoft-prod.deb https://packages.microsoft.com/config/debian/12/packages-microsoft-prod.deb',
    'dpkg -i /tmp/packages-microsoft-prod.deb && rm -f /tmp/packages-microsoft-prod.deb',
    'apt update -y',
    (map {
        my $p = $_;
        (
            "wget -q -O /tmp/$$p{name}.deb $$p{url}",
            "apt install -y --allow-downgrades $$p{apt_name}",
            "echo \"$$p{hash}  /tmp/$$p{name}.deb\" > /tmp/hash_file.txt && sha1sum -c /tmp/hash_file.txt",
            "rm -f /tmp/$$p{name}.deb /tmp/hash_file.txt",
        )
    } @packages),
    'apt remove -y wget',
    'dotnet --version',
);
