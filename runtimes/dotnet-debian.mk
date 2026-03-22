# Runtime: .NET on Debian (Microsoft repo)
# Note: Uses SHA-1 for package verification because Microsoft .deb packages
# are distributed via apt, which uses SHA-1 in its package metadata.
SBUILD_FLAGS += --chroot-setup-commands='apt install -y wget'
SBUILD_FLAGS += --chroot-setup-commands='cd /tmp && wget -q https://packages.microsoft.com/config/debian/12/packages-microsoft-prod.deb -O packages-microsoft-prod.deb'
SBUILD_FLAGS += --chroot-setup-commands='dpkg -i /tmp/packages-microsoft-prod.deb'
SBUILD_FLAGS += --chroot-setup-commands='apt update -y'

$(foreach n,$(RUNTIME_PKG_INDICES),\
$(eval SBUILD_FLAGS += --chroot-setup-commands='wget -q -O /tmp/$$(PKG$(n)_NAME).deb $$(PKG$(n)_URL)') \
$(eval SBUILD_FLAGS += --chroot-setup-commands='cd /tmp && apt install -y --allow-downgrades $$(PKG$(n)_APT_NAME)') \
$(eval SBUILD_FLAGS += --chroot-setup-commands='cd /tmp && apt download -y $$(PKG$(n)_APT_NAME)') \
$(eval SBUILD_FLAGS += --chroot-setup-commands='cd /tmp && ls && sha1sum $$(PKG$(n)_NAME).deb') \
$(eval SBUILD_FLAGS += --chroot-setup-commands='cd /tmp && echo $$(PKG$(n)_HASH) $$(PKG$(n)_NAME).deb >> hash_file.txt && cat hash_file.txt') \
$(eval SBUILD_FLAGS += --chroot-setup-commands='cd /tmp && sha1sum -c hash_file.txt') \
)

SBUILD_FLAGS += --chroot-setup-commands='dotnet --version'
SBUILD_FLAGS += --chroot-setup-commands='apt remove -y wget'
