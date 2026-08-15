use super::*;

/// Load every `*.yaml`/`*.yml` file from `<config_dir>/themes/`. Files are
/// processed in lexical order, so a later file can `extends:` any earlier
/// one. Parse and resolution errors are logged and skipped, not propagated.
pub fn load_user_themes(config_dir: &Path, manager: &ThemeManager) -> Result<Vec<Theme>> {
    let themes_dir = config_dir.join(THEMES_DIR);

    if !themes_dir.exists() {
        return Ok(Vec::new());
    }
    if !themes_dir.is_dir() {
        bail!(
            "Expected '{}' to be a directory, found a file: {}",
            THEMES_DIR,
            themes_dir.display()
        );
    }

    let mut paths: Vec<PathBuf> = fs::read_dir(&themes_dir)
        .with_context(|| format!("Failed to read themes directory: {}", themes_dir.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|s| s.to_str())
                    .is_some_and(|ext| ext == THEME_EXT_YAML || ext == THEME_EXT_YML)
        })
        .collect();
    paths.sort();

    let mut loaded: Vec<Theme> = Vec::with_capacity(paths.len());
    for path in paths {
        match load_one_theme(&path, &loaded, manager) {
            Ok(theme) => loaded.push(theme),
            Err(err) => {
                log::warn!(
                    "Skipping theme file '{}': {}",
                    path.display(),
                    format_error_chain(&err)
                );
            }
        }
    }

    Ok(loaded)
}

fn load_one_theme(path: &Path, already_loaded: &[Theme], manager: &ThemeManager) -> Result<Theme> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read theme file: {}", path.display()))?;
    let file: ThemeFile = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse YAML theme file: {}", path.display()))?;

    if file.name.trim().is_empty() {
        bail!("Theme file is missing 'name' field");
    }

    let base = match file
        .extends
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(parent_name) => {
            find_base_theme(parent_name, already_loaded, manager).with_context(|| {
                format!(
                    "Theme '{}' extends unknown theme '{}'",
                    file.name, parent_name
                )
            })?
        }
        None => Theme::default(),
    };

    Ok(file.resolve(&base))
}

fn find_base_theme(name: &str, already_loaded: &[Theme], manager: &ThemeManager) -> Result<Theme> {
    if let Ok(theme) = manager.get_theme(name) {
        return Ok(theme.clone());
    }
    if let Some(theme) = already_loaded
        .iter()
        .find(|theme| theme.name.eq_ignore_ascii_case(name))
    {
        return Ok(theme.clone());
    }
    bail!("unknown theme '{}'", name)
}

fn format_error_chain(err: &anyhow::Error) -> String {
    err.chain()
        .map(|cause| cause.to_string())
        .collect::<Vec<_>>()
        .join(": ")
}
