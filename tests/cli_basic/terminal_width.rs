use crate::support::mdv_process_command;
use assert_cmd::Command;
use std::os::unix::{fs::PermissionsExt, process::CommandExt};
use std::{env, fs, io, time::Duration};
use tempfile::TempDir;

#[test]
fn terminal_size_queries_do_not_grow_with_document_length() {
    let directory = TempDir::new().unwrap();
    let tput = directory.path().join("tput");
    let calls = directory.path().join("calls");
    fs::write(
        &tput,
        r#"#!/bin/sh
printf '%s\n' "$1" >> "$MDV_TEST_TPUT_LOG"
case "$1" in
    cols) printf '80\n' ;;
    lines) printf '24\n' ;;
esac
"#,
    )
    .unwrap();
    fs::set_permissions(&tput, fs::Permissions::from_mode(0o755)).unwrap();
    let original_path = env::var_os("PATH").unwrap();
    let path = env::join_paths(
        std::iter::once(directory.path().to_path_buf()).chain(env::split_paths(&original_path)),
    )
    .unwrap();

    let queries = [1, 64].map(|count| {
        fs::write(&calls, "").unwrap();
        let mut process = mdv_process_command();
        process.env("PATH", &path).env("MDV_TEST_TPUT_LOG", &calls);
        // SAFETY: setsid is async-signal-safe and accesses no shared Rust state.
        unsafe {
            process.pre_exec(|| {
                if libc::setsid() == -1 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let output = Command::from_std(process)
            .args(["-", "--color", "never", "--no-code-guessing"])
            .write_stdin(format!(
                "{}\nTail sentinel\n",
                "{% hint style=\"info\" %}\n".repeat(count)
            ))
            .timeout(Duration::from_secs(10))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("Tail sentinel"));
        fs::read_to_string(&calls)
            .unwrap()
            .lines()
            .filter(|argument| *argument == "cols")
            .count()
    });

    assert!(
        queries[0] > 0,
        "terminal size detection must use the test tput"
    );
    assert_eq!(
        queries[0], queries[1],
        "terminal size queries must be bounded"
    );
}
