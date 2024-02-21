use clap::{App, Arg};
use super::package_build::PackageBuild;
use super::config::PackageBuildConfig;

pub fn run_cli() {
    let matches = App::new("My Package Builder")
        .version("1.0")
        .author("Your Name <email@example.com>")
        .about("Builds and tests packages")
        .arg(Arg::with_name("arch")
            .long("arch")
            .value_name("ARCH")
            .help("Sets the target architecture")
            .takes_value(true)
            .default_value("amd64"))
        .arg(Arg::with_name("source_url")
            .long("source-url")
            .value_name("URL")
            .help("Sets the source URL")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("previous_build_hash")
            .long("prev-build-hash")
            .value_name("HASH")
            .help("Sets the hash of the previous build")
            .takes_value(true)
            .default_value(""))
        .arg(Arg::with_name("source_is_git")
            .long("source-is-git")
            .help("Indicates if the source is a git repository")
            .takes_value(false))
        .get_matches();

    let arch = matches.value_of("arch").unwrap().to_string();
    let source_url = matches.value_of("source_url").unwrap().to_string();
    let previous_build_hash = matches.value_of("previous_build_hash").unwrap().to_string();
    let source_is_git = matches.is_present("source_is_git");

    let config = PackageBuildConfig {
        arch: vec![arch],
        source_url,
        previous_build_hash,
        source_is_git,
    };

    let pkg_build = PackageBuild::new(config);

    match pkg_build.prepare() {
        Ok(_) => println!("Prepare successful"),
        Err(e) => println!("Error during preparation: {}", e),
    }
    match pkg_build.build_and_test() {
        Ok(_) => println!("Build and test successful"),
        Err(e) => println!("Error during build and test: {}", e),
    }
    match pkg_build.verify() {
        Ok(_) => println!("Verify successful"),
        Err(e) => println!("Error during verify: {}", e),
    }
}
