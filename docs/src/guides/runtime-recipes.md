# Runtime Recipes

Languages that need external toolchains (compilers, runtimes, package managers) use the `[runtime]` section to install them into the sbuild chroot before the build runs.

## How it works

When you set `[runtime].recipe`, pkg-builder looks up the corresponding recipe template in `runtimes/<recipe>.recipe`. The recipe contains commands to download, verify, and install the toolchain. Template variables like `{{binary_url}}` are filled in from the other fields in `[runtime]`.

## Available recipes

### C

No `[runtime]` section needed — the C toolchain (gcc, make, etc.) is already available in the base sbuild chroot.

### Rust

```toml
[runtime]
recipe = "rust"
binary_url = "https://static.rust-lang.org/dist/rust-1.77.2-x86_64-unknown-linux-gnu.tar.xz"
binary_gpg_asc = """-----BEGIN PGP SIGNATURE-----
...
-----END PGP SIGNATURE-----
"""
```

Downloads the official Rust release, verifies the GPG signature against the Rust project's key, and installs rustc, cargo, and the standard library.

| Field | Description |
|-------|-------------|
| `binary_url` | URL to the Rust release tarball |
| `binary_gpg_asc` | Detached GPG signature for the tarball |

### Go

```toml
[runtime]
recipe = "go"
binary_url = "https://go.dev/dl/go1.22.2.linux-amd64.tar.gz"
binary_checksum = "5901c52b7a78002aeff14a21f93e0f064f74ce1360fce51c6ee68cd471216a17"
```

Downloads the official Go release, verifies the SHA-256 checksum, and installs to `/usr/local/go`.

| Field | Description |
|-------|-------------|
| `binary_url` | URL to the Go release tarball |
| `binary_checksum` | SHA-256 checksum of the tarball |

### Node.js (JavaScript/TypeScript)

```toml
[runtime]
recipe = "node"
binary_url = "https://nodejs.org/download/release/v20.12.2/node-v20.12.2-linux-x64.tar.gz"
binary_checksum = "f8f9b6877778ed2d5f920a5bd853f0f8a8be1c42f6d448c763a95625cbbb4b0d"
```

Downloads Node.js, verifies the checksum, and installs node, npm, npx, and corepack. Use the same recipe for both JavaScript and TypeScript projects.

| Field | Description |
|-------|-------------|
| `binary_url` | URL to the Node.js release tarball |
| `binary_checksum` | SHA-256 checksum of the tarball |

### Java

```toml
[runtime]
recipe = "java"
binary_url = "https://download.oracle.com/java/21/archive/jdk-21.0.2_linux-x64_bin.tar.gz"
binary_checksum = "..."
jdk_version = "21.0.2"
```

Downloads a JDK, verifies the checksum, and installs java and javac.

| Field | Description |
|-------|-------------|
| `binary_url` | URL to the JDK tarball |
| `binary_checksum` | SHA-256 checksum of the tarball |
| `jdk_version` | JDK version (used for install paths) |

### Java with Gradle

```toml
[runtime]
recipe = "java-gradle"
binary_url = "https://download.oracle.com/java/21/archive/jdk-21.0.2_linux-x64_bin.tar.gz"
binary_checksum = "..."
jdk_version = "21.0.2"
gradle_version = "8.7"
gradle_binary_url = "https://services.gradle.org/distributions/gradle-8.7-bin.zip"
gradle_binary_checksum = "..."
```

Same as the Java recipe, but also installs Gradle.

| Field | Description |
|-------|-------------|
| `binary_url` | URL to the JDK tarball |
| `binary_checksum` | SHA-256 checksum of the JDK tarball |
| `jdk_version` | JDK version (used for install paths) |
| `gradle_version` | Gradle version (used for install paths) |
| `gradle_binary_url` | URL to the Gradle distribution zip |
| `gradle_binary_checksum` | SHA-256 checksum of the Gradle zip |

### Nim

```toml
[runtime]
recipe = "nim"
binary_url = "https://nim-lang.org/download/nim-2.0.2-linux_x64.tar.xz"
binary_checksum = "..."
nim_version = "2.0.2"
```

Downloads a Nim release, verifies the checksum, and installs the nim compiler.

| Field | Description |
|-------|-------------|
| `binary_url` | URL to the Nim release tarball |
| `binary_checksum` | SHA-256 checksum of the tarball |
| `nim_version` | Nim version (used for install paths) |

### .NET (Ubuntu Noble)

```toml
[runtime]
recipe = "dotnet-noble"
packages = [
    { name = "dotnet-sdk-8.0", apt_name = "dotnet-sdk-8.0", url = "...", hash = "..." },
]
```

Installs .NET from the Ubuntu PPA backports repository. Each package is downloaded, installed, and verified by SHA-1 hash.

### .NET (Debian)

```toml
[runtime]
recipe = "dotnet-debian"
packages = [
    { name = "dotnet-sdk-8.0", apt_name = "dotnet-sdk-8.0", url = "...", hash = "..." },
]
```

Installs .NET from the Microsoft Debian repository.

### .NET (backup/direct)

```toml
[runtime]
recipe = "dotnet-backup"
extra_deps = [
    { name = "libicu-dev" },
]
packages = [
    { name = "dotnet-sdk-8.0", url = "...", hash = "..." },
]
```

Installs .NET by directly downloading and installing `.deb` files. Used as a fallback when repository-based installation doesn't work.

See also the [Recipes Reference](../reference/recipes.md) for a quick-reference table of all recipes and their fields.
