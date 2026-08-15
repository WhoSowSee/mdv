use super::*;

#[test]
fn test_text_highlight_background() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "Normal ==highlighted text== end.").unwrap();

    let output = mdv_cmd()
        .arg("-c")
        .arg("80")
        .arg(temp_file.path())
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let clean = strip_ansi(&stdout);
    assert!(clean.contains("highlighted text"));
    assert!(!clean.contains("==highlighted text=="));
    assert!(stdout.contains("\u{1b}[48;"));
}

#[test]
fn test_init_config_creates_config_file_from_path() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    let output = mdv_cmd()
        .arg("--init-config")
        .arg(temp_dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(config_path.exists());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(&config_path.display().to_string()));

    let config = fs::read_to_string(&config_path).unwrap();
    assert!(config.contains("theme: \"terminal\""));
    assert!(config.contains("link_style: \"clickable\""));
}

#[test]
fn test_init_config_creates_config_file_in_current_directory() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    let output = mdv_cmd()
        .arg("--init-config")
        .arg(".")
        .current_dir(&temp_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(config_path.exists());
}

#[test]
fn test_init_config_creates_config_file_from_config_file_arg() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("nested").join("config.yaml");

    let output = mdv_cmd()
        .arg("--init-config")
        .arg("--config-file")
        .arg(temp_dir.path().join("nested"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(config_path.exists());
}

#[test]
fn test_init_config_refuses_to_overwrite_existing_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    fs::write(&config_path, "theme: \"monokai\"\n").unwrap();

    let output = mdv_cmd()
        .arg("--init-config")
        .arg(temp_dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("Config file already exists"),
        "stderr:\n{}",
        stderr
    );
    assert_eq!(
        fs::read_to_string(&config_path).unwrap(),
        "theme: \"monokai\"\n"
    );
}

#[test]
fn test_init_config_uses_env_config_path() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    let output = mdv_cmd()
        .arg("--init-config")
        .env("MDV_CONFIG_PATH", temp_dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(config_path.exists());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(&config_path.display().to_string()));
}

#[test]
fn test_init_config_positional_path_overrides_config_file_arg() {
    let temp_dir = TempDir::new().unwrap();
    let config_file_path = temp_dir.path().join("config-file");
    let positional_path = temp_dir.path().join("positional");
    let positional_config = positional_path.join("config.yaml");

    let output = mdv_cmd()
        .arg("--init-config")
        .arg(&positional_path)
        .arg("--config-file")
        .arg(&config_file_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(positional_config.exists());
    assert!(!config_file_path.join("config.yaml").exists());
}

#[test]
fn test_pager_mode_prints_to_stdout_when_output_is_not_a_terminal() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "# Pager output\n").unwrap();

    let output = mdv_cmd()
        .arg("--pager")
        .arg(temp_file.path())
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let clean = strip_ansi(&stdout);
    assert!(clean.contains("Pager output"), "stdout:\n{}", stdout);
}

#[test]
fn test_interactive_requires_terminal_output() {
    let mut cmd = mdv_cmd();
    cmd.arg("--interactive")
        .write_stdin("# Input")
        .timeout(Duration::from_secs(2));
    cmd.assert().failure().stderr(predicate::str::contains(
        "interactive mode requires a terminal",
    ));
}
