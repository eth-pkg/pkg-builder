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

Tests enabled in the config run automatically after a successful build when using `pkg-builder build --with-tests`. You can also run them individually.

## Lintian

[Lintian](https://lintian.debian.org/) is a static analysis tool that checks `.deb` and `.dsc` files against Debian policy. It catches common packaging errors like missing fields, incorrect permissions, and policy violations.

There are two ways to run lintian:

```bash
# Run lintian on the host against built .changes files
pkg-builder test lintian

# Or run lintian inside sbuild as part of the build (when lintian is enabled in config)
pkg-builder build --with-tests
```

Note: `test lintian` runs lintian on the host, which is separate from lintian running inside sbuild during `build --with-tests`.

### Suppressing warnings

Some lintian warnings may not apply to your package. Add overrides in `src/debian/<package-name>.lintian-overrides` for binary package warnings, or `src/debian/source/lintian-overrides` for source package warnings.

## Piuparts

[Piuparts](https://piuparts.debian.org/) tests the package install/upgrade/removal cycle. It installs the `.deb` in a clean chroot, then removes it, checking for errors at each step.

```bash
pkg-builder test piuparts
```

Piuparts catches issues like:

- Files left behind after removal
- Missing dependencies that prevent installation
- Broken maintainer scripts (preinst, postinst, prerm, postrm)

## Autopkgtest

[Autopkgtest](https://autopkgtest.ubuntu.com/) runs functional tests defined in `debian/tests/`. These are tests that exercise the installed package rather than the build process.

```bash
pkg-builder test autopkgtest
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

Use `--with-tests` to run all tests enabled in your config after a successful build:

```bash
pkg-builder build --with-tests
```

## Tool versions

Pin test tool versions in `[tools]` for reproducibility:

```toml
[tools]
lintian = "2.116.3"
piuparts = "1.1.7"
autopkgtest = "5.28"
```
