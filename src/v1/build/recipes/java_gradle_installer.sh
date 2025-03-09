apt install -y wget unzip
mkdir -p /opt/lib/gradle-${gradle_version}
cd /tmp && wget -q --output-document gradle.tar.gz ${gradle_binary_url}
cd /tmp && echo \"${gradle_binary_checksum} gradle.tar.gz\" > hash_file.txt && cat hash_file.txt
cd /tmp && sha256sum -c hash_file.txt
cd /tmp && unzip gradle.tar.gz && mv gradle-${gradle_version} /opt/lib
ln -s /opt/lib/gradle-${gradle_version}/bin/gradle /usr/bin/gradle
gradle -version
apt remove -y wget