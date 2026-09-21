use super::*;
use crate::cli::ColorDepth;

#[test]
fn color_schema_is_strict_and_runtime_style_is_not_serialized() {
    for (name, mode) in [
        ("auto", ColorMode::Auto),
        ("always", ColorMode::Always),
        ("never", ColorMode::Never),
    ] {
        let config: Config = serde_yaml::from_str(&format!("color: {name}\n")).unwrap();
        assert_eq!(config.color, mode);
        let serialized = serde_yaml::to_value(config).unwrap();
        assert_eq!(serialized["color"].as_str(), Some(name));
        assert!(serialized.get("no_colors").is_none());
        assert!(serialized.get("output_style").is_none());
    }
    for value in ["true", "false", "1", "null", "''", "AUTO", "' always '"] {
        assert!(serde_yaml::from_str::<Config>(&format!("color: {value}\n")).is_err());
    }
    assert!(serde_yaml::from_str::<Config>("color: auto\ncolor: never\n").is_err());
    assert_eq!(
        serde_yaml::from_str::<Config>("{}\n").unwrap().color,
        ColorMode::Auto
    );
    for value in ["16", "'16'"] {
        let config: Config = serde_yaml::from_str(&format!("color_depth: {value}\n")).unwrap();
        assert_eq!(config.color_depth, ColorDepth::Ansi16);
    }
    for value in ["8", "true", "null", "AUTO"] {
        assert!(serde_yaml::from_str::<Config>(&format!("color_depth: {value}\n")).is_err());
    }
}

#[test]
fn removed_color_key_is_rejected_without_rejecting_other_unknown_keys() {
    let decoder = serde_yaml::Deserializer::from_str("no_colors: false\n");
    assert!(Config::deserialize(decoder).is_err());
    for yaml in [
        "no_colors: false\n",
        "no_colors: null\n",
        "no_colors: {arbitrary: content}\n",
        "color: auto\nno_colors: true\n",
    ] {
        let error = serde_yaml::from_str::<Config>(yaml).unwrap_err();
        assert!(error.to_string().contains(REMOVED_COLOR_SETTING));
        assert!(error.location().is_some());
    }
    assert!(serde_yaml::from_str::<Config>("unrelated_unknown_setting: true\n").is_ok());
    let config: Config = serde_yaml::from_str("cols: 42\ncustom_theme: {text: red}\n").unwrap();
    assert_eq!(config.cols, Some(42));
}

#[test]
fn color_layers_preserve_exact_modes_and_explicit_auto() {
    use ColorMode::{Always, Auto, Never};
    let _environment = env_lock();
    let config_dir = TempDir::new().unwrap();
    std::fs::write(config_dir.path().join("config.yaml"), "color: never\n").unwrap();
    for (name, setting) in [
        ("inherit", ""),
        ("auto", "color: auto\n"),
        ("plain", "color: never\n"),
        ("painted", "color: always\n"),
    ] {
        write_preset(
            config_dir.path(),
            &format!("{name}.yaml"),
            &format!("name: {name}\n{setting}"),
        );
    }
    for (preset, env, cli, expected) in [
        (None, None, None, Never),
        (Some("inherit"), None, None, Never),
        (Some("auto"), None, None, Auto),
        (Some("painted"), None, None, Always),
        (Some("plain"), Some("always"), None, Always),
        (Some("painted"), Some("auto"), None, Auto),
        (None, Some("auto"), None, Auto),
        (Some("painted"), Some("never"), None, Never),
        (Some("plain"), Some("always"), Some("never"), Never),
        (Some("plain"), Some("never"), Some("always"), Always),
        (None, Some("always"), Some("auto"), Auto),
    ] {
        let _env = env.map(|value| EnvVarGuard::set_temp(COLOR_ENV, value));
        let mut args = vec![
            "mdv".into(),
            "--config-file".into(),
            config_dir.path().as_os_str().to_owned(),
        ];
        if let Some(preset) = preset {
            args.extend(["--preset".into(), preset.into()]);
        }
        if let Some(cli) = cli {
            args.extend(["--color".into(), cli.into()]);
        }
        let (cli, matches) = parse_cli_from(args);
        assert_eq!(Config::from_cli(&cli, &matches).unwrap().color, expected);
    }
}

#[cfg(any(unix, windows))]
#[test]
fn invalid_environment_color_is_an_error() {
    let _environment = env_lock();
    for value in ["", "AUTO", " always", "never ", "true", "invalid"] {
        let _guard = EnvVarGuard::set_temp(COLOR_ENV, value);
        assert!(mdv_color_override().is_err(), "accepted {value:?}");
    }
    #[cfg(windows)]
    let invalid = {
        use std::os::windows::ffi::OsStringExt;
        OsString::from_wide(&[0xd800])
    };
    #[cfg(unix)]
    let invalid = {
        use std::os::unix::ffi::OsStringExt;
        OsString::from_vec(vec![0xff])
    };
    let _guard = EnvVarGuard::set_temp(COLOR_ENV, invalid);
    assert!(
        mdv_color_override()
            .unwrap_err()
            .to_string()
            .contains("not valid Unicode")
    );
}
