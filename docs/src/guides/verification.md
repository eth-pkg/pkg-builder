# Verification

pkg-builder supports hash verification to confirm that a build is reproducible — that the same configuration produces the same output files.

## How it works

The `[verify]` section in `pkg-builder.toml` stores expected SHA-1 hashes for the built `.dsc` (source package descriptor) and `.deb` (binary package) files:

```toml
[verify]
package_hash = [
    { name = "hello-world-c_1.0.0-1.dsc", hash = "0843091f9e3cff991450c9687935e67644f6b032" },
    { name = "hello-world-c_1.0.0-1_amd64.deb", hash = "acd656c4554f41eb2a0ad570ef99ff19db51273d" },
]
```

Each entry specifies the expected filename and its SHA-1 hash.

## Running verification

```bash
pkg-builder verify
```

This computes the SHA-1 hash of each file listed in `package_hash` (looking in the workdir) and compares it against the expected value. If any hash doesn't match, the command reports which files differ.

## Setting up verification for a new package

1. Build the package without a `[verify]` section (or with empty hashes)
2. Run `pkg-builder verify` — it will report the actual hashes
3. Copy the actual hashes into your `pkg-builder.toml`
4. Rebuild and verify again to confirm reproducibility

## File naming convention

The expected filenames follow Debian conventions:

- **`.dsc`**: `<name>_<version>-<revision>.dsc`
- **`.deb`**: `<name>_<version>-<revision>_<arch>.deb`

For example, package `hello-world-c` version `1.0.0` revision `1` on amd64:
- `hello-world-c_1.0.0-1.dsc`
- `hello-world-c_1.0.0-1_amd64.deb`

## Prerequisites for reproducibility

For hashes to match across builds, you typically need:

- **Snapshot pinning** — so dependencies are identical (see [Snapshot Pinning](snapshot-pinning.md))
- **Pinned tool versions** — same sbuild, debcrafter, and pkg-builder versions
- **Same architecture** — cross-architecture builds produce different binaries
- **Normalized timestamps** — pkg-builder handles this automatically via `SOURCE_DATE_EPOCH`
