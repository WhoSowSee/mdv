//! User-defined themes loaded from `<config_dir>/themes/*.yaml`.
use crate::theme::{parse_color_value, Color, SyntaxTheme, Theme, ThemeManager};
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

const THEMES_DIR: &str = "themes";
const THEME_EXT_YAML: &str = "yaml";
const THEME_EXT_YML: &str = "yml";

/// [`Color`] that accepts the same value formats as `--custom-theme`
/// (named, hex, rgb, ansi-index) and reuses [`parse_color_value`].
#[derive(Debug, Clone)]
pub(crate) struct ColorYaml(pub(crate) Color);

impl<'de> Deserialize<'de> for ColorYaml {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        parse_color_value(&raw).map(ColorYaml).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub(crate) struct ThemeFile {
    pub name: String,
    pub description: Option<String>,
    pub extends: Option<String>,

    pub text: Option<ColorYaml>,
    pub text_light: Option<ColorYaml>,
    pub h1: Option<ColorYaml>,
    pub h2: Option<ColorYaml>,
    pub h3: Option<ColorYaml>,
    pub h4: Option<ColorYaml>,
    pub h5: Option<ColorYaml>,
    pub h6: Option<ColorYaml>,
    pub code: Option<ColorYaml>,
    pub code_block: Option<ColorYaml>,
    pub quote: Option<ColorYaml>,
    pub link: Option<ColorYaml>,
    pub emphasis: Option<ColorYaml>,
    pub strong: Option<ColorYaml>,
    pub strikethrough: Option<ColorYaml>,
    pub highlight_background: Option<ColorYaml>,
    pub background: Option<ColorYaml>,
    pub border: Option<ColorYaml>,
    pub list_marker: Option<ColorYaml>,
    pub table_header: Option<ColorYaml>,
    pub table_border: Option<ColorYaml>,
    pub error: Option<ColorYaml>,
    pub warning: Option<ColorYaml>,

    pub syntax: Option<SyntaxFile>,
}
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub(crate) struct SyntaxFile {
    pub keyword: Option<ColorYaml>,
    pub string: Option<ColorYaml>,
    pub comment: Option<ColorYaml>,
    pub number: Option<ColorYaml>,
    pub operator: Option<ColorYaml>,
    pub function: Option<ColorYaml>,
    pub variable: Option<ColorYaml>,
    pub type_name: Option<ColorYaml>,
}

impl ThemeFile {
    /// Resolve this partial description against `base`, producing a fully
    /// populated [`Theme`]. Omitted fields inherit from `base`; specified
    /// fields override it.
    pub fn resolve(&self, base: &Theme) -> Theme {
        let pick = |override_color: &Option<ColorYaml>, base_color: &Color| -> Color {
            override_color
                .as_ref()
                .map(|c| c.0.clone())
                .unwrap_or_else(|| base_color.clone())
        };
        let pick_bg = |override_color: &Option<ColorYaml>, base_color: &Option<Color>| -> Option<Color> {
            override_color
                .as_ref()
                .map(|c| c.0.clone())
                .or_else(|| base_color.clone())
        };

        let syntax = match &self.syntax {
            Some(syntax_file) => SyntaxTheme {
                keyword: pick(&syntax_file.keyword, &base.syntax.keyword),
                string: pick(&syntax_file.string, &base.syntax.string),
                comment: pick(&syntax_file.comment, &base.syntax.comment),
                number: pick(&syntax_file.number, &base.syntax.number),
                operator: pick(&syntax_file.operator, &base.syntax.operator),
                function: pick(&syntax_file.function, &base.syntax.function),
                variable: pick(&syntax_file.variable, &base.syntax.variable),
                type_name: pick(&syntax_file.type_name, &base.syntax.type_name),
            },
            None => base.syntax.clone(),
        };

        Theme {
            name: self.name.clone(),
            description: self
                .description
                .clone()
                .unwrap_or_else(|| base.description.clone()),
            text: pick(&self.text, &base.text),
            text_light: pick(&self.text_light, &base.text_light),
            h1: pick(&self.h1, &base.h1),
            h2: pick(&self.h2, &base.h2),
            h3: pick(&self.h3, &base.h3),
            h4: pick(&self.h4, &base.h4),
            h5: pick(&self.h5, &base.h5),
            h6: pick(&self.h6, &base.h6),
            code: pick(&self.code, &base.code),
            code_block: pick(&self.code_block, &base.code_block),
            quote: pick(&self.quote, &base.quote),
            link: pick(&self.link, &base.link),
            emphasis: pick(&self.emphasis, &base.emphasis),
            strong: pick(&self.strong, &base.strong),
            strikethrough: pick(&self.strikethrough, &base.strikethrough),
            highlight_background: pick(&self.highlight_background, &base.highlight_background),
            background: pick_bg(&self.background, &base.background),
            border: pick(&self.border, &base.border),
            list_marker: pick(&self.list_marker, &base.list_marker),
            table_header: pick(&self.table_header, &base.table_header),
            table_border: pick(&self.table_border, &base.table_border),
            error: pick(&self.error, &base.error),
            warning: pick(&self.warning, &base.warning),
            syntax,
        }
    }
}

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

fn load_one_theme(
    path: &Path,
    already_loaded: &[Theme],
    manager: &ThemeManager,
) -> Result<Theme> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read theme file: {}", path.display()))?;
    let file: ThemeFile = serde_yaml::from_str(&content).with_context(|| {
        format!(
            "Failed to parse YAML theme file: {}",
            path.display()
        )
    })?;

    if file.name.trim().is_empty() {
        bail!("Theme file is missing 'name' field");
    }

    let base = match file.extends.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(parent_name) => find_base_theme(parent_name, already_loaded, manager)
            .with_context(|| format!(
                "Theme '{}' extends unknown theme '{}'",
                file.name, parent_name
            ))?,
        None => Theme::default(),
    };

    Ok(file.resolve(&base))
}

fn find_base_theme(
    name: &str,
    already_loaded: &[Theme],
    manager: &ThemeManager,
) -> Result<Theme> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn empty_themes_dir_returns_empty_vec() {
        let tmp = TempDir::new().unwrap();
        let manager = ThemeManager::new();
        assert!(load_user_themes(tmp.path(), &manager).unwrap().is_empty());
    }

    #[test]
    fn missing_themes_dir_returns_empty_vec() {
        let tmp = TempDir::new().unwrap();
        let nested = tmp.path().join("does-not-exist");
        let manager = ThemeManager::new();
        assert!(load_user_themes(&nested, &manager).unwrap().is_empty());
    }

    #[test]
    fn themes_path_must_be_a_directory() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join(THEMES_DIR);
        fs::write(&file, "not a directory").unwrap();
        let manager = ThemeManager::new();
        assert!(load_user_themes(tmp.path(), &manager).is_err());
    }

    #[test]
    fn loads_full_theme_with_all_fields() {
        let tmp = TempDir::new().unwrap();
        let themes = tmp.path().join(THEMES_DIR);
        fs::create_dir(&themes).unwrap();
        fs::write(
            themes.join("warm.yaml"),
            "name: warm\ndescription: warm palette\ntext: white\ntext_light: grey\nh1: \"#ff5577\"\nh2: green\nh3: yellow\nh4: blue\nh5: magenta\nh6: cyan\ncode: red\ncode_block: red\nquote: darkgrey\nlink: blue\nemphasis: yellow\nstrong: red\nstrikethrough: darkgrey\nhighlight_background: \"#222222\"\nbackground: \"#111111\"\nborder: grey\nlist_marker: green\ntable_header: yellow\ntable_border: grey\nerror: red\nwarning: yellow\nsyntax:\n  keyword: red\n  string: green\n  comment: darkgrey\n  number: magenta\n  operator: red\n  function: green\n  variable: white\n  type_name: blue\n",
        )
        .unwrap();

        let loaded = load_user_themes(tmp.path(), &ThemeManager::new()).unwrap();
        assert_eq!(loaded.len(), 1);
        let theme = &loaded[0];
        assert_eq!(theme.name, "warm");
        assert_eq!(theme.description, "warm palette");
        assert_eq!(theme.h1, Color::Rgb { r: 0xff, g: 0x55, b: 0x77 });
        assert_eq!(theme.syntax.keyword, Color::Red);
    }

    #[test]
    fn partial_fields_fill_from_default() {
        let tmp = TempDir::new().unwrap();
        let themes = tmp.path().join(THEMES_DIR);
        fs::create_dir(&themes).unwrap();
        fs::write(themes.join("partial.yaml"), "name: partial\nh1: red\n").unwrap();

        let loaded = load_user_themes(tmp.path(), &ThemeManager::new()).unwrap();
        let theme = &loaded[0];
        assert_eq!(theme.h1, Color::Red);
        assert_eq!(theme.h2, Theme::default().h2);
    }

    #[test]
    fn extends_builtin_theme() {
        let tmp = TempDir::new().unwrap();
        let themes = tmp.path().join(THEMES_DIR);
        fs::create_dir(&themes).unwrap();
        fs::write(
            themes.join("warm-mono.yaml"),
            "name: warm-mono\nextends: monokai\nh1: \"#ff0000\"\n",
        )
        .unwrap();

        let loaded = load_user_themes(tmp.path(), &ThemeManager::new()).unwrap();
        let theme = &loaded[0];
        assert_eq!(theme.name, "warm-mono");
        assert_eq!(theme.h1, Color::Rgb { r: 255, g: 0, b: 0 });
        assert_eq!(
            theme.quote,
            Color::Rgb {
                r: 117,
                g: 113,
                b: 94
            }
        );
    }

    #[test]
    fn extends_can_chain_user_themes() {
        let tmp = TempDir::new().unwrap();
        let themes = tmp.path().join(THEMES_DIR);
        fs::create_dir(&themes).unwrap();
        fs::write(
            themes.join("a.yaml"),
            "name: a\nextends: monokai\nh1: red\n",
        )
        .unwrap();
        fs::write(
            themes.join("b.yaml"),
            "name: b\nextends: a\nh2: green\n",
        )
        .unwrap();

        let loaded = load_user_themes(tmp.path(), &ThemeManager::new()).unwrap();
        assert_eq!(loaded.len(), 2);
        let b = loaded.iter().find(|t| t.name == "b").unwrap();
        assert_eq!(b.h2, Color::Green);
        assert_eq!(b.h1, Color::Red);
        assert_eq!(
            b.quote,
            Color::Rgb {
                r: 117,
                g: 113,
                b: 94
            }
        );
    }

    #[test]
    fn invalid_yaml_is_skipped_not_fatal() {
        let tmp = TempDir::new().unwrap();
        let themes = tmp.path().join(THEMES_DIR);
        fs::create_dir(&themes).unwrap();
        fs::write(themes.join("broken.yaml"), "this is: not a: valid theme").unwrap();
        fs::write(themes.join("good.yaml"), "name: good\nh1: red\n").unwrap();

        let loaded = load_user_themes(tmp.path(), &ThemeManager::new()).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "good");
    }

    #[test]
    fn unknown_extends_is_skipped() {
        let tmp = TempDir::new().unwrap();
        let themes = tmp.path().join(THEMES_DIR);
        fs::create_dir(&themes).unwrap();
        fs::write(
            themes.join("orphan.yaml"),
            "name: orphan\nextends: nonexistent\nh1: red\n",
        )
        .unwrap();
        fs::write(themes.join("good.yaml"), "name: good\nh1: red\n").unwrap();

        let loaded = load_user_themes(tmp.path(), &ThemeManager::new()).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "good");
    }

    #[test]
    fn ignores_non_yaml_files() {
        let tmp = TempDir::new().unwrap();
        let themes = tmp.path().join(THEMES_DIR);
        fs::create_dir(&themes).unwrap();
        fs::write(themes.join("readme.txt"), "name: should-be-ignored\n").unwrap();
        fs::write(themes.join("real.yaml"), "name: real\nh1: red\n").unwrap();

        let loaded = load_user_themes(tmp.path(), &ThemeManager::new()).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "real");
    }

    #[test]
    fn syntax_block_is_optional_and_merges() {
        let tmp = TempDir::new().unwrap();
        let themes = tmp.path().join(THEMES_DIR);
        fs::create_dir(&themes).unwrap();
        fs::write(
            themes.join("code-only.yaml"),
            "name: code-only\nextends: monokai\nsyntax:\n  keyword: \"#abcdef\"\n",
        )
        .unwrap();

        let loaded = load_user_themes(tmp.path(), &ThemeManager::new()).unwrap();
        let theme = &loaded[0];
        assert_eq!(
            theme.syntax.keyword,
            Color::Rgb {
                r: 0xab,
                g: 0xcd,
                b: 0xef
            }
        );
        assert_eq!(
            theme.syntax.string,
            Color::Rgb {
                r: 230,
                g: 219,
                b: 116
            }
        );
    }
}
