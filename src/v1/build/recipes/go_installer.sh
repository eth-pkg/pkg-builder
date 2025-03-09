apt install -y wget
cd /tmp && wget -q -O go.tar.gz ${go_binary_url}
cd /tmp && echo \"${go_binary_checksum} go.tar.gz\" >> hash_file.txt && cat hash_file.txt
cd /tmp && sha256sum -c hash_file.txt
cd /tmp && rm -rf /usr/local/go && mkdir /usr/local/go && tar -C /usr/local -xzf go.tar.gz
ln -s /usr/local/go/bin/go /usr/bin/go
go version
chmod -R a+rwx /usr/local/go/pkg
apt remove -y wget
