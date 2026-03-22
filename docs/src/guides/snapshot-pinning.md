# Snapshot Pinning

Snapshot pinning locks your build against a specific point-in-time copy of the Debian package archive. This ensures that dependency resolution produces the same result every time, regardless of when you run the build.

## How it works

Debian maintains daily snapshots of its archive at [snapshot.debian.org](https://snapshot.debian.org/). When you specify a snapshot date, pkg-builder configures the sbuild chroot to use that snapshot instead of the live mirror. This means `apt` inside the chroot sees exactly the same packages that were available on that date.

## Configuration

Add snapshot fields to the `[build]` section:

```toml
[build]
distribution = "trixie"
arch = "amd64"
workdir = "~/.pkg-builder/packages/trixie"
snapshot_date = "20250101"
snapshot_security_date = "20250115T000000Z"
```

| Field | Format | Description |
|-------|--------|-------------|
| `snapshot_date` | `YYYYMMDD` | Date of the main archive snapshot |
| `snapshot_security_date` | `YYYYMMDDTHHMMSSZ` | Date of the security archive snapshot |

Both fields are optional. If omitted, the build uses the live mirrors.

## When to use snapshot pinning

- **Reproducible builds** — pin to a date to get deterministic dependency resolution
- **CI stability** — prevent builds from breaking when new package versions are uploaded
- **Auditing** — record exactly which dependencies were used for a specific build

## Limitations

- Snapshot pinning is only available for **Debian** distributions (bookworm, trixie). Ubuntu does not have an equivalent snapshot service.
- Snapshots may eventually be garbage-collected by snapshot.debian.org, though in practice they are retained for years.
- The security snapshot date is separate from the main archive date because security updates are published on a different cadence.

## Choosing dates

A good strategy is to set the snapshot date to the day you finalize the package, then record it in `pkg-builder.toml` for future builds. If you need a security fix that was published after your snapshot date, update `snapshot_security_date` to a date after the fix was released.
