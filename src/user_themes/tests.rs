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
        "name: warm\ndescription: warm palette\ntext: white\ntext_light: grey\nline_number: yellow\nline_number_separator: blue\nh1: \"#ff5577\"\nh2: green\nh3: yellow\nh4: blue\nh5: magenta\nh6: cyan\ncode: red\nquote: darkgrey\nlink: blue\nemphasis: yellow\nstrong: red\nstrikethrough: darkgrey\nhighlight_background: \"#222222\"\nbackground: \"#111111\"\nborder: grey\nlist_marker: green\ntable_header: yellow\ntable_border: grey\nerror: red\nwarning: yellow\nsyntax:\n  keyword: red\n  string: green\n  comment: darkgrey\n  number: magenta\n  operator: red\n  function: green\n  variable: white\n  type_name: blue\n",
    )
    .unwrap();

    let loaded = load_user_themes(tmp.path(), &ThemeManager::new()).unwrap();
    assert_eq!(loaded.len(), 1);
    let theme = &loaded[0];
    assert_eq!(theme.name, "warm");
    assert_eq!(theme.description, "warm palette");
    assert_eq!(
        theme.h1,
        Color::Rgb {
            r: 0xff,
            g: 0x55,
            b: 0x77
        }
    );
    assert_eq!(theme.line_number, Color::Yellow);
    assert_eq!(theme.line_number_separator, Color::Blue);
    assert_eq!(theme.syntax.keyword, Color::Red);
}

#[test]
fn removed_code_block_field_is_rejected() {
    let error = serde_yaml::from_str::<ThemeFile>("name: legacy\ncode_block: red\n")
        .expect_err("removed code_block field must be rejected");
    assert!(
        error.to_string().contains("unknown field `code_block`"),
        "unexpected error: {error}"
    );
}

#[test]
fn partial_fields_fill_from_default() {
    let tmp = TempDir::new().unwrap();
    let themes = tmp.path().join(THEMES_DIR);
    fs::create_dir(&themes).unwrap();
    fs::write(
        themes.join("partial.yaml"),
        "name: partial\nh1: red\nline_number: yellow\nline_number_separator: blue\n",
    )
    .unwrap();

    let loaded = load_user_themes(tmp.path(), &ThemeManager::new()).unwrap();
    let theme = &loaded[0];
    assert_eq!(theme.h1, Color::Red);
    assert_eq!(theme.h2, Theme::default().h2);
    assert_eq!(theme.line_number, Color::Yellow);
    assert_eq!(theme.line_number_separator, Color::Blue);
}

#[test]
fn user_theme_can_enable_transparent_pager_status_bar() {
    let tmp = TempDir::new().unwrap();
    let themes = tmp.path().join(THEMES_DIR);
    fs::create_dir(&themes).unwrap();
    fs::write(
        themes.join("transparent.yaml"),
        "name: transparent\npager_status_bar_transparent: true\n",
    )
    .unwrap();

    let loaded = load_user_themes(tmp.path(), &ThemeManager::new()).unwrap();
    assert_eq!(loaded.len(), 1);
    let yaml = serde_yaml::to_value(&loaded[0]).unwrap();
    assert_eq!(
        yaml.get("pager_status_bar_transparent"),
        Some(&serde_yaml::Value::Bool(true))
    );
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
    fs::write(themes.join("b.yaml"), "name: b\nextends: a\nh2: green\n").unwrap();

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
