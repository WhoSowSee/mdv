use crate::config::Config;
use anyhow::{Context, Result, bail};
use serde_yaml::{Mapping, Value};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

const PRESETS_DIR: &str = "presets";
const BUILTIN_PRESETS: [(&str, &str); 3] = [
    (
        "compact",
        include_str!("../assets/config/presets/compact.yaml"),
    ),
    (
        "reader",
        include_str!("../assets/config/presets/reader.yaml"),
    ),
    (
        "showcase",
        include_str!("../assets/config/presets/showcase.yaml"),
    ),
];

enum PresetSource {
    BuiltIn,
    Custom,
}

struct Preset {
    source: PresetSource,
    settings: PresetFile,
}

struct PresetFile {
    name: String,
    settings: Mapping,
}

impl PresetFile {
    fn parse(contents: &str, source: &str) -> Result<Self> {
        let Value::Mapping(mut settings) = serde_yaml::from_str(contents)
            .with_context(|| format!("Failed to parse YAML preset: {source}"))?
        else {
            bail!("Preset must be a YAML mapping: {source}");
        };
        let name = settings
            .remove(Value::String("name".to_string()))
            .with_context(|| format!("Preset is missing a non-empty 'name' field: {source}"))?;
        let name = serde_yaml::from_value::<String>(name)
            .with_context(|| format!("Preset 'name' must be a string: {source}"))?;
        let name = name.trim().to_string();
        if name.is_empty() {
            bail!("Preset is missing a non-empty 'name' field: {source}");
        }

        let preset = Self { name, settings };
        preset.merged_with(&Config::default(), source)?;
        Ok(preset)
    }

    fn merged_with(&self, config: &Config, source: &str) -> Result<Config> {
        let Value::Mapping(mut merged) = serde_yaml::to_value(config)
            .with_context(|| format!("Failed to serialize configuration for preset: {source}"))?
        else {
            bail!("Failed to serialize configuration for preset: {source}");
        };

        for (key, value) in &self.settings {
            if !merged.contains_key(key) {
                bail!("Unknown preset setting {} in {source}", key_debug(key));
            }
            merged.insert(key.clone(), value.clone());
        }

        serde_yaml::from_value(Value::Mapping(merged))
            .with_context(|| format!("Failed to parse preset settings: {source}"))
    }

    fn apply_to(self, config: &mut Config) -> Result<()> {
        let mut merged = self.merged_with(config, &self.name)?;
        merged.config_file = config.config_file.clone();
        merged.config_dir = config.config_dir.clone();
        *config = merged;
        Ok(())
    }
}

fn key_debug(key: &Value) -> String {
    key.as_str()
        .map(|key| format!("'{key}'"))
        .unwrap_or_else(|| format!("{key:?}"))
}

fn load_presets(config_dir: Option<&Path>) -> Result<BTreeMap<String, Preset>> {
    let mut presets = BTreeMap::new();

    for (expected_name, contents) in BUILTIN_PRESETS {
        let settings = PresetFile::parse(contents, expected_name)?;
        if settings.name != expected_name {
            bail!(
                "Embedded preset '{}' declares the unexpected name '{}'",
                expected_name,
                settings.name
            );
        }
        presets.insert(
            settings.name.clone(),
            Preset {
                source: PresetSource::BuiltIn,
                settings,
            },
        );
    }

    let Some(config_dir) = config_dir else {
        return Ok(presets);
    };
    let presets_dir = config_dir.join(PRESETS_DIR);
    if !presets_dir.exists() {
        return Ok(presets);
    }
    if !presets_dir.is_dir() {
        bail!(
            "Expected '{}' to be a directory, found a file: {}",
            PRESETS_DIR,
            presets_dir.display()
        );
    }

    let mut paths = Vec::new();
    for entry in fs::read_dir(&presets_dir).with_context(|| {
        format!(
            "Failed to read presets directory: {}",
            presets_dir.display()
        )
    })? {
        let path = entry
            .with_context(|| format!("Failed to read entry in {}", presets_dir.display()))?
            .path();
        if path.is_file()
            && path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| matches!(extension, "yaml" | "yml"))
        {
            paths.push(path);
        }
    }
    paths.sort();

    for path in paths {
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read preset file: {}", path.display()))?;
        let settings = PresetFile::parse(&contents, &path.display().to_string())?;
        if matches!(
            presets.get(&settings.name).map(|preset| &preset.source),
            Some(PresetSource::Custom)
        ) {
            bail!(
                "Duplicate user preset name '{}' in {}",
                settings.name,
                path.display()
            );
        }
        presets.insert(
            settings.name.clone(),
            Preset {
                source: PresetSource::Custom,
                settings,
            },
        );
    }

    Ok(presets)
}

pub(crate) fn apply_named_preset(config: &mut Config, name: &str) -> Result<()> {
    let mut presets = load_presets(config.config_dir.as_deref())?;
    let available = presets.keys().cloned().collect::<Vec<_>>().join(", ");
    let preset = presets.remove(name).ok_or_else(|| {
        anyhow::anyhow!("Unknown preset '{name}'. Available presets: {available}")
    })?;
    preset.settings.apply_to(config)
}

pub(crate) fn format_available_presets(
    config: &Config,
    active_name: Option<&str>,
) -> Result<String> {
    let presets = load_presets(config.config_dir.as_deref())?;
    let mut output = String::from("Available presets:\n\n");
    for (name, preset) in presets {
        let source = match preset.source {
            PresetSource::BuiltIn => "built-in",
            PresetSource::Custom => "custom",
        };
        let active = if active_name == Some(name.as_str()) {
            " [active]"
        } else {
            ""
        };
        writeln!(output, "  {name:<20} - {source}{active}")?;
    }
    Ok(output)
}
