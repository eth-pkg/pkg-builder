# Runtime: .NET backup version (direct .deb install)
my @packages = (
{{packages_perl}}
);

my @runtime_commands = (
    'apt install -y wget libicu-dev',
    (map {
        my $p = $_;
        (
            "wget -q -O /tmp/$$p{name}.deb $$p{url}",
            "dpkg -i /tmp/$$p{name}.deb",
            "echo \"$$p{hash}  /tmp/$$p{name}.deb\" > /tmp/hash_file.txt && sha1sum -c /tmp/hash_file.txt",
            "rm -f /tmp/$$p{name}.deb /tmp/hash_file.txt",
        )
    } @packages),
    'apt remove -y wget',
    'dotnet --version',
);
