# Source Types

The `[source]` section of `pkg-builder.toml` tells pkg-builder where to get the source code for your package. There are three source types: tarball, git, and virtual.

## Tarball

The most common source type. Points to a `.tar.gz` or `.tar.xz` archive, either a local file or a remote URL.

**Local file:**

```toml
[source]
type = "tarball"
url = "hello-world-1.0.0.tar.gz"
hash = "c93bdd829eca65af1e303d4a0b31cde0c3d3c2003fa1ca985393c412264b42c3..."
```

**Remote URL:**

```toml
[source]
type = "tarball"
url = "https://example.com/project-1.0.0.tar.gz"
hash = "a7c7eb7779e319cc9c884128a64bd0997481c16aab75824b7f6a02844f847cbc..."
```

The `hash` field is optional but recommended. It contains a SHA-512 hash of the tarball for integrity verification.

## Git

Clone a git repository at a specific tag. Useful when the upstream project doesn't publish release tarballs, or when you need to include git submodules.

```toml
[source]
type = "git"
url = "https://github.com/org/repo.git"
tag = "v1.0.0"
```

### Submodule pinning

For reproducible builds with git submodules, pin each submodule to a specific commit:

```toml
[source]
type = "git"
url = "https://github.com/org/repo.git"
tag = "v1.0.0"
submodules = [
    { commit = "72523ee3f865...", path = "vendor/lib1" },
    { commit = "ab3ff9fad45f...", path = "vendor/lib2" },
]
```

Each entry specifies the expected commit hash and the submodule path. pkg-builder verifies these during source preparation.

## Virtual

For meta-packages that don't contain any source code — they only declare dependencies on other packages.

```toml
[source]
type = "virtual"
```

No other fields are needed. The resulting package will have no files of its own, just dependency relationships defined in the debcrafter spec.

## Choosing a source type

| Scenario | Source type |
|----------|------------|
| Upstream publishes release tarballs | `tarball` with remote URL |
| You maintain the source alongside the packaging | `tarball` with local file |
| Upstream uses git tags but no tarballs | `git` |
| Project has git submodules | `git` with submodule pinning |
| Dependency-only meta-package | `virtual` |
