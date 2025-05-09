# FAQ

## Patch source

Install quilt if not installed. 

Checkout source that you need to modify, you specify version number if needed
   ```bash
   # Assuming that your downloaded source already setup with in <DIR_TO_YOUR_PACKAGE_FILES>
   pkg-builder package <YOUR_PACKAGE>/pkg-builder.toml
   cd .pkg-builder/packages/<YOUR_PACKAGE>/<YOUR_PACKAGE>-<PACKAGE_VERSION_WITHOUT_REVISION>
   quilt push -a # apply existing patches
   
   quilt new your_patch_name.patch
   quilt add <FILE_YOU_WISH_TO_MODIY>
   ... do your changes 
   ... vim <FILE_YOU_WISH_TO_MODIY>
   ... 
   
   quilt refresh # this will save the modified patches under /debian/patches
   ```

Copy the patched source to the folder, so you can build package from it.
   ```bash
   rm .pc/.quilt_patches
   rm .pc/.quilt_series
   rm .pc/.version
   rm .pc/applied-patches
   cp -R .pc <DIR_TO_YOUR_PACKAGE_FILES>/<YOUR_PACKAGE>/src 
   cp -R patches <DIR_TO_YOUR_PACKAGE_FILES>/<YOUR_PACKAGE>/src/debian 
   
   ```

## Lintian errors 

### generating copyright files 
Generally recommended to check the package for copyright and include that as such 

```text
Files: *
Copyright: 2022 ORIGINAL PACKAGE AUTHORS
License: GPL-3+

Files: debian/*
Copyright: 2022 MAINTAINER_NAME
License: GPL-3+

# Provide the short license for each referenced license
License: GPL-3+
 The full text of the GPL version 3 is distributed in
 /usr/share/common-licenses/GPL-3 on Debian systems.
```

But in some cases some file could be in another license, which lintian going to fail (license included in file header),
so for this purposes it is good to checkout the source code or navigate to the pkg-builder dir 
and run `debmake -c > debian/copyright` and check for such files by comparing the two files. 

### build essential warning 

In case encountaring the following error message, remove build-essential from your dependencies.
`build-essential` packages are already present in the minimal chroot environment provided by `sbuild`.


```text
E: <package> source: build-depends-on-build-essential Build-Depends
N:
N:   You depend on the build-essential package, which is only a metapackage
N:   depending on build tools that have to be installed in all build
N:   environments.
N:
N:   Please refer to Relationships between source and binary packages -
N:   Build-Depends, Build-Depends-Indep, Build-Depends-Arch, Build-Conflicts,
N:   Build-Conflicts-Indep, Build-Conflicts-Arch (Section 7.7) in the Debian
N:   Policy Manual for details.
N:
N:   Visibility: error
N:   Show-Always: no
N:   Check: fields/package-relations
```