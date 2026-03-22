# Runtime: Java (JDK only)
my $binary_url = '{{binary_url}}';
my $binary_checksum = '{{binary_checksum}}';
my $jdk_version = '{{jdk_version}}';

my @runtime_commands = (
    'apt install -y wget',
    "wget -q -O /tmp/jdk.tar.gz $binary_url",
    "echo \"$binary_checksum  /tmp/jdk.tar.gz\" > /tmp/hash_file.txt && sha256sum -c /tmp/hash_file.txt",
    "mkdir -p /opt/lib/jvm/jdk-${jdk_version}-oracle",
    "tar -zxf /tmp/jdk.tar.gz -C /opt/lib/jvm/jdk-${jdk_version}-oracle --strip-components=1 && rm -f /tmp/jdk.tar.gz /tmp/hash_file.txt",
    "ln -s /opt/lib/jvm/jdk-${jdk_version}-oracle/bin/java /usr/bin/java",
    "ln -s /opt/lib/jvm/jdk-${jdk_version}-oracle/bin/javac /usr/bin/javac",
    'apt remove -y wget',
    'java -version',
);
