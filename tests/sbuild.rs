
#[cfg(test)]
mod tests {
    use env_logger::Env;
    use std::sync::Once;
    use tempfile::{tempdir};
    use pkg_builder::v1::cli::{get_config, get_distribution};

    static INIT: Once = Once::new();

    // Set up logging for tests
    fn setup() {
        INIT.call_once(|| {
            env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
        });
    }

    #[test]
    fn test_build_virtual_package_in_sbuild_env() {
        setup();


        let config_file = "examples/bookworm/virtual-package/pkg-builder.toml".to_string();

        let temp_dir = tempdir().unwrap();
        let build_files_dir = temp_dir.path().to_string_lossy().to_string() ;
        let sbuild_cach_dir = temp_dir.path().to_string_lossy().to_string();
        let mut config = get_config(config_file.clone()).expect("Could not read config_file");
        config.build_env.workdir = Some(build_files_dir);
        config.build_env.sbuild_cache_dir = Some(sbuild_cach_dir);
        let distribution = get_distribution(config, config_file).expect("Could not get distribution");

        let result = distribution.clean_build_env();
        assert!(result.is_ok());
        let result = distribution.create_build_env();
        assert!(result.is_ok());
     //   assert!(temp_file.path().exists());

        let result = distribution.package();
        assert!(result.is_ok());
        assert!(temp_dir.path().exists())
    }
    #[test]
    #[ignore]
    fn test_build_rust_package_in_sbuild_env() {
        setup();

        unreachable!("Test case not implemented yet");
    }
    #[test]
    #[ignore]

    fn test_build_go_package_in_sbuild_env() {
        setup();

        unreachable!("Test case not implemented yet");
    }

    #[test]
    #[ignore]

    fn test_build_javascript_package_in_sbuild_env() {
        setup();

        unreachable!("Test case not implemented yet");
    }

    #[test]
    #[ignore]

    fn test_build_java_package_in_sbuild_env() {
        setup();

        unreachable!("Test case not implemented yet");
    }

    #[test]
    #[ignore]

    fn test_build_dotnet_package_in_sbuild_env() {
        setup();

        unreachable!("Test case not implemented yet");
    }

    #[test]
    #[ignore]

    fn test_build_typescript_package_in_sbuild_env() {
        setup();

        unreachable!("Test case not implemented yet");
    }

    #[test]
    #[ignore]

    fn test_build_nim_package_in_sbuild_env() {
        setup();
        unreachable!("Test case not implemented yet");
    }
}
