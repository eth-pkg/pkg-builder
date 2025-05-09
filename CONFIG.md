# Configuration Reference

pkg-builder uses TOML configuration files to define package metadata, build environments, and dependencies.

## Basic Structure

```toml
[package_fields]
spec_file = "path/to/spec.sss"
package_name = "package-name"
version_number = "1.0.0"
revision_number = "1"
homepage = "https://example.com"

[package_type]
package_type = "default|git|virtual"
# Fields vary based on package_type

[build_env]
codename = "bookworm"
arch = "amd64"
pkg_builder_version = "0.3.1"
debcrafter_version = "8189263"
# Additional build environment options
```

## Package Fields

```toml
[package_fields]
spec_file = "hello-world.sss"           # Path to spec file
package_name = "hello-world"            # Package name
version_number = "1.0.0"                # Version
revision_number = "1"                   # Revision number
homepage = "https://github.com/..."     # Project homepage
```

## Package Types

### Virtual Package

```toml
[package_type]
package_type = "virtual"
```

### Default Package Type

```toml
[package_type]
package_type = "default"
tarball_url = "hello-world-1.0.0.tar.gz"
tarball_hash = "c93bdd829eca65af1e303d..."  # Optional checksum

[package_type.language_env]
language_env = "c|rust|go|javascript|typescript|java|dotnet|nim|python"
# Additional fields based on language_env
```

### Git Package Type

```toml
[package_type]
package_type = "git"
git_tag = "v1.0.0"
git_url = "https://github.com/user/repo.git"

[[package_type.submodules]]
commit = "abcdef123456"
path = "path/to/submodule"

[package_type.language_env]
language_env = "c|rust|go|javascript|typescript|java|dotnet|nim|python"
# Additional fields based on language_env
```

## Language Environments

### C (Default)

```toml
[package_type.language_env]
language_env = "c"
```

### Python

```toml
[package_type.language_env]
language_env = "python"
```

### Rust

```toml
[package_type.language_env]
language_env = "rust"
rust_version = "1.67.0"
rust_binary_url = "https://static.rust-lang.org/..."
rust_binary_gpg_asc = "..."
```

### Go

```toml
[package_type.language_env]
language_env = "go"
go_version = "1.19.0"
go_binary_url = "https://go.dev/dl/..."
go_binary_checksum = "..."
```

### JavaScript/TypeScript

```toml
[package_type.language_env]
language_env = "javascript"  # or "typescript"
node_version = "18.12.0"
node_binary_url = "https://nodejs.org/dist/..."
node_binary_checksum = "..."
yarn_version = "1.22.19"  # Optional
```

### Java

```toml
[package_type.language_env]
language_env = "java"
is_oracle = false
jdk_version = "17.0.5"
jdk_binary_url = "https://..."
jdk_binary_checksum = "..."

# Optional Gradle configuration
[package_type.language_env.gradle]
gradle_version = "7.6"
gradle_binary_url = "https://..."
gradle_binary_checksum = "..."
```

### .NET

```toml
[package_type.language_env]
language_env = "dotnet"
use_backup_version = false
deps = ["dependency1", "dependency2"]  # Optional

[[package_type.language_env.dotnet_packages]]
name = "package-name"
hash = "checksum"
url = "https://..."
```

### Nim

```toml
[package_type.language_env]
language_env = "nim"
nim_version = "1.6.8"
nim_binary_url = "https://nim-lang.org/..."
nim_version_checksum = "..."
```

## Build Environment

```toml
[build_env]
codename = "bookworm"                # Target distribution
arch = "amd64"                       # Target architecture
pkg_builder_version = "0.3.1"        # Tool version
debcrafter_version = "8189263"       # Debcrafter version
sbuild_cache_dir = "/path/to/cache"  # Optional cache directory, defaults to ~/.cache/sbuild
run_lintian = true                   # Enable lintian checks
run_piuparts = true                  # Enable piuparts tests
run_autopkgtest = true               # Enable autopkgtest
lintian_version = "2.116.3"          # Lintian version
piuparts_version = "1.1.7"           # Piuparts version
autopkgtest_version = "5.28"         # Autopkgtest version
sbuild_version = "0.85.6"            # Sbuild version
workdir = "~/.pkg-builder/..."       # Working directory
```