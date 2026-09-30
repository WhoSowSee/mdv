use crate::support::mdv_process_command;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tempfile::TempDir;

fn render_command(path: &Path, args: &[&str]) -> Command {
    let mut command = mdv_process_command();
    command
        .args(["--color", "never", "--cols", "80"])
        .args(args)
        .arg(path);
    command
}

struct MonitorProcess {
    child: Child,
    lines: Receiver<String>,
    reader: Option<JoinHandle<()>>,
    output: String,
}

impl MonitorProcess {
    fn start(mut command: Command) -> Self {
        command.arg("--monitor");
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let (sender, lines) = mpsc::channel();
        let reader = thread::spawn(move || {
            let mut stdout = BufReader::new(stdout);
            let mut line = String::new();
            while stdout.read_line(&mut line).unwrap() != 0 {
                if sender.send(std::mem::take(&mut line)).is_err() {
                    break;
                }
            }
        });
        let mut monitor = Self {
            child,
            lines,
            reader: Some(reader),
            output: String::new(),
        };
        monitor.wait_for("Monitoring file:");
        monitor
    }

    fn wait_for(&mut self, marker: &str) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !self.output.contains(marker) {
            let line = self
                .lines
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .unwrap_or_else(|error| {
                    panic!(
                        "monitor did not print {marker:?}: {error}; stdout: {:?}",
                        self.output
                    )
                });
            self.output.push_str(&line);
        }
    }
}

impl Drop for MonitorProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

#[test]
fn monitor_preserves_document_options_on_reload() {
    for (html, initial, updated) in [
        (
            false,
            "title: Initial properties\n\nINITIAL_BODY\n",
            "title: Updated properties\n\nUPDATED_BODY\n",
        ),
        (
            true,
            "<p class=\"front-matter-plain\"><strong>title:</strong> Initial properties</p>\n<p>INITIAL_BODY</p>\n",
            "<p class=\"front-matter-plain\"><strong>title:</strong> Updated properties</p>\n<p>UPDATED_BODY</p>\n",
        ),
    ] {
        let directory = TempDir::new().unwrap();
        let path = directory.path().join("monitor.md");
        fs::write(
            &path,
            "\u{feff}---\ntitle: Initial properties\n---\n\nINITIAL_BODY\n",
        )
        .unwrap();
        let mut command = render_command(&path, &["--front-matter", "plain"]);
        if html {
            command.arg("--html");
        }
        let mut monitor = MonitorProcess::start(command);
        assert_eq!(
            monitor.output.split("Monitoring file:").next().unwrap(),
            initial,
            "html={html}"
        );

        fs::write(
            &path,
            "\u{feff}---\ntitle: Updated properties\n---\n\nUPDATED_BODY\n",
        )
        .unwrap();
        monitor.wait_for("UPDATED_BODY");
        assert_eq!(
            monitor
                .output
                .rsplit_once("\n--- File changed, re-rendering ---\n\n")
                .unwrap()
                .1,
            updated,
            "html={html}"
        );
        assert_eq!(monitor.output.matches("INITIAL_BODY").count(), 1);
    }
}

#[test]
fn monitor_reuses_the_prepared_theme_on_reload() {
    let directory = TempDir::new().unwrap();
    let themes = directory.path().join("themes");
    fs::create_dir(&themes).unwrap();
    let theme = themes.join("monitor-test.yaml");
    fs::write(&theme, "name: monitor-test\ntext: '#010203'\n").unwrap();
    fs::write(
        directory.path().join("config.yaml"),
        "theme: monitor-test\n",
    )
    .unwrap();
    let path = directory.path().join("prepared-theme.md");
    fs::write(&path, "INITIAL_THEME\n").unwrap();
    let mut command = render_command(&path, &[]);
    command.arg("--config-file").arg(directory.path());
    let mut monitor = MonitorProcess::start(command);

    fs::remove_file(&theme).unwrap();
    fs::write(&path, "UPDATED_THEME\n").unwrap();
    monitor.wait_for("UPDATED_THEME\n");
}
