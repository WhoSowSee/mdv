use std::env;
use std::error::Error;
use std::path::Path;
use std::process::Command;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=build/version.rs");

    let mut version = env::var("CARGO_PKG_VERSION")?;
    if Path::new(".git").try_exists()? {
        track_git_head()?;
        let revision = command_output(Command::new("git").args([
            "log",
            "-1",
            "--format=%h %cs",
            "--abbrev=7",
            "HEAD",
        ]))?;
        version.push_str(&format!(" ({revision})"));
    }
    println!("cargo::rustc-env=MDV_BUILD_VERSION={version}");
    println!("cargo::rustc-env=MDV_BUILD_TARGET={}", env::var("TARGET")?);

    let rustc = command_output(
        Command::new(env::var_os("RUSTC").ok_or("RUSTC is not set")?).arg("--version"),
    )?;
    let rustc = rustc
        .strip_prefix("rustc ")
        .ok_or("unexpected rustc version output")?;
    println!("cargo::rustc-env=MDV_BUILD_RUSTC={rustc}");
    Ok(())
}

fn track_git_head() -> Result<(), Box<dyn Error>> {
    if Path::new(".git").is_file() {
        println!("cargo::rerun-if-changed=.git");
    }
    let paths = command_output(Command::new("git").args([
        "rev-parse",
        "--git-path",
        "HEAD",
        "--git-path",
        "refs",
        "--git-path",
        "packed-refs",
    ]))?;
    for path in paths.lines() {
        if Path::new(path).try_exists()? {
            println!("cargo::rerun-if-changed={path}");
        }
    }

    Ok(())
}

fn command_output(command: &mut Command) -> Result<String, Box<dyn Error>> {
    let output = command.output()?;
    if !output.status.success() {
        return Err(format!(
            "{command:?} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    let output = String::from_utf8(output.stdout)?.trim().to_owned();
    if output.is_empty() {
        return Err(format!("{command:?} returned empty output").into());
    }
    Ok(output)
}
