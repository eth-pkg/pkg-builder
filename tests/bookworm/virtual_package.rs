#[cfg(test)]
mod virtual_package {
    use env_logger::Env;
    use pkg_builder::v1::distribution::debian::bookworm::{
        BookwormPackager,
    };
    use pkg_builder::v1::distribution::debian::bookworm_config_builder::{BookwormPackagerConfig, BookwormPackagerConfigBuilder};
    use pkg_builder::v1::packager::{Packager, BackendBuildEnv};
    use std::sync::Once;
    use pkg_builder::v1::packager;

    static INIT: Once = Once::new();

    // Set up logging for tests
    fn setup() {
        INIT.call_once(|| {
            env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
        });
    }
    fn get_virtual_package_config() -> BookwormPackagerConfig {
        let config = BookwormPackagerConfigBuilder::default()
            .arch(Some("amd64".to_string()))
            .package_name(Some("test-virtual-package".to_string()))
            .version_number(Some("1.0.0".to_string()))
            .tarball_url(None)
            .git_source(None)
            .package_type(Some("virtual".to_string()))
            .debcrafter_version(Some("latest".to_string()))
            .spec_file(Some(
                "test-virtual-package.sss".to_string(),
            ))
            .config_root("examples/bookworm/virtual-package/".to_string())
            .homepage(Some("https://github.com/eth-pkg/pkg-builder#examples".to_string()))
            .config()
            .map_err(|err| packager::Error::MissingConfigFields(err.to_string()))
            .unwrap();
        config
    }


    #[test]
    fn test_virtual_package_build() {
        setup();
        let config = get_virtual_package_config();
        let packager = BookwormPackager::new(config);
        let result = packager.package();

        assert!(result.is_ok());
    }

    #[test]
    fn test_virtual_package_clean_build_env() {
        setup();
        let config = get_virtual_package_config();
        let packager = BookwormPackager::new(config);
        let build_env = packager.get_build_env();

        assert!(build_env.is_ok());

        let result = build_env.unwrap().clean();

        assert!(result.is_err(), "Command wasn't invoked with root privileges");
    }

    #[test]
    fn test_virtual_package_create_build_env() {
        setup();

        let config = get_virtual_package_config();

        let packager = BookwormPackager::new(config);
        let build_env = packager.get_build_env();

        assert!(build_env.is_ok());

        let result = build_env.unwrap().create();

        assert!(result.is_err(), "Command must be invoked with root privileges");
    }

}
