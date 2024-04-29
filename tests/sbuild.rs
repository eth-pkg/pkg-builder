#[cfg(test)]
mod tests {
    use env_logger::Env;
    use pkg_builder::v1::pkg_config::{get_config, PkgConfig};
    use std::fs;
    use std::path::Path;
    use std::sync::Once;
    use pkg_builder::v1::cli::get_distribution;

    static INIT: Once = Once::new();
    static CODENAME: &str = "bookworm";
    static ARCH: &str = "amd64";
    static SBUILD_CACHE_DIR: &str = "/tmp/pkg-builder/cache-dir";
    static BUILD_FILES_DIR: &str = "/tmp/pkg-builder/build-dir";
    // Set up logging for tests

    fn setup_build_env() {
        // Doesn't matter which config file is read for env setup
        let config_file = "examples/bookworm/virtual-package/pkg-builder.toml".to_string();
        let cache_file_name = format!("{}-{}.tar.gz", CODENAME, ARCH);
        let cache_file = Path::new(SBUILD_CACHE_DIR).join(cache_file_name);
        if cache_file.exists() {
            // cache file exists do not recreate it
            return;
        }
        let mut config = get_config::<PkgConfig>(config_file.clone()).expect("Could not read config_file");
        config.build_env.workdir = Some(BUILD_FILES_DIR.to_string());
        config.build_env.sbuild_cache_dir = Some(SBUILD_CACHE_DIR.to_string());
        let distribution =
            get_distribution(config, config_file).expect("Could not get distribution");

        let result = distribution.clean_build_env();
        match result {
            Ok(_) => {
                assert!(result.is_ok());
            }
            Err(err) => {
                panic!("Could not clean build env: {}", err);
            }
        }
        let result = distribution.create_build_env();
        match result {
            Ok(_) => {
                assert!(result.is_ok());
            }
            Err(err) => {
                panic!("Could not create build env: {}", err);
            }
        }
    }
    fn setup() {
        INIT.call_once(|| {
            env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
        });
    }

    fn get_debian_output(config: PkgConfig) -> Vec<String> {
        let mut vec = vec![
            format!(
                "{}_{}-{}_{}.buildinfo",
                config.package_fields.package_name,
                config.package_fields.version_number,
                config.package_fields.revision_number,
                config.build_env.arch,
            ),
            format!(
                "{}-{}",
                config.package_fields.package_name, config.package_fields.version_number,
            ),
            format!(
                "{}_{}-{}_source.changes",
                config.package_fields.package_name,
                config.package_fields.version_number,
                config.package_fields.revision_number,
            ),
            format!(
                "{}_{}.orig.tar.gz",
                config.package_fields.package_name, config.package_fields.version_number,
            ),
            format!(
                "{}_{}-{}_{}.build",
                config.package_fields.package_name,
                config.package_fields.version_number,
                config.package_fields.revision_number,
                config.build_env.arch,
            ),
            // format!(
            //     "{}-{}-{}_{}-<timezone>.build",
            //     config.package_fields.package_name,
            //     config.package_fields.version_number,
            //     config.package_fields.revision_number,
            //     config.build_env.arch,
            // ),
            format!(
                "{}_{}-{}_{}.changes",
                config.package_fields.package_name,
                config.package_fields.version_number,
                config.package_fields.revision_number,
                config.build_env.arch,
            ),
            format!(
                "{}_{}-{}.dsc",
                config.package_fields.package_name,
                config.package_fields.version_number,
                config.package_fields.revision_number,
            ),
            format!(
                "{}_{}-{}.debian.tar.xz",
                config.package_fields.package_name,
                config.package_fields.version_number,
                config.package_fields.revision_number,
            ),
            format!(
                "{}_{}-{}_{}.deb",
                config.package_fields.package_name,
                config.package_fields.version_number,
                config.package_fields.revision_number,
                config.build_env.arch
            ),
        ];
        vec.sort();
        vec
    }

    fn test_builds(config_file: &str, run_piuparts: bool, run_autopkgtest: bool) {
        let mut config = get_config::<PkgConfig>(config_file.to_string()).expect("Could not read config_file");
        let work_dir = Path::new(BUILD_FILES_DIR).join("virtual");
        config.build_env.workdir = Some(work_dir.clone().to_str().unwrap().to_string());
        config.build_env.sbuild_cache_dir = Some(SBUILD_CACHE_DIR.to_string());
        config.build_env.run_piuparts = Some(run_piuparts);
        config.build_env.run_autopkgtest = Some(run_autopkgtest);
        let distribution = get_distribution(config.clone(), config_file.to_string())
            .expect("Could not get distribution");

        let result = if run_piuparts {
            distribution.run_piuparts()
        } else if run_autopkgtest {
            distribution.run_autopkgtests()
        } else {
            distribution.package()
        };
        match result {
            Ok(_) => {
                assert!(result.is_ok());
            }
            Err(err) => {
                let action = if run_piuparts {
                    "piuparts"
                } else if run_autopkgtest {
                    "autopkgtests"
                } else {
                    "package"
                };
                panic!("Could not {}: {}", action, err);
            }
        }
        // Read the contents of the directory
        let build_artificats_dir = work_dir.join(config.package_fields.package_name.clone());
        let entries =
            fs::read_dir(build_artificats_dir.clone()).expect("Could not read package directory.");
        let mut output: Vec<String> = vec![];
        for entry in entries {
            let entry = entry.expect("Could not access entry.");
            let file_name = entry.file_name().into_string().unwrap();
            // TODO ignore testing the timezone build
            if !file_name.ends_with("Z.build") && !file_name.contains("dbgsym") {
                output.push(file_name);
            }
        }
        output.sort();
        let expected_output = get_debian_output(config);
        let build_artificats_dir_path: &Path = build_artificats_dir.as_ref();
        assert!(build_artificats_dir_path.exists());
        // Check if the vectors are equal
        if expected_output.len() != output.len() {
            panic!("Number of files does not match, expected:{:?}, received:{:?}", expected_output, output)
        }

        for (idx, (expected, actual)) in expected_output.iter().zip(output.iter()).enumerate() {
            assert_eq!(
                expected, actual,
                "File at index {} does not match: expected '{}', actual '{}'",
                idx, expected, actual
            );
        }
    }
    #[test]
    fn test_build_virtual_package_in_sbuild_env() {
        setup();
        setup_build_env();
        let config_file = "examples/bookworm/virtual-package/pkg-builder.toml".to_string();
        test_builds(&config_file, false, false);
    }

    #[test]
    fn test_build_rust_package_in_sbuild_env() {
        setup();
        setup_build_env();

        let config_file = "examples/bookworm/rust/hello-world/pkg-builder.toml".to_string();
        test_builds(&config_file, false, false);
    }

    #[test]
    fn test_build_go_package_in_sbuild_env() {
        setup();
        setup_build_env();

        let config_file = "examples/bookworm/go/hello-world/pkg-builder.toml".to_string();
        test_builds(&config_file, false, false);
    }

    #[test]
    fn test_build_javascript_package_in_sbuild_env() {
        setup();
        setup_build_env();

        let config_file = "examples/bookworm/javascript/hello-world/pkg-builder.toml".to_string();
        test_builds(&config_file, false, false);
    }

    #[test]
    fn test_build_java_package_in_sbuild_env() {
        setup();
        setup_build_env();

        let config_file = "examples/bookworm/java/hello-world/pkg-builder.toml".to_string();
        test_builds(&config_file, false, false);
    }

    #[test]
    fn test_build_dotnet_package_in_sbuild_env() {
        setup();
        setup_build_env();
        let config_file = "examples/bookworm/dotnet/hello-world/pkg-builder.toml".to_string();
        test_builds(&config_file, false, false);
    }

    #[test]
    fn test_build_typescript_package_in_sbuild_env() {
        setup();
        setup_build_env();

        let config_file = "examples/bookworm/typescript/hello-world/pkg-builder.toml".to_string();
        test_builds(&config_file, false, false);
    }

    #[test]
    fn test_build_nim_package_in_sbuild_env() {
        setup();
        setup_build_env();

        let config_file = "examples/bookworm/nim/hello-world/pkg-builder.toml".to_string();
        test_builds(&config_file, false, false);
    }

    #[test]
    fn test_build_gradle_java_package_in_sbuild_env() {
        setup();
        setup_build_env();

        let config_file = "examples/bookworm/java/hello-world-gradle/pkg-builder.toml".to_string();
        test_builds(&config_file, false, false);
    }

    // piuparts, these must be run as root, only run on CI

    #[test]
    #[ignore]
    fn test_build_virtual_package_in_sbuild_env_piuparts() {
        setup();
        let config_file = "examples/bookworm/virtual-package/pkg-builder.toml".to_string();
        test_builds(&config_file, true, false);
    }

    #[test]
    #[ignore]
    fn test_build_rust_package_in_sbuild_env_piuparts() {
        setup();

        let config_file = "examples/bookworm/rust/hello-world/pkg-builder.toml".to_string();
        test_builds(&config_file, true, false);
    }

    #[test]
    #[ignore]
    fn test_build_go_package_in_sbuild_env_piuparts() {
        setup();

        let config_file = "examples/bookworm/go/hello-world/pkg-builder.toml".to_string();
        test_builds(&config_file, true, false);
    }

    #[test]
    #[ignore]
    fn test_build_javascript_package_in_sbuild_env_piuparts() {
        setup();

        let config_file = "examples/bookworm/javascript/hello-world/pkg-builder.toml".to_string();
        test_builds(&config_file, true, false);
    }

    #[test]
    #[ignore]
    fn test_build_java_package_in_sbuild_env_piuparts() {
        setup();

        let config_file = "examples/bookworm/java/hello-world/pkg-builder.toml".to_string();
        test_builds(&config_file, true, false);
    }

    #[test]
    #[ignore]
    fn test_build_dotnet_package_in_sbuild_env_piuparts() {
        setup();
        let config_file = "examples/bookworm/dotnet/hello-world/pkg-builder.toml".to_string();
        test_builds(&config_file, true, false);
    }

    #[test]
    #[ignore]
    fn test_build_typescript_package_in_sbuild_env_piuparts() {
        setup();

        let config_file = "examples/bookworm/typescript/hello-world/pkg-builder.toml".to_string();
        test_builds(&config_file, true, false);
    }

    #[test]
    #[ignore]
    fn test_build_nim_package_in_sbuild_env_piuparts() {
        setup();

        let config_file = "examples/bookworm/nim/hello-world/pkg-builder.toml".to_string();
        test_builds(&config_file, true, false);
    }

    #[test]
    #[ignore]
    fn test_build_gradle_java_package_in_sbuild_env_piuparts() {
        setup();

        let config_file = "examples/bookworm/java/hello-world-gradle/pkg-builder.toml".to_string();
        test_builds(&config_file, true, false);
    }

    // autopkgtest

    #[test]
    #[ignore]
    fn test_build_virtual_package_in_sbuild_env_autopkgtest() {
        // setup();
        // let config_file = "examples/bookworm/virtual-package/pkg-builder.toml".to_string();
        // test_builds(&config_file, false, true);
    }

    #[test]
    #[ignore]
    fn test_build_rust_package_in_sbuild_env_autopkgtest() {
        setup();

        let config_file = "examples/bookworm/rust/hello-world/pkg-builder.toml".to_string();
        test_builds(&config_file, false, true);
    }

    #[test]
    #[ignore]
    fn test_build_go_package_in_sbuild_env_autopkgtest() {
        setup();

        let config_file = "examples/bookworm/go/hello-world/pkg-builder.toml".to_string();
        test_builds(&config_file, false, true);
    }

    #[test]
    #[ignore]
    fn test_build_javascript_package_in_sbuild_env_autopkgtest() {
        setup();

        let config_file = "examples/bookworm/javascript/hello-world/pkg-builder.toml".to_string();
        test_builds(&config_file, false, true);
    }

    #[test]
    #[ignore]
    fn test_build_java_package_in_sbuild_env_autopkgtest() {
        setup();

        let config_file = "examples/bookworm/java/hello-world/pkg-builder.toml".to_string();
        test_builds(&config_file, false, true);
    }

    #[test]
    #[ignore]
    fn test_build_dotnet_package_in_sbuild_env_autopkgtest() {
        setup();
        let config_file = "examples/bookworm/dotnet/hello-world/pkg-builder.toml".to_string();
        test_builds(&config_file, false, true);
    }

    #[test]
    #[ignore]
    fn test_build_typescript_package_in_sbuild_env_autopkgtest() {
        setup();

        let config_file = "examples/bookworm/typescript/hello-world/pkg-builder.toml".to_string();
        test_builds(&config_file, false, true);
    }

    #[test]
    #[ignore]
    fn test_build_nim_package_in_sbuild_env_autopkgtest() {
        setup();

        let config_file = "examples/bookworm/nim/hello-world/pkg-builder.toml".to_string();
        test_builds(&config_file, false, true);
    }

    #[test]
    #[ignore]
    fn test_build_gradle_java_package_in_sbuild_env_autopkgtest() {
        setup();

        let config_file = "examples/bookworm/java/hello-world-gradle/pkg-builder.toml".to_string();
        test_builds(&config_file, false, true);
    }


    // verify 
    fn test_verify(config_file: &str, verify_config_file: &str) {
        setup_build_env();
        let mut config = get_config::<PkgConfig>(config_file.to_string()).expect("Could not read config_file");
        let work_dir = Path::new(BUILD_FILES_DIR).join("virtual");
        config.build_env.workdir = Some(work_dir.clone().to_str().unwrap().to_string());
        config.build_env.sbuild_cache_dir = Some(SBUILD_CACHE_DIR.to_string());
        let distribution = get_distribution(config.clone(), config_file.to_string())
            .expect("Could not get distribution");

        let config = get_config(verify_config_file.to_string()).expect("Could not read config_file");

        let result = distribution.verify(config);
        match result {
            Ok(_) => {
                assert!(result.is_ok());
            }
            Err(err) => {
              
                panic!("Could not verify: {}", err);
            }
        }
      
    }


    #[test]
    #[ignore]
    fn test_build_virtual_package_in_sbuild_env_verify() {
        setup();
        let config_file = "examples/bookworm/virtual-package/pkg-builder.toml".to_string();
        let verify_config_file = "examples/bookworm/virtual-package/pkg-builder-verify.toml".to_string();
        test_verify(&config_file, &verify_config_file);
    }

    #[test]
    #[ignore]
    fn test_build_rust_package_in_sbuild_env_verify() {
        setup();

        let config_file = "examples/bookworm/rust/hello-world/pkg-builder.toml".to_string();
        let verify_config_file = "examples/bookworm/rust/hello-world/pkg-builder-verify.toml".to_string();
        test_verify(&config_file, &verify_config_file);
    }

    #[test]
    #[ignore]
    fn test_build_go_package_in_sbuild_env_verify() {
        setup();

        let config_file = "examples/bookworm/go/hello-world/pkg-builder.toml".to_string();
        let verify_config_file = "examples/bookworm/go/hello-world/pkg-builder-verify.toml".to_string();
        test_verify(&config_file, &verify_config_file);
    }

    #[test]
    #[ignore]
    fn test_build_javascript_package_in_sbuild_env_verify() {
        setup();

        let config_file = "examples/bookworm/javascript/hello-world/pkg-builder.toml".to_string();
        let verify_config_file = "examples/bookworm/javascript/hello-world/pkg-builder-verify.toml".to_string();
        test_verify(&config_file, &verify_config_file);
    }

    #[test]
    #[ignore]
    fn test_build_java_package_in_sbuild_env_verify() {
        setup();

        let config_file = "examples/bookworm/java/hello-world/pkg-builder.toml".to_string();
        let verify_config_file = "examples/bookworm/java/hello-world/pkg-builder-verify.toml".to_string();
        test_verify(&config_file, &verify_config_file);
    }

    #[test]
    #[ignore]
    fn test_build_dotnet_package_in_sbuild_env_verify() {
        setup();
        let config_file = "examples/bookworm/dotnet/hello-world/pkg-builder.toml".to_string();
        let verify_config_file = "examples/bookworm/dotnet/hello-world/pkg-builder-verify.toml".to_string();
        test_verify(&config_file, &verify_config_file);
    }

    #[test]
    #[ignore]
    fn test_build_typescript_package_in_sbuild_env_verify() {
        setup();

        let config_file = "examples/bookworm/typescript/hello-world/pkg-builder.toml".to_string();
        let verify_config_file = "examples/bookworm/typescript/hello-world/pkg-builder-verify.toml".to_string();
        test_verify(&config_file, &verify_config_file);
    }

    #[test]
    #[ignore]
    fn test_build_nim_package_in_sbuild_env_verify() {
        setup();

        let config_file = "examples/bookworm/nim/hello-world/pkg-builder.toml".to_string();
        let verify_config_file = "examples/bookworm/nim/hello-world/pkg-builder-verify.toml".to_string();
        test_verify(&config_file, &verify_config_file);
    }

    #[test]
    #[ignore]
    fn test_build_gradle_java_package_in_sbuild_env_verify() {
        setup();

        let config_file = "examples/bookworm/java/hello-world-gradle/pkg-builder.toml".to_string();
        let verify_config_file = "examples/bookworm/java/hello-world-gradle/pkg-builder-verify.toml".to_string();
        test_verify(&config_file, &verify_config_file);
    }
}
