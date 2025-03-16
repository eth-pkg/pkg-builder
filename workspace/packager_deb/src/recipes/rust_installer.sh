apt install -y wget gpg gpg-agent
cd /tmp && wget -q -O package.tar.xz ${rust_binary_url}
cd /tmp && echo "${rust_binary_gpg_asc}" >> package.tar.xz.asc
wget -qO- https://keybase.io/rust/pgp_keys.asc | gpg --import
cd /tmp && gpg --verify package.tar.xz.asc package.tar.xz
cd /tmp && tar xvJf package.tar.xz -C . --strip-components=1
cd /tmp && /bin/bash install.sh
apt remove -y wget gpg gpg-agent