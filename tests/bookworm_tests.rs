#[cfg(test)]
mod bookworm_tests {
    use env_logger::Env;
    use pkg_builder::v1::distribution::debian::bookworm::{
        BookwormPackager, BookwormPackagerConfigBuilder,
    };
    use pkg_builder::v1::packager::{Packager, PackagerError};
    
    #[test]
    fn test_create_virtual_package() {
        env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
        let config = BookwormPackagerConfigBuilder::new()
            .arch(Some("amd64".to_string()))
            .package_name(Some("test-virtual-package".to_string()))
            .version_number(Some("1.0.0".to_string()))
            .tarball_url(None)
            .git_source(None)
            .is_virtual_package(true)
            .debcrafter_version(Some("latest".to_string()))
            .spec_file(Some(
                "examples/bookworm/virtual-package/test-virtual-package.sss".to_string(),
            ))
            .config()
            .map_err(|err| PackagerError::MissingConfigFields(err.to_string()))
            .unwrap();

        let packager = BookwormPackager::new(config);
        let result = packager.package();

        let error_message = "sbuild_createchroot is not installed. Please install it".to_string();
        assert!(result.is_err() && result.unwrap_err().contains(&error_message));
    }

}
