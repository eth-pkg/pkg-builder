# Project Initialization

The `pkg-builder init` command scaffolds a new package project by generating a `pkg-builder.toml` configuration file and the debcrafter spec files (`.sss` and `.sps`).

## Interactive mode

Run `init` without flags for an interactive wizard that prompts for each value:

```bash
pkg-builder init --output ./my-package
```

The wizard walks through:

1. **Package metadata** — name, version, revision, homepage, maintainer, section, summary
2. **Source type** — tarball, git, or virtual
3. **Source details** — URL, tag (for git), hash (for tarballs)
4. **Build target** — distribution and architecture
5. **Runtime** — language toolchain recipe and its required fields

## Auto-resolution

For some runtimes, `init` can automatically fetch the latest version and its verification data:

| Runtime | Auto-resolved fields |
|---------|---------------------|
| Go | `binary_url`, `binary_checksum` (latest stable from go.dev) |
| Node.js | `binary_url`, `binary_checksum` (latest LTS from nodejs.org) |
| Rust | `binary_url`, `binary_gpg_asc` (fetches GPG signature for a given version) |
| Nim | `binary_url`, `binary_checksum` (downloads and hashes the archive) |

When auto-resolution succeeds, the wizard shows the resolved values and lets you confirm or override with a different version.

For tarball sources with remote URLs, `init` downloads the tarball to the output directory and computes its SHA-256 hash automatically.

## Generated files

`init` creates three files in the output directory:

### `pkg-builder.toml`

The main configuration file with all sections pre-populated:

- `[package]` — name, version, revision, homepage, spec file path
- `[source]` — type-specific source configuration
- `[build]` — distribution, architecture, working directory
- `[runtime]` — recipe and toolchain fields (if applicable)
- `[tools]` — pinned to current tool versions

### `<name>.sss` (source service spec)

A debcrafter source spec with:

- Package name and maintainer
- Debian section
- Empty build dependencies and variants (to be filled in)
- Reference to the binary package spec

### `<name>.sps` (package spec)

A debcrafter binary package spec with:

- Package name and architecture
- Summary and long description (from the summary you provided)
- Empty dependency, conflict, and file lists (to be filled in)

## Non-interactive mode

Pass all values as CLI flags to skip all prompts:

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

## Runtimes that require manual setup

The `.NET` runtimes (`dotnet-noble`, `dotnet-debian`, `dotnet-backup`) require manually specifying package lists in the `[runtime]` section after `init` generates the files. The wizard will inform you when this is the case.

## Next steps

After running `init`:

1. Review and customize the generated `pkg-builder.toml`
2. Fill in build dependencies in the `.sss` file
3. Add installed files, dependencies, and descriptions in the `.sps` file
4. Create the build environment: `pkg-builder --config ./my-package/pkg-builder.toml env create`
5. Build the package: `pkg-builder --config ./my-package/pkg-builder.toml build`
