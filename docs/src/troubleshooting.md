# Troubleshooting

## Lintian errors

### `build-depends-on-build-essential`

```text
E: <package> source: build-depends-on-build-essential Build-Depends
```

Remove `build-essential` from your dependencies. The `build-essential` packages are already present in the minimal chroot environment provided by sbuild.

### Copyright file issues

Lintian may report missing or incorrect copyright information. The copyright file must account for all licenses found in the source.

To generate a starting point, check out the source and run:

```bash
debmake -c > debian/copyright
```

A typical `debian/copyright` file:

```text
Files: *
Copyright: 2022 ORIGINAL PACKAGE AUTHORS
License: GPL-3+

Files: debian/*
Copyright: 2022 MAINTAINER_NAME
License: GPL-3+

License: GPL-3+
 The full text of the GPL version 3 is distributed in
 /usr/share/common-licenses/GPL-3 on Debian systems.
```

Some source files may have different licenses in their headers. Compare the generated output against the existing copyright file to find discrepancies.

### Suppressing lintian warnings

For warnings that don't apply to your package, add overrides:

- **Binary package warnings**: `src/debian/<package-name>.lintian-overrides`
- **Source package warnings**: `src/debian/source/lintian-overrides`

## Build environment issues

### Chroot not found

If `pkg-builder build` fails because the chroot doesn't exist, create it first:

```bash
pkg-builder --config path/to/pkg-builder.toml env create
```

### Stale chroot

If builds fail due to stale packages in the chroot, clean and recreate it:

```bash
pkg-builder --config path/to/pkg-builder.toml env clean
pkg-builder --config path/to/pkg-builder.toml env create
```

## Hash verification failures

If `pkg-builder verify` reports mismatched hashes:

1. **Check snapshot pinning** — without pinned snapshots, dependency versions may change between builds
2. **Check tool versions** — ensure the `[tools]` section matches the versions actually installed
3. **Check architecture** — hashes are architecture-specific
4. **Rebuild from clean** — run `pkg-builder clean` then rebuild with `pkg-builder build`

See [Verification](guides/verification.md) for more on setting up reproducible builds.
