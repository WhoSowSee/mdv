use assert_cmd::Command;
use std::sync::OnceLock;
use tempfile::TempDir;

pub(crate) fn mdv_cmd() -> Command {
    static CONFIG_DIR: OnceLock<TempDir> = OnceLock::new();
    let config_dir = CONFIG_DIR.get_or_init(|| TempDir::new().expect("isolated config directory"));
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("mdv"));
    command
        .env_remove("MDV_COLOR")
        .env_remove("MDV_PAGER")
        .env_remove("TMUX")
        .env_remove("STY")
        .env_remove("TERM_PROGRAM")
        .env("TERM", "xterm-256color")
        .env("COLORTERM", "truecolor")
        .env("MDV_CONFIG_PATH", config_dir.path());
    command
}
