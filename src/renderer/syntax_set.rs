use anyhow::{Context, Result, bail};
use std::path::Path;
use std::sync::{Arc, LazyLock};
use syntect::dumps::from_uncompressed_data;
use syntect::parsing::SyntaxSet;

const EMBEDDED_SYNTAX_SET: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/syntaxes.bin"));

/// Global cache of syntaxes to avoid unpacking them every time a renderer is created.
static SYNTAX_SET: LazyLock<Arc<SyntaxSet>> =
    LazyLock::new(|| Arc::new(load_embedded_syntax_set()));

fn load_embedded_syntax_set() -> SyntaxSet {
    from_uncompressed_data::<SyntaxSet>(EMBEDDED_SYNTAX_SET).unwrap_or_else(|err| {
        log::error!(
            "Failed to load the embedded syntax set: {err}. Falling back to syntect defaults."
        );
        SyntaxSet::load_defaults_newlines()
    })
}

/// Load the embedded syntax set, optionally extended with user syntaxes.
pub fn load_full_syntax_set(syntaxes_dir: Option<&Path>) -> Result<Arc<SyntaxSet>> {
    let Some(syntaxes_dir) = syntaxes_dir else {
        return Ok(Arc::clone(&SYNTAX_SET));
    };

    if !syntaxes_dir.exists() {
        bail!(
            "Syntaxes directory does not exist: {}",
            syntaxes_dir.display()
        );
    }
    if !syntaxes_dir.is_dir() {
        bail!(
            "Syntaxes path must be a directory: {}",
            syntaxes_dir.display()
        );
    }

    let mut builder = load_embedded_syntax_set().into_builder();
    builder
        .add_from_folder(syntaxes_dir, true)
        .with_context(|| {
            format!(
                "Failed to load custom syntaxes from {}",
                syntaxes_dir.display()
            )
        })?;

    Ok(Arc::new(builder.build()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn invalid_custom_syntax_is_rejected() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Broken.sublime-syntax"),
            "name: [broken",
        )
        .unwrap();

        let error =
            load_full_syntax_set(Some(temp_dir.path())).expect_err("invalid syntax must fail");

        assert!(error.to_string().contains("Failed to load custom syntaxes"));
    }
}
