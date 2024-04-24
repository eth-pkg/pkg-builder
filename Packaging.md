

## Patch source

Install quilt if not installed. 

Checkout source that you need to modify, you specify version number if needed
   ```bash
   # Assuming that your downloaded source already setup with in <DIR_TO_YOUR_PACKAGE_FILES>
   pkg-builder package <YOUR_PACKAGE>/pkg-builder.toml
   cd .pkg-builder/packages/<YOUR_PACKAGE>/<YOUR_PACKAGE>-<PACKAGE_VERSION_WITHOUT_REVISION>
   ln -s debian/patches patches # create link so series file if not exist yet
   dquilt push -a # apply existing patches
   
   dquilt new your_patch_name.patch
   dquilt add <FILE_YOU_WISH_TO_MODIY>
   ... do your changes 
   ... vim <FILE_YOU_WISH_TO_MODIY>
   ... 
   
   dquilt refresh # this will save the modified patches under /debian/patches
   ```

Copy the patched source to the folder, so you can build package from it.
   ```bash
   cp -R .pc <DIR_TO_YOUR_PACKAGE_FILES>/<YOUR_PACKAGE>/src/debian 
   cp -R debian/patches <DIR_TO_YOUR_PACKAGE_FILES>/<YOUR_PACKAGE>/src/debian 
   ```
