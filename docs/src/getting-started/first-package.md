# Your First Package

This tutorial walks through building a simple C hello-world package for Debian bookworm using pkg-builder. By the end, you'll have a working `.deb` file.

## Prerequisites

Make sure you've completed the [Installation](installation.md) steps. You should have `pkg-builder` on your PATH and sbuild set up.

## Option A: Use `pkg-builder init`

The fastest way to create a new package is with the interactive wizard:

```bash
pkg-builder init --output ./my-package
```

This prompts for package name, version, source type, distribution, runtime, and more. It generates three files:

- `pkg-builder.toml` — package configuration
- `<name>.sss` — debcrafter source spec
- `<name>.sps` — debcrafter package spec

For tarball sources, `init` can automatically download the tarball and compute its SHA-256 hash. For Go, Node.js, Rust, and Nim runtimes, it can auto-resolve the latest toolchain version and fetch checksums.

You can also run it fully non-interactively by passing all values as flags — see the [CLI Reference](../reference/cli.md#pkg-builder-init) for details.

Once the files are generated, skip ahead to [Step 1: Create the build environment](#step-1-create-the-build-environment).

## Option B: Use an existing example

## The example project

The repository includes a complete C example at `examples/bookworm/c/hello-world/`. Let's look at what's inside:

```
examples/bookworm/c/hello-world/
  pkg-builder.toml          # Package configuration
  hello-world-c.sss         # debcrafter spec file
  hello-world-c.sps         # debcrafter source spec
  hello-world-c.changelog   # Debian changelog
  hello-world-1.0.0.tar.gz  # Source tarball
  src/debian/                # Debian packaging files (rules, copyright, tests)
```

## The configuration file

Here's the `pkg-builder.toml` that defines the package:

```toml
[package]
name = "hello-world-c"
version = "1.0.0"
revision = "1"
homepage = "https://github.com/eth-pkg/pkg-builder#examples"
spec = "hello-world-c.sss"

[source]
type = "tarball"
url = "hello-world-1.0.0.tar.gz"
hash = "c93bdd829eca65af1e..."

[build]
distribution = "bookworm"
arch = "amd64"
workdir = "~/.pkg-builder/packages/bookworm"

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
autopkgtest = "5.28"

[verify]
package_hash = [
    { name = "hello-world-c_1.0.0-1.dsc", hash = "0843091f9e3cff991450c9687935e67644f6b032" },
    { name = "hello-world-c_1.0.0-1_amd64.deb", hash = "acd656c4554f41eb2a0ad570ef99ff19db51273d" },
]
```

Let's break this down:

- **`[package]`** — Package metadata: name, version, revision, and the debcrafter `.sss` spec file.
- **`[source]`** — Where to get the source code. This example uses a local tarball, but it could be a URL or a git repository.
- **`[build]`** — Target distribution, architecture, and working directory for build artifacts.
- **`[testing]`** — Which tests to run after building (lintian, piuparts, autopkgtest).
- **`[tools]`** — Pinned versions of all external tools for reproducibility.
- **`[verify]`** — Expected SHA-1 hashes of the output `.dsc` and `.deb` files.

## Step 1: Create the build environment

The build environment is an sbuild chroot — an isolated Debian installation where the package gets built:

```bash
pkg-builder --config examples/bookworm/c/hello-world/pkg-builder.toml env create
```

This creates a bookworm amd64 chroot under `/srv/chroot/`. You only need to do this once per distribution.

## Step 2: Build the package

```bash
pkg-builder --config examples/bookworm/c/hello-world/pkg-builder.toml build
```

pkg-builder will:

1. Generate a Makefile from the C runtime recipe and distribution template
2. Prepare the source tarball in the working directory
3. Run debcrafter to generate Debian packaging files from the `.sss` spec
4. Invoke sbuild to build the package inside the chroot
5. Run lintian, piuparts, and autopkgtest (as configured in `[testing]`)

The output `.deb` and `.dsc` files are placed in the workdir (`~/.pkg-builder/packages/bookworm/`).

## Step 3: Verify the build

```bash
pkg-builder --config examples/bookworm/c/hello-world/pkg-builder.toml verify
```

This compares the SHA-1 hashes of the built files against the expected values in `[verify].package_hash`. If they match, the build is reproducible.

## Next steps

- Read [Concepts](concepts.md) to understand how pkg-builder, debcrafter, and sbuild fit together
- See [Source Types](../guides/source-types.md) to learn about git and virtual package sources
- See [Runtime Recipes](../guides/runtime-recipes.md) to build packages for other languages
- Browse `examples/` for working configurations across all supported languages and distributions
