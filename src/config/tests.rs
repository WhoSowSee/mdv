use super::*;
use crate::cli::Cli;

use clap::{Arg, Command, CommandFactory, FromArgMatches};
use std::ffi::{OsStr, OsString};
use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;

struct EnvVarGuard {
    key: &'static str,
    original: Option<OsString>,
}

fn set_env_var<K, V>(key: K, value: V)
where
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    unsafe {
        std::env::set_var(key, value);
    }
}

fn remove_env_var<K>(key: K)
where
    K: AsRef<OsStr>,
{
    unsafe {
        std::env::remove_var(key);
    }
}

impl EnvVarGuard {
    fn set_temp<K>(key: &'static str, value: K) -> Self
    where
        K: AsRef<OsStr>,
    {
        let original = std::env::var_os(key);
        set_env_var(key, value);
        Self { key, original }
    }
}

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
    ENV_MUTEX
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("lock env mutex")
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(ref value) = self.original {
            set_env_var(self.key, value);
        } else {
            remove_env_var(self.key);
        }
    }
}

fn parse_cli_from(args: Vec<OsString>) -> (Cli, clap::ArgMatches) {
    let matches = Cli::command().get_matches_from(args);
    let cli = Cli::from_arg_matches(&matches).expect("parse cli from matches");
    (cli, matches)
}

fn parse_with_config(config_contents: &str) -> Config {
    let temp_dir = TempDir::new().expect("create temp dir");
    let config_path = temp_dir.path().join("config.yaml");
    std::fs::write(&config_path, config_contents).expect("write config file");

    let (cli, matches) = parse_cli_from(vec![
        OsString::from("mdv"),
        OsString::from("--config-file"),
        temp_dir.path().as_os_str().to_owned(),
    ]);

    Config::from_cli(&cli, &matches).expect("load config")
}

fn write_preset(config_dir: &std::path::Path, filename: &str, contents: &str) {
    let presets_dir = config_dir.join("presets");
    std::fs::create_dir_all(&presets_dir).expect("create presets dir");
    std::fs::write(presets_dir.join(filename), contents).expect("write preset file");
}

mod environment;
mod loading;
mod writing;
