apt install -y wget
rm -rf /tmp/nim-${nim_version} && rm -rf /usr/lib/nim/nim-${nim_version} && rm -rf /opt/lib/nim/nim-${nim_version} && mkdir /tmp/nim-${nim_version}
mkdir -p /opt/lib/nim && mkdir -p /usr/lib/nim
cd /tmp && wget -q ${nim_binary_url}
cd /tmp && echo ${nim_version_checksum} >> hash_file.txt && cat hash_file.txt
cd /tmp && sha256sum -c hash_file.txt
cd /tmp && tar xJf nim-${nim_version}-linux_x64.tar.xz -C nim-${nim_version} --strip-components=1
cd /tmp && mv nim-${nim_version} /opt/lib/ni
ln -s /opt/lib/nim/nim-${nim_version}/bin/nim /usr/bin/nim
nim --version
apt remove -y wget
