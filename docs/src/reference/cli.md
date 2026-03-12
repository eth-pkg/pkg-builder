# CLI Reference

All commands accept an optional path to a `pkg-builder.toml` file. If omitted, pkg-builder looks for `pkg-builder.toml` in the current directory.

## `pkg-builder package`

Build a Debian package.

```bash
pkg-builder package [CONFIG] [OPTIONS]
```

**Options:**

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--run-lintian` | bool | from config | Override: run lintian after build |
| `--run-piuparts` | flag | `false` | Run piuparts after build |
| `--run-autopkgtest` | flag | `false` | Run autopkgtest after build |

This is the main command. It generates the Makefile, prepares the source, runs debcrafter, builds via sbuild, and optionally runs tests.

## `pkg-builder env create`

Create the sbuild chroot environment for the target distribution.

```bash
pkg-builder env create [CONFIG]
```

This must be run once per distribution before the first build. The chroot is reused for subsequent builds.

## `pkg-builder env clean`

Remove the sbuild chroot environment.

```bash
pkg-builder env clean [CONFIG]
```

## `pkg-builder generate`

Generate the Makefile without building.

```bash
pkg-builder generate [CONFIG]
```

Useful for inspecting the generated build steps or debugging the template expansion.

## `pkg-builder lintian`

Run lintian checks on an already-built package.

```bash
pkg-builder lintian [CONFIG]
```

## `pkg-builder piuparts`

Run piuparts install/remove tests on an already-built package.

```bash
pkg-builder piuparts [CONFIG]
```

## `pkg-builder autopkgtest`

Run autopkgtest functional tests on an already-built package.

```bash
pkg-builder autopkgtest [CONFIG]
```

## `pkg-builder verify`

Verify package file hashes against the expected values in `[verify].package_hash`.

```bash
pkg-builder verify [CONFIG]
```

## `pkg-builder clean`

Clean build artifacts from the working directory.

```bash
pkg-builder clean [CONFIG]
```

## `pkg-builder version`

Print the pkg-builder version.

```bash
pkg-builder version
```
