# Runtime: .NET on Ubuntu Noble (PPA backports)
SBUILD_FLAGS += --chroot-setup-commands='apt-get install software-properties-common -y'
SBUILD_FLAGS += --chroot-setup-commands='add-apt-repository ppa:dotnet/backports'
SBUILD_FLAGS += --chroot-setup-commands='apt-get update -y'
SBUILD_FLAGS += --chroot-setup-commands='apt install -y wget'

$(foreach n,$(RUNTIME_PKG_INDICES),\
$(eval SBUILD_FLAGS += --chroot-setup-commands='wget -q -O /tmp/$$(PKG$(n)_NAME).deb $$(PKG$(n)_URL)') \
$(eval SBUILD_FLAGS += --chroot-setup-commands='cd /tmp && apt install -y $$(PKG$(n)_APT_NAME)') \
$(eval SBUILD_FLAGS += --chroot-setup-commands='cd /tmp && apt download -y $$(PKG$(n)_APT_NAME)') \
$(eval SBUILD_FLAGS += --chroot-setup-commands='cd /tmp && ls && sha1sum $$(PKG$(n)_NAME).deb') \
$(eval SBUILD_FLAGS += --chroot-setup-commands='cd /tmp && echo $$(PKG$(n)_HASH) $$(PKG$(n)_NAME).deb >> hash_file.txt && cat hash_file.txt') \
$(eval SBUILD_FLAGS += --chroot-setup-commands='cd /tmp && sha1sum -c hash_file.txt') \
)

SBUILD_FLAGS += --chroot-setup-commands='dotnet --version'
SBUILD_FLAGS += --chroot-setup-commands='apt remove -y wget'
