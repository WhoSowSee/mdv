use std::io::Write;
use std::process::{Command, Stdio};

struct PagerCommand {
    command: String,
    args: Vec<String>,
}

/// Write the rendered output to a pager if one is available, otherwise print it.
pub fn page_or_print(output: &str) -> std::io::Result<()> {
    if let Some(pager) = detect_pager() {
        match Command::new(&pager.command)
            .args(&pager.args)
            .stdin(Stdio::piped())
            .spawn()
        {
            Ok(mut child) => {
                if let Some(mut stdin) = child.stdin.take() {
                    stdin.write_all(output.as_bytes())?;
                }
                child.wait()?;
                return Ok(());
            }
            Err(err) => {
                log::warn!("Failed to start pager '{}': {}", pager.command, err);
            }
        }
    }

    print!("{}", output);
    Ok(())
}

fn detect_pager() -> Option<PagerCommand> {
    if let Ok(pager) = std::env::var("MDV_PAGER")
        && !pager.is_empty()
    {
        return parse_pager_command(&pager);
    }

    if let Ok(pager) = std::env::var("PAGER")
        && !pager.is_empty()
    {
        return parse_pager_command(&pager);
    }

    Some(PagerCommand {
        command: "less".to_string(),
        args: vec!["-R".to_string(), "-F".to_string(), "-K".to_string()],
    })
}

fn parse_pager_command(raw: &str) -> Option<PagerCommand> {
    let mut parts = raw.split_whitespace().map(String::from).peekable();
    let command = parts.next()?;
    let args: Vec<String> = parts.collect();

    if command == "less" && args.is_empty() {
        Some(PagerCommand {
            command,
            args: vec!["-R".to_string(), "-F".to_string(), "-K".to_string()],
        })
    } else {
        Some(PagerCommand { command, args })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn parse_pager_command_splits_command_and_args() {
        let pager = parse_pager_command("less -R -F").unwrap();
        assert_eq!(pager.command, "less");
        assert_eq!(pager.args, vec!["-R", "-F"]);
    }

    #[test]
    fn detect_pager_prefers_mdv_pager_over_pager() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("MDV_PAGER", "cat");
            std::env::set_var("PAGER", "less");
        }
        let pager = detect_pager().unwrap();
        assert_eq!(pager.command, "cat");
        assert!(pager.args.is_empty());
        unsafe {
            std::env::remove_var("MDV_PAGER");
            std::env::remove_var("PAGER");
        }
    }
}
