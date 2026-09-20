use anyhow::Result;

#[cfg(windows)]
pub(crate) fn split_command(raw: &str) -> Result<Vec<String>> {
    validate_windows_quotes(raw)?;
    Ok(winsplit::split(raw))
}

#[cfg(not(windows))]
pub(crate) fn split_command(raw: &str) -> Result<Vec<String>> {
    shell_words::split(raw).map_err(Into::into)
}

#[cfg(windows)]
fn validate_windows_quotes(raw: &str) -> Result<()> {
    let mut in_quotes = false;
    let mut backslashes = 0;

    for character in raw.chars() {
        match character {
            '\\' => backslashes += 1,
            '"' => {
                if backslashes % 2 == 0 {
                    in_quotes = !in_quotes;
                }
                backslashes = 0;
            }
            _ => backslashes = 0,
        }
    }

    if in_quotes {
        anyhow::bail!("unclosed double quote in command")
    }
    Ok(())
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn rejects_unclosed_double_quotes() {
        assert!(split_command("pager \"builtin").is_err());
        assert!(split_command("pager \\\"builtin").is_ok());
    }
}
