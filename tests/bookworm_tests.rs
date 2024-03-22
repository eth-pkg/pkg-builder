#[cfg(test)]
mod bookworm_tests {
    use pkg_builder::v1::distribution::debian::bookworm::{
        BookwormPackager, BookwormPackagerConfigBuilder,
    };
    use pkg_builder::v1::packager::{Packager, PackagerError};

    #[test]
    fn test_create_virtual_package() {
        let config = BookwormPackagerConfigBuilder::new()
            .arch(Some("amd64".to_string()))
            .package_name(Some("test-virtual-package".to_string()))
            .version_number(Some("1.0.0".to_string()))
            .tarball_url(None)
            .git_source(None)
            .is_virtual_package(true)
            .lang_env(Some("rust".to_string()))
            .debcrafter_version(Some("latest".to_string()))
            .spec_file(Some(
                "tests/misc/bookworm/virtual-package/test-virtual-package.sss".to_string(),
            ))
            .config()
            .map_err(|err| PackagerError::MissingConfigFields(err.to_string()))
            .unwrap();

        let packager = BookwormPackager::new(config);
        let result = packager
            .package()
            .map_err(|err| PackagerError::PackagingError(err.to_string()));

        assert!(result.is_ok(), "{:?}", result);
    }
}
