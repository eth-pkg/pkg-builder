# Recipes Reference

Quick-reference table of all runtime recipes and the `[runtime]` fields they require.

## Recipe summary

| Recipe | Language | Verification | Key fields |
|--------|----------|-------------|------------|
| `c` | C | — | *(none — no `[runtime]` needed)* |
| `rust` | Rust | GPG signature | `binary_url`, `binary_gpg_asc` |
| `go` | Go | SHA-256 | `binary_url`, `binary_checksum` |
| `node` | JavaScript, TypeScript | SHA-256 | `binary_url`, `binary_checksum` |
| `java` | Java | SHA-256 | `binary_url`, `binary_checksum`, `jdk_version` |
| `java-gradle` | Java + Gradle | SHA-256 | `binary_url`, `binary_checksum`, `jdk_version`, `gradle_version`, `gradle_binary_url`, `gradle_binary_checksum` |
| `nim` | Nim | SHA-256 | `binary_url`, `binary_checksum`, `nim_version` |
| `dotnet-noble` | .NET (Ubuntu) | SHA-1 | `packages[]` |
| `dotnet-debian` | .NET (Debian) | SHA-1 | `packages[]` |
| `dotnet-backup` | .NET (direct) | SHA-1 | `packages[]`, `extra_deps[]`? |

Fields marked with `?` are optional.

## Field details

### Common fields

| Field | Type | Description |
|-------|------|-------------|
| `recipe` | string | Recipe name (required for all) |
| `binary_url` | string | Download URL for the toolchain archive |
| `binary_checksum` | string | SHA-256 hash of the archive |
| `binary_gpg_asc` | string | Detached GPG signature (Rust only) |

### Java fields

| Field | Type | Description |
|-------|------|-------------|
| `jdk_version` | string | JDK version, used for install directory naming |
| `gradle_version` | string | Gradle version (java-gradle only) |
| `gradle_binary_url` | string | Gradle distribution download URL |
| `gradle_binary_checksum` | string | SHA-256 hash of the Gradle distribution |

### Nim fields

| Field | Type | Description |
|-------|------|-------------|
| `nim_version` | string | Nim version, used for install directory naming |

### .NET package fields

The `dotnet-*` recipes use a `packages` array instead of `binary_url`:

```toml
[runtime]
recipe = "dotnet-noble"
packages = [
    { name = "dotnet-sdk-8.0", apt_name = "dotnet-sdk-8.0", url = "https://...", hash = "sha1..." },
]
```

| Field | Type | Description |
|-------|------|-------------|
| `packages[].name` | string | Package filename (without `.deb`) |
| `packages[].apt_name` | string | APT package name (for `dotnet-noble` and `dotnet-debian`) |
| `packages[].url` | string | Download URL for the `.deb` file |
| `packages[].hash` | string | SHA-1 hash of the `.deb` file |

The `dotnet-backup` recipe also supports:

| Field | Type | Description |
|-------|------|-------------|
| `extra_deps[].name` | string | Additional APT packages to install |

## Recipe template syntax

Recipe files in `runtimes/` use a simple template language:

- `{{field_name}}` — replaced with the value of `[runtime].field_name`
- `APT_INSTALL <packages>` — install packages via apt
- `APT_REMOVE <packages>` — remove packages via apt
- `DOWNLOAD <url> <dest>` — download a file
- `RUN <command>` — execute a shell command
- `SYMLINK <target> <link>` — create a symbolic link
- `REPEAT <var> IN {{list}}` / `END` — iterate over an array field
