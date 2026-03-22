# CLI Reference

## Global options

All commands accept global options before the subcommand:

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--config <PATH>` | string | `.` (current directory) | Path to `pkg-builder.toml` |
| `--resume` | flag | `false` | Skip build steps whose output artifacts already exist |
| `-V`, `--version` | flag | | Print version and exit |
| `-h`, `--help` | flag | | Print help |

## `pkg-builder init`

Interactive project setup wizard. When run without flags, prompts for each value interactively. Any flag provided skips the corresponding prompt.

```bash
pkg-builder init [OPTIONS]
```

**Options:**

| Flag | Type | Description |
|------|------|-------------|
| `--name` | string | Package name |
| `--version` | string | Package version |
| `--revision` | string | Package revision |
| `--homepage` | string | Homepage URL |
| `--maintainer-name` | string | Maintainer name |
| `--maintainer-email` | string | Maintainer email |
| `--section` | string | Debian section (e.g., `net`, `utils`, `devel`, `admin`, `libs`, `web`) |
| `--summary` | string | Short package description |
| `--source-type` | string | Source type: `tarball`, `git`, or `virtual` |
| `--url` | string | Source URL (tarball URL or git repo URL) |
| `--tag` | string | Git tag (for git source type) |
| `--distribution` | string | Target distribution: `bookworm`, `trixie`, or `noble` |
| `--arch` | string | Target architecture |
| `--runtime` | string | Runtime recipe: `go`, `rust`, `node`, `java`, `java-gradle`, `nim`, `c`, or `none` |
| `--output` | string | Output directory (default: current directory) |
| `--upstream-hash` | string | Expected upstream hash for tarball verification |

**Example (fully non-interactive):**

```bash
pkg-builder init \
  --name my-package \
  --version 1.0.0 \
  --revision 1 \
  --homepage https://example.com \
  --maintainer-name "Jane Doe" \
  --maintainer-email jane@example.com \
  --section net \
  --summary "My package description" \
  --source-type tarball \
  --url https://example.com/my-package-1.0.0.tar.gz \
  --distribution bookworm \
  --arch amd64 \
  --runtime none \
  --output ./my-package
```

## `pkg-builder build`

Build a Debian package. Runs a 4-step pipeline: acquire source, extract, generate debian packaging (via debcrafter), and invoke sbuild.

```bash
pkg-builder build
```

By default, `build` cleans the output directory and runs from scratch. Use `--resume` to skip steps whose output artifacts already exist (e.g., skip downloading if the tarball is present).

## `pkg-builder env create`

Create the sbuild chroot environment for the target distribution.

```bash
pkg-builder env create
```

This must be run once per distribution before the first build. The chroot is reused for subsequent builds.

## `pkg-builder env clean`

Remove the sbuild chroot environment (chroot tarball only).

```bash
pkg-builder env clean
```

## `pkg-builder verify`

Verify package file hashes against the expected values in `[verify].package_hash`.

```bash
pkg-builder verify
```

## `pkg-builder clean`

Clean build artifacts from the working directory.

```bash
pkg-builder clean
```
