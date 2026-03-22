# Runtime: .NET backup version (direct .deb install)
SBUILD_FLAGS += --chroot-setup-commands='apt install -y wget'
SBUILD_FLAGS += --chroot-setup-commands='apt install -y libicu-dev'

$(foreach n,$(RUNTIME_PKG_INDICES),\
$(eval SBUILD_FLAGS += --chroot-setup-commands='cd /tmp && wget -q $$(PKG$(n)_URL)') \
$(eval SBUILD_FLAGS += --chroot-setup-commands='cd /tmp && ls && dpkg -i $$(PKG$(n)_NAME).deb') \
$(eval SBUILD_FLAGS += --chroot-setup-commands='cd /tmp && ls && sha1sum $$(PKG$(n)_NAME).deb') \
$(eval SBUILD_FLAGS += --chroot-setup-commands='cd /tmp && echo $$(PKG$(n)_HASH) $$(PKG$(n)_NAME).deb > hash_file.txt && cat hash_file.txt') \
$(eval SBUILD_FLAGS += --chroot-setup-commands='cd /tmp && sha1sum -c hash_file.txt') \
)

SBUILD_FLAGS += --chroot-setup-commands='dotnet --version'
SBUILD_FLAGS += --chroot-setup-commands='apt remove -y wget'
