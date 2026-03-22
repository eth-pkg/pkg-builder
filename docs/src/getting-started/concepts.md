# Concepts

This page explains the key components of the pkg-builder ecosystem and how they fit together.

## The packaging pipeline

Building a Debian package with pkg-builder involves three main tools:

```
pkg-builder.toml
       │
       ▼
  pkg-builder (executor)
       │
       ├── Step 1: Acquire source (download/clone/copy)
       ├── Step 2: Extract source tarball
       ├── Step 3: debcrafter ──► debian/ files (control, rules, changelog, ...)
       └── Step 4: sbuild ──────► .deb + .dsc (built inside a chroot)
```

### pkg-builder

The orchestrator. It reads your `pkg-builder.toml` configuration and directly executes a 4-step build pipeline:

1. **Acquires the source** (downloads tarballs via HTTP, clones git repos, or creates virtual packages)
2. **Extracts the source** tarball into the working directory
3. **Invokes debcrafter** (as a library) to generate `debian/` packaging files from `.sss` spec files
4. **Runs sbuild** to build the package inside an isolated chroot environment

### debcrafter

A separate tool ([debcrafter](https://github.com/Kixunil/debcrafter)) that generates Debian packaging files from `.sss` (source spec) and `.sps` (package spec) files. These spec files are a higher-level, more concise way to describe Debian package metadata and relationships than hand-writing `debian/control`, `debian/rules`, etc.

Key files:
- **`.sss` (source service spec)** — describes the source package: build dependencies, binary packages it produces, and their relationships
- **`.sps` (package spec)** — describes individual binary packages and their contents
- **`.changelog`** — standard Debian changelog, used by debcrafter during generation

### sbuild

A Debian tool that builds packages inside an isolated chroot environment. This ensures builds are clean and reproducible — only explicitly declared dependencies are available. pkg-builder uses a [patched fork](https://github.com/eth-pkg/sbuild) of sbuild.

The chroot is created once per distribution with `pkg-builder env create` (or `pkg-builder --config <path> env create`) and reused for all subsequent builds.

## Runtime recipes

Not all languages can build with just the packages available in the Debian archive. Rust, Go, Node.js, Java, Nim, and .NET all need external toolchains installed into the chroot before the build runs.

pkg-builder handles this through **runtime recipes** — template files in `runtimes/` that describe how to download and install a specific toolchain. Each recipe uses template variables (like `{{binary_url}}`) that get filled in from the `[runtime]` section of your `pkg-builder.toml`.

For example, a Go package needs:

```toml
[runtime]
recipe = "go"
binary_url = "https://go.dev/dl/go1.22.2.linux-amd64.tar.gz"
binary_checksum = "5901c52b7a78002aeff14a21f93e0f064f74ce1360fce51c6ee68cd471216a17"
```

C packages don't need a `[runtime]` section — the C toolchain is already in the base chroot.

See [Runtime Recipes](../guides/runtime-recipes.md) for the full list of recipes and their required fields.

## Reproducibility

pkg-builder achieves reproducible builds through several mechanisms:

- **Snapshot pinning** — locks Debian/Ubuntu package archives to a specific date so dependency resolution is deterministic
- **Tool version pinning** — the `[tools]` section records exact versions of pkg-builder, debcrafter, sbuild, and test tools
- **Source hashing** — the `[source].hash` field ensures the source code hasn't changed
- **Output verification** — the `[verify]` section stores expected SHA-1 hashes of built `.dsc` and `.deb` files

Together, these mean that given the same `pkg-builder.toml`, you should always get the same output.

## Working directory

Each build uses a working directory (`[build].workdir`) where pkg-builder stages the source tree, runs the build, and places output files. By default this is `~/.pkg-builder/packages/<distribution>/`. The `pkg-builder clean` command removes build artifacts from this directory.
