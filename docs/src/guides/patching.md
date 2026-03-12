# Patching

Sometimes the upstream source needs modifications to build correctly as a Debian package. pkg-builder uses [quilt](https://wiki.debian.org/UsingQuilt) for managing patches, the standard Debian patching tool.

## Creating a patch

First, build the package to get the unpacked source tree:

```bash
pkg-builder package path/to/pkg-builder.toml
```

Then navigate to the unpacked source in the working directory:

```bash
cd ~/.pkg-builder/packages/<distribution>/<package-name>-<version>
```

Apply any existing patches and create a new one:

```bash
quilt push -a              # Apply existing patches
quilt new your_patch.patch # Start a new patch
quilt add <file>           # Track a file for changes
```

Make your changes to the tracked file(s), then save:

```bash
quilt refresh              # Save the patch to debian/patches/
```

## Copying patches back

After creating the patch, copy the quilt state back to your packaging source directory:

```bash
# Clean up quilt metadata
rm .pc/.quilt_patches
rm .pc/.quilt_series
rm .pc/.version
rm .pc/applied-patches

# Copy back to your package source
cp -R .pc <your-package-dir>/src
cp -R patches <your-package-dir>/src/debian
```

Rebuild the package to verify the patch applies cleanly.
