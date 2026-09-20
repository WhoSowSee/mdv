use super::interrupt::{InterruptGuard, configure_child_interrupt};
use crate::process_command::split_command;
use anyhow::{Context, Result, anyhow, bail};
use std::ffi::OsString;
use std::io::{ErrorKind, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Mutex;

const PAGER_ENV: &str = "MDV_PAGER";
const PAGER_ACTIVE_ENV: &str = "_MDV_PAGER_ACTIVE";
const PAGER_EXPECTATION: &str = "'builtin', 'default', or a pager command";
static EXTERNAL_PAGER_LOCK: Mutex<()> = Mutex::new(());

pub(crate) enum PagerBackend {
    Builtin,
    External(ExternalPagerCommand),
}

pub(crate) struct ExternalPagerCommand {
    program: String,
    args: Vec<String>,
}

#[derive(Clone, Copy)]
enum PagerSource {
    Environment,
    CommandLine,
}

impl PagerBackend {
    pub(crate) const fn is_builtin(&self) -> bool {
        matches!(self, Self::Builtin)
    }

    pub(crate) fn resolve(cli_value: Option<&str>) -> Result<Self> {
        let env_value = std::env::var_os(PAGER_ENV)
            .map(unicode_environment_value)
            .transpose()?;
        Self::from_values(env_value.as_deref(), cli_value)
    }

    fn from_values(env_value: Option<&str>, cli_value: Option<&str>) -> Result<Self> {
        let mut backend = match env_value {
            Some(value) => Self::parse(value, PagerSource::Environment)?,
            None => Self::Builtin,
        };
        if let Some(value) = cli_value {
            backend = Self::parse(value, PagerSource::CommandLine)?;
        }
        Ok(backend)
    }

    fn parse(raw: &str, source: PagerSource) -> Result<Self> {
        let parts = split_command(raw).with_context(|| source.invalid_value(raw))?;
        let Some((program, args)) = parts.split_first() else {
            bail!(source.invalid_value(raw));
        };

        if matches!(program.as_str(), "builtin" | "default") {
            if args.is_empty() {
                return Ok(Self::Builtin);
            }
            bail!(
                "Invalid pager command '{raw}' from {}; '{program}' does not accept arguments",
                source.description(),
            );
        }
        if invokes_mdv(program) {
            bail!(
                "Invalid pager command '{raw}' from {}; using mdv as its own pager would recurse",
                source.description()
            );
        }

        Ok(Self::External(ExternalPagerCommand {
            program: program.clone(),
            args: args.to_vec(),
        }))
    }
}

impl ExternalPagerCommand {
    pub(super) fn page(&self, output: &str) -> Result<()> {
        if std::env::var_os(PAGER_ACTIVE_ENV).is_some() {
            bail!("Refusing to start an external pager recursively");
        }
        let _pager_lock = EXTERNAL_PAGER_LOCK
            .lock()
            .map_err(|_| anyhow!("External pager process lock poisoned"))?;
        let _interrupt_guard = InterruptGuard::install()
            .with_context(|| "Failed to prepare interrupt handling for pager")?;
        let mut command = Command::new(&self.program);
        command
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .env(PAGER_ACTIVE_ENV, "1");
        configure_child_interrupt(&mut command);
        let mut child = command
            .spawn()
            .with_context(|| format!("Failed to start pager '{}'", self.program))?;
        let mut input = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("Failed to open stdin for pager '{}'", self.program))?;

        if let Err(error) = input.write_all(output.as_bytes())
            && error.kind() != ErrorKind::BrokenPipe
        {
            drop(input);
            let _ = child.kill();
            let _ = child.wait();
            return Err(error)
                .with_context(|| format!("Failed to write to pager '{}'", self.program));
        }
        drop(input);

        let status = child
            .wait()
            .with_context(|| format!("Failed to wait for pager '{}'", self.program))?;
        if !status.success() {
            bail!("Pager '{}' exited with status {status}", self.program);
        }
        Ok(())
    }
}

impl PagerSource {
    const fn description(self) -> &'static str {
        match self {
            Self::Environment => "environment variable MDV_PAGER",
            Self::CommandLine => "--pager",
        }
    }

    fn invalid_value(self, raw: &str) -> String {
        format!(
            "Invalid value '{raw}' for {}; expected {PAGER_EXPECTATION}",
            self.description(),
        )
    }
}

fn unicode_environment_value(value: OsString) -> Result<String> {
    value.into_string().map_err(|_| {
        anyhow!(
            "Environment variable {PAGER_ENV} is not valid Unicode; expected {PAGER_EXPECTATION}"
        )
    })
}

fn invokes_mdv(program: &str) -> bool {
    let program_name = executable_name(program);
    if executable_names_equal(&program_name, env!("CARGO_PKG_NAME")) {
        return true;
    }

    std::env::current_exe()
        .ok()
        .map(|path| executable_name(&path.to_string_lossy()))
        .is_some_and(|current| executable_names_equal(&program_name, &current))
}

fn executable_name(program: &str) -> String {
    let name = Path::new(program)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    if cfg!(windows) {
        let lowercase = name.to_ascii_lowercase();
        let extension = [".exe", ".cmd", ".bat", ".com"]
            .into_iter()
            .find(|extension| lowercase.ends_with(extension));
        extension
            .map(|extension| name[..name.len() - extension.len()].to_string())
            .unwrap_or(name)
    } else {
        name
    }
}

fn executable_names_equal(left: &str, right: &str) -> bool {
    if cfg!(windows) {
        left.eq_ignore_ascii_case(right)
    } else {
        left == right
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn pager_precedence_is_cli_then_environment_then_builtin() {
        for (env_value, cli_value) in [
            (None, None),
            (Some("less -R"), Some("builtin")),
            (Some("less -R"), Some("default")),
            (Some("default"), None),
        ] {
            assert!(matches!(
                PagerBackend::from_values(env_value, cli_value).unwrap(),
                PagerBackend::Builtin
            ));
        }

        let PagerBackend::External(command) =
            PagerBackend::from_values(Some("less -R"), None).unwrap()
        else {
            panic!("expected external pager");
        };
        assert_eq!(command.program, "less");
        assert_eq!(command.args, ["-R"]);
    }

    #[test]
    fn invalid_pager_values_are_rejected() {
        for value in ["", "   ", "builtin --flag", "default --flag", "mdv"] {
            assert!(
                PagerBackend::from_values(Some(value), None).is_err(),
                "accepted {value:?}"
            );
        }
    }

    #[test]
    fn dotted_program_names_are_not_treated_as_mdv() {
        assert!(!invokes_mdv("mdv.wrapper"));
    }

    #[cfg(windows)]
    #[test]
    fn windows_launcher_extensions_are_case_insensitive() {
        for program in ["mdv.exe", "MDV.EXE", "mdv.CmD", r"C:\\Tools\\MDV.BAT"] {
            assert!(invokes_mdv(program), "program: {program}");
        }
    }

    #[cfg(not(windows))]
    #[test]
    fn malformed_pager_command_is_rejected() {
        assert!(PagerBackend::from_values(Some("less '"), None).is_err());
    }

    #[test]
    fn external_pager_receives_rendered_output() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("pager-output.txt");
        let command = capture_command(&output_path);

        command.page("pager payload\n").unwrap();

        let captured = std::fs::read_to_string(output_path).unwrap();
        assert!(captured.contains("pager payload"), "captured: {captured:?}");
    }

    #[test]
    fn external_pager_failure_is_reported() {
        let error = failing_command().page("pager payload\n").unwrap_err();

        assert!(
            error.to_string().contains("exited with status"),
            "error: {error:#}"
        );
    }

    #[cfg(windows)]
    fn capture_command(output_path: &Path) -> ExternalPagerCommand {
        let output_path = output_path.display().to_string().replace('\'', "''");
        ExternalPagerCommand {
            program: "powershell.exe".to_string(),
            args: vec![
                "-NoProfile".to_string(),
                "-NonInteractive".to_string(),
                "-Command".to_string(),
                format!(
                    "if ($env:_MDV_PAGER_ACTIVE -ne '1') {{ exit 9 }}; $content = [Console]::In.ReadToEnd(); [IO.File]::WriteAllText('{output_path}', $content)"
                ),
            ],
        }
    }

    #[cfg(not(windows))]
    fn capture_command(output_path: &Path) -> ExternalPagerCommand {
        ExternalPagerCommand {
            program: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                "test \"$_MDV_PAGER_ACTIVE\" = 1 && cat > \"$1\"".to_string(),
                "sh".to_string(),
                output_path.display().to_string(),
            ],
        }
    }

    #[cfg(windows)]
    fn failing_command() -> ExternalPagerCommand {
        ExternalPagerCommand {
            program: "cmd.exe".to_string(),
            args: vec!["/D".to_string(), "/C".to_string(), "exit /B 7".to_string()],
        }
    }

    #[cfg(not(windows))]
    fn failing_command() -> ExternalPagerCommand {
        ExternalPagerCommand {
            program: "sh".to_string(),
            args: vec!["-c".to_string(), "exit 7".to_string()],
        }
    }
}
