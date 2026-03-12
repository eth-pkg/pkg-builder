# Testing

pkg-builder integrates three Debian quality assurance tools: lintian, piuparts, and autopkgtest. Each checks a different aspect of your package.

## Enabling tests

Add a `[testing]` section to your `pkg-builder.toml`:

```toml
[testing]
lintian = true
piuparts = true
autopkgtest = true
```

All three default to `false` if the `[testing]` section is omitted.

Tests run automatically after a successful build when using `pkg-builder package`. You can also run them individually.

## Lintian

[Lintian](https://lintian.debian.org/) is a static analysis tool that checks `.deb` and `.dsc` files against Debian policy. It catches common packaging errors like missing fields, incorrect permissions, and policy violations.

```bash
# Run after building
pkg-builder lintian path/to/pkg-builder.toml

# Or as part of the build
pkg-builder package path/to/pkg-builder.toml --run-lintian true
```

### Suppressing warnings

Some lintian warnings may not apply to your package. Add overrides in `src/debian/<package-name>.lintian-overrides` for binary package warnings, or `src/debian/source/lintian-overrides` for source package warnings.

## Piuparts

[Piuparts](https://piuparts.debian.org/) tests the package install/upgrade/removal cycle. It installs the `.deb` in a clean chroot, then removes it, checking for errors at each step.

```bash
pkg-builder piuparts path/to/pkg-builder.toml
```

Piuparts catches issues like:

- Files left behind after removal
- Missing dependencies that prevent installation
- Broken maintainer scripts (preinst, postinst, prerm, postrm)

## Autopkgtest

[Autopkgtest](https://autopkgtest.ubuntu.com/) runs functional tests defined in `debian/tests/`. These are tests that exercise the installed package rather than the build process.

```bash
pkg-builder autopkgtest path/to/pkg-builder.toml
```

### Defining tests

Create `src/debian/tests/control` and `src/debian/tests/tests` in your package source:

**`src/debian/tests/control`:**
```
Tests: tests
Depends: <package-name>
```

**`src/debian/tests/tests`:**
```bash
#!/bin/sh
set -e
# Test that the installed binary works
hello-world
```

The test script should exit 0 on success and non-zero on failure.

## Running all tests during build

You can enable tests via CLI flags without modifying the config file:

```bash
pkg-builder package path/to/pkg-builder.toml --run-lintian true --run-piuparts --run-autopkgtest
```

## Tool versions

Pin test tool versions in `[tools]` for reproducibility:

```toml
[tools]
lintian = "2.116.3"
piuparts = "1.1.7"
autopkgtest = "5.28"
```
