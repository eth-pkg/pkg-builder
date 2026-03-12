# Configuration Reference

A `pkg-builder.toml` file defines everything needed to build a Debian package. This page documents every section and field.

## `[package]`

Package metadata.

```toml
[package]
name = "hello-world-c"
version = "1.0.0"
revision = "1"
homepage = "https://github.com/eth-pkg/pkg-builder#examples"
spec = "hello-world-c.sss"
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | yes | Debian package name |
| `version` | string | yes | Upstream version number |
| `revision` | string | yes | Debian revision number |
| `homepage` | string | yes | Project homepage URL |
| `spec` | path | yes | Path to the debcrafter `.sss` spec file |

## `[source]`

Source code location. The `type` field determines which other fields are available.

### `type = "tarball"`

```toml
[source]
type = "tarball"
url = "https://example.com/project-1.0.0.tar.gz"
hash = "c93bdd829eca65af1e..."
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | yes | Must be `"tarball"` |
| `url` | string | yes | URL or local path to the source tarball |
| `hash` | string | no | SHA-512 hash of the tarball |

### `type = "git"`

```toml
[source]
type = "git"
url = "https://github.com/org/repo.git"
tag = "v1.0.0"
submodules = [
    { commit = "72523ee3f865...", path = "vendor/lib1" },
]
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | yes | Must be `"git"` |
| `url` | string | yes | Git repository URL |
| `tag` | string | yes | Git tag to check out |
| `submodules` | array | no | List of pinned submodules |

Each submodule entry:

| Field | Type | Description |
|-------|------|-------------|
| `commit` | string | Expected commit hash |
| `path` | string | Submodule path within the repo |

### `type = "virtual"`

```toml
[source]
type = "virtual"
```

No additional fields. Used for meta-packages with no source code.

## `[build]`

Build environment configuration.

```toml
[build]
distribution = "bookworm"
arch = "amd64"
workdir = "~/.pkg-builder/packages/bookworm"
chroot_dir = "/srv/chroot"
snapshot_date = "20250101"
snapshot_security_date = "20250115T000000Z"
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `distribution` | string | yes | Target distribution: `"bookworm"`, `"trixie"`, or `"noble numbat"` |
| `arch` | string | yes | Target architecture (currently only `"amd64"`) |
| `workdir` | path | no | Working directory for build artifacts |
| `chroot_dir` | path | no | Path to the chroot directory |
| `snapshot_date` | string | no | Debian archive snapshot date (`YYYYMMDD`) |
| `snapshot_security_date` | string | no | Security archive snapshot date (`YYYYMMDDTHHMMSSZ`) |

Snapshot fields are only available for Debian distributions. See [Snapshot Pinning](../guides/snapshot-pinning.md).

## `[runtime]`

Runtime toolchain configuration. Required for languages that need external compilers or runtimes.

```toml
[runtime]
recipe = "rust"
binary_url = "https://static.rust-lang.org/dist/rust-1.77.2-x86_64-unknown-linux-gnu.tar.xz"
binary_gpg_asc = "..."
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `recipe` | string | yes | Recipe name: `c`, `rust`, `go`, `node`, `java`, `java-gradle`, `nim`, `dotnet-noble`, `dotnet-debian`, `dotnet-backup` |

All other fields depend on the recipe. The recipe template uses `{{field_name}}` placeholders that are filled from the remaining fields via a flat key-value mapping. See [Runtime Recipes](../guides/runtime-recipes.md) for the fields required by each recipe.

## `[testing]`

Test configuration. All fields default to `false` if this section is omitted.

```toml
[testing]
lintian = true
piuparts = true
autopkgtest = true
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `lintian` | bool | `false` | Run lintian static analysis |
| `piuparts` | bool | `false` | Run piuparts install/remove tests |
| `autopkgtest` | bool | `false` | Run autopkgtest functional tests |

See [Testing](../guides/testing.md) for details on each tool.

## `[tools]`

Pinned versions of external tools. Used for reproducibility — recording which tool versions produced the build.

```toml
[tools]
pkg_builder = "0.3.1"
debcrafter = "8189263"
sbuild = "0.85.6"
lintian = "2.116.3"
piuparts = "1.1.7"
autopkgtest = "5.28"
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `pkg_builder` | string | yes | pkg-builder version |
| `debcrafter` | string | yes | debcrafter version or commit hash |
| `sbuild` | string | yes | sbuild version |
| `lintian` | string | yes | lintian version |
| `piuparts` | string | yes | piuparts version |
| `autopkgtest` | string | yes | autopkgtest version |

## `[verify]`

Package hash verification. Optional — omit this section if you don't need reproducibility checking.

```toml
[verify]
package_hash = [
    { name = "hello-world-c_1.0.0-1.dsc", hash = "0843091f9e3cff991450c9687935e67644f6b032" },
    { name = "hello-world-c_1.0.0-1_amd64.deb", hash = "acd656c4554f41eb2a0ad570ef99ff19db51273d" },
]
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `package_hash` | array | yes | List of expected output file hashes |

Each entry:

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Expected output filename |
| `hash` | string | Expected SHA-1 hash |

See [Verification](../guides/verification.md) for usage.

## Complete example

```toml
[package]
name = "hello-world-go"
version = "1.0.0"
revision = "1"
homepage = "https://github.com/eth-pkg/pkg-builder#examples"
spec = "hello-world-go.sss"

[source]
type = "tarball"
url = "hello-world-go-1.0.0.tar.gz"
hash = "c268da86e5489a61491313aac237baf895cf269da477cbe9dc8bf4afdf0847a5..."

[build]
distribution = "noble numbat"
arch = "amd64"
workdir = "~/.pkg-builder/packages/noble"

[runtime]
recipe = "go"
binary_url = "https://go.dev/dl/go1.22.2.linux-amd64.tar.gz"
binary_checksum = "5901c52b7a78002aeff14a21f93e0f064f74ce1360fce51c6ee68cd471216a17"

[testing]
lintian = true
piuparts = true
autopkgtest = true

[tools]
pkg_builder = "0.3.1"
debcrafter = "8189263"
sbuild = "0.85.6"
lintian = "2.116.3"
piuparts = "1.1.7"
autopkgtest = "5.20"

[verify]
package_hash = [
    { name = "hello-world-go_1.0.0-1.dsc", hash = "af497430bbe28a04b0aba2b0cee0125894cdbbb5" },
    { name = "hello-world-go_1.0.0-1_amd64.deb", hash = "fb84917f1065d70cacb25bc41b9a13606710b3de" },
]
```
