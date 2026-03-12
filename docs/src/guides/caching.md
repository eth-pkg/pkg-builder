# Caching packages for faster builds

When building or rebuilding many packages, each sbuild invocation fetches build dependencies from Debian/Ubuntu mirrors. A local caching proxy avoids redundant downloads and significantly speeds up batch builds.

## Setting up apt-cacher-ng

```bash
sudo apt install apt-cacher-ng
sudo systemctl enable --now apt-cacher-ng
```

This starts a caching proxy on port 3142. The admin UI is available at `http://localhost:3142/acng-report.html`.

## Transparent proxy (recommended)

To cache packages without modifying any build configs or sbuild chroots, redirect HTTP traffic at the kernel level:

```bash
sudo iptables -t nat -A OUTPUT -p tcp --dport 80 -m owner ! --uid-owner apt-cacher-ng -j REDIRECT --to-port 3142
```

This keeps builds fully reproducible — sbuild still fetches from the same mirrors with the same pinned snapshots, the cache just serves already-downloaded packages locally.

To persist the rule across reboots:

```bash
sudo apt install iptables-persistent
sudo netfilter-persistent save
```

## Verifying the cache is working

```bash
# Check service status
sudo systemctl status apt-cacher-ng

# Watch requests during a build
tail -f /var/log/apt-cacher-ng/apt-cacher.log

# Inspect cached files
ls /var/cache/apt-cacher-ng/
```

The web UI at `http://localhost:3142/acng-report.html` shows hit/miss ratios and transfer statistics.

### Reading the log — cache hits vs misses

Log format: `timestamp|direction|size|client_ip|path`

- **`I`** = delivered to client (inbound to requester)
- **`O`** = fetched from upstream mirror (outbound to internet)

**Cache miss** — both `I` and `O` lines appear for the same file (fetched from mirror, then forwarded):

```
1773338962|I|15002|192.168.1.102|uburep/pool/main/b/blinker/python3-blinker_1.7.0-1_all.deb
1773338962|O|14606|192.168.1.102|uburep/pool/main/b/blinker/python3-blinker_1.7.0-1_all.deb
```

**Cache hit** — only an `I` line, no matching `O` (served directly from local cache).

On the first build everything will be misses. Subsequent builds of the same or similar packages should show only `I` lines.

## Tracking exact package versions

sbuild generates `.buildinfo` files that list every build dependency with its exact version. These files are in the build output directory alongside the `.deb` and `.changes` files. Combined with the pinned snapshot archives in `pkg-builder.toml`, this provides a complete record of what went into each build.

## Limitations

- Only caches HTTP, not HTTPS. Mirrors in sbuild chroots should use `http://` URLs (e.g. `http://deb.debian.org`, `http://archive.ubuntu.com`). The default snapshot URLs used by pkg-builder already use HTTP.
- Index files (`Release`, `Packages.gz`) are re-validated on each request to stay current, so metadata is always fresh while `.deb` files are served from cache.
