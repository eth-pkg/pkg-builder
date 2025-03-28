apt install -y wget
mkdir -p /opt/lib/jvm/jdk-${jdk_version}-oracle && mkdir -p /usr/lib/jvm
cd /tmp && wget -q --output-document jdk.tar.gz ${jdk_binary_url}
cd /tmp && echo "${jdk_binary_checksum} jdk.tar.gz" >>hash_file.txt && cat hash_file.txt
cd /tmp && sha256sum -c hash_file.txt
cd /tmp && tar -zxf jdk.tar.gz -C /opt/lib/jvm/jdk-${jdk_version}-oracle --strip-components=1
ln -s /opt/lib/jvm/jdk-${jdk_version}-oracle/bin/java /usr/bin/java
ln -s /opt/lib/jvm/jdk-${jdk_version}-oracle/bin/javac /usr/bin/javac
java -version
apt remove -y wget
