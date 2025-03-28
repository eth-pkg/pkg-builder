apt install -y wget
cd /tmp && wget -q -O node.tar.gz ${node_binary_url}
cd /tmp && echo "${node_binary_checksum} node.tar.gz" >> hash_file.txt && cat hash_file.txt
cd /tmp && sha256sum -c hash_file.txt
cd /tmp && rm -rf /usr/share/node && mkdir /usr/share/node && tar -C /usr/share/node -xzf node.tar.gz --strip-components=1
ls -l /usr/share/node/bin
ln -s /usr/share/node/bin/node /usr/bin/node
ln -s /usr/share/node/bin/npm /usr/bin/npm
ln -s /usr/share/node/bin/npx /usr/bin/npx
ln -s /usr/share/node/bin/corepack /usr/bin/corepack
apt remove -y wget
node --version
npm --version
