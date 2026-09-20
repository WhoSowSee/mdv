use super::*;

#[test]
fn environment_config_path_is_used() {
    let _env_lock = env_lock();
    let temp_dir = TempDir::new().expect("create temp dir");
    let config_path = temp_dir.path().join("config.yaml");
    std::fs::write(&config_path, "color: never\n").expect("write config file");

    let _guard = EnvVarGuard::set_temp(CONFIG_FILE_ENV, temp_dir.path().as_os_str());
    let (cli, matches) = parse_cli_from(vec![OsString::from("mdv")]);

    let config = Config::from_cli(&cli, &matches).expect("load config from env");
    assert_eq!(config.color, ColorMode::Never);
    assert_eq!(
        config.config_file.as_deref(),
        Some(config_path.as_path()),
        "config should record loaded path"
    );
}

#[test]
fn arg_has_user_value_detects_command_line_sources() {
    let matches = Cli::command().get_matches_from(vec![
        OsString::from("mdv"),
        OsString::from("--wrap"),
        OsString::from("none"),
    ]);

    assert!(arg_has_user_value(&matches, "wrap_mode"));
}

#[test]
fn arg_has_user_value_ignores_default_values() {
    let matches = Command::new("mdv-test")
        .arg(Arg::new("color").default_value("auto"))
        .get_matches_from(vec!["mdv-test"]);

    assert!(!arg_has_user_value(&matches, "color"));
}

#[test]
fn config_file_rejects_existing_file_path() {
    let _env_lock = env_lock();
    let temp_dir = TempDir::new().expect("create temp dir");
    let config_path = temp_dir.path().join("config.yaml");
    std::fs::write(&config_path, "color: never\n").expect("write config file");

    let (cli, matches) = parse_cli_from(vec![
        OsString::from("mdv"),
        OsString::from("--config-file"),
        config_path.as_os_str().to_owned(),
    ]);

    let error = Config::from_cli(&cli, &matches).expect_err("file path must fail");
    assert!(
        error.to_string().contains("must be a directory"),
        "error: {error}"
    );
}

#[test]
fn expand_tilde_replaces_leading_tilde_with_home_dir() {
    let home = dirs::home_dir().expect("home directory");
    assert_eq!(expand_tilde(Path::new("~")), home);
    assert_eq!(
        expand_tilde(Path::new("~/.config/mdv")),
        home.join(".config").join("mdv")
    );
    assert_eq!(
        expand_tilde(Path::new("not/tilde/path")),
        Path::new("not/tilde/path")
    );
}
