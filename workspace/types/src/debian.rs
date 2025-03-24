pub enum DebCommandPayload {
    Verify {
        verify_config: Option<String>,
        no_package: Option<bool>,
    },
    Lintian,
    Piuparts,
    Autopkgtest,
    Package {
        run_lintian: Option<bool>,
        run_piuparts: Option<bool>,
        run_autopkgtest: Option<bool>,
    },
    EnvCreate,
    EnvClean,
}
