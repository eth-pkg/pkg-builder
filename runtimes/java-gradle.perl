# Runtime: Java with Gradle
my $binary_url = '{{binary_url}}';
my $binary_checksum = '{{binary_checksum}}';
my $jdk_version = '{{jdk_version}}';
my $gradle_binary_url = '{{gradle_binary_url}}';
my $gradle_binary_checksum = '{{gradle_binary_checksum}}';
my $gradle_version = '{{gradle_version}}';

my @runtime_commands = (
    'apt install -y wget unzip',
    # JDK
    "wget -q -O /tmp/jdk.tar.gz $binary_url",
    "echo \"$binary_checksum  /tmp/jdk.tar.gz\" > /tmp/hash_file.txt && sha256sum -c /tmp/hash_file.txt",
    "mkdir -p /opt/lib/jvm/jdk-${jdk_version}-oracle",
    "tar -zxf /tmp/jdk.tar.gz -C /opt/lib/jvm/jdk-${jdk_version}-oracle --strip-components=1 && rm -f /tmp/jdk.tar.gz /tmp/hash_file.txt",
    "ln -s /opt/lib/jvm/jdk-${jdk_version}-oracle/bin/java /usr/bin/java",
    "ln -s /opt/lib/jvm/jdk-${jdk_version}-oracle/bin/javac /usr/bin/javac",
    # Gradle
    "wget -q -O /tmp/gradle.zip $gradle_binary_url",
    "echo \"$gradle_binary_checksum  /tmp/gradle.zip\" > /tmp/hash_file.txt && sha256sum -c /tmp/hash_file.txt",
    "cd /tmp && unzip -q gradle.zip && mv gradle-${gradle_version} /opt/lib/ && rm -f /tmp/gradle.zip /tmp/hash_file.txt",
    "ln -s /opt/lib/gradle-${gradle_version}/bin/gradle /usr/bin/gradle",
    'apt remove -y wget unzip',
    'java -version',
    'gradle -version',
);
