use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn optional_background_inheritance_preserves_explicit_clears_and_resets() {
    let fields = [
        "background",
        "emphasis:background",
        "strong:background",
        "strong_emphasis:background",
        "code:background",
        "strikethrough:background",
    ];
    let assignments = |value: &str| {
        fields
            .iter()
            .map(|field| format!("{}: {value}\n", field.replace(':', ":\n  ")))
            .collect::<String>()
    };
    let parent: schema::ThemeFile =
        serde_yaml::from_str(&format!("name: parent\n{}", assignments("red"))).unwrap();
    let mut parent = parent.resolve(&Theme::default());
    parent.pager.selection.background = Some(Color::Red);
    parent.pager.title = Some(Color::Blue);
    let child: ThemeFile =
        serde_yaml::from_str("name: child\npager: {selection: {text: yellow}}\n").unwrap();
    let child = child.resolve(&parent);
    assert_eq!(child.pager.title, Some(Color::Blue));
    assert_eq!(child.pager.selection.text, Some(Color::Yellow));
    assert_eq!(child.pager.selection.background, Some(Color::Red));
    let child: ThemeFile =
        serde_yaml::from_str("name: child\npager: {title: null, selection: {background: null}}\n")
            .unwrap();
    let child = child.resolve(&parent);
    assert!(child.pager.title.is_none() && child.pager.selection.background.is_none());
    for (yaml, expected) in [
        ("name: child\n".to_string(), Some(Color::Red)),
        (
            "name: child\ncode: {text: yellow}\n".to_string(),
            Some(Color::Red),
        ),
        (format!("name: child\n{}", assignments("null")), None),
        (
            format!("name: child\n{}", assignments("reset")),
            Some(Color::Reset),
        ),
    ] {
        let child: schema::ThemeFile = serde_yaml::from_str(&yaml).unwrap();
        let child = child.resolve(&parent);
        for color in [
            child.background,
            child.emphasis.background,
            child.strong.background,
            child.strong_emphasis.background,
            child.code.background,
            child.strikethrough.background,
        ] {
            assert_eq!(color, expected, "{yaml}");
        }
    }
}

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
        "name: warm\ndescription: warm palette\ntext: white\ntext_light: grey\nh1: '#ff5577'\nh2: green\nh3: yellow\nh4: blue\nh5: magenta\nh6: cyan\ncode:\n  text: red\nquote: darkgrey\nlink: blue\nemphasis:\n  text: yellow\nstrong:\n  text: red\nstrikethrough:\n  text: darkgrey\nhighlight:\n  background: '#222222'\nbackground: '#111111'\nerror: red\nwarning: yellow\nsyntax:\n  keyword: red\n  string: green\n  comment: darkgrey\n  number: magenta\n  operator: red\n  function: green\n  variable: white\n  type_name: blue\nline_number:\n  number: yellow\n  separator: blue\ntable:\n  header: yellow\n  border: grey\nmath:\n  text: cyan\n  border: blue\nlist:\n  unordered: green\n",
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
    assert_eq!(theme.line_number.number, Color::Yellow);
    assert_eq!(theme.line_number.separator, Color::Blue);
    assert_eq!(&theme.math.text, &Color::Cyan);
    assert_eq!(
        theme.math.border.as_ref().expect("math border"),
        &Color::Blue
    );
    assert_eq!(theme.syntax.keyword, Color::Red);
}

#[test]
fn removed_theme_fields_are_rejected() {
    for field in [
        "border",
        "code_background",
        "details",
        "pager_status_bar_transparent",
    ] {
        let yaml = format!("name: legacy\n{field}: red\n");
        let error = serde_yaml::from_str::<ThemeFile>(&yaml).unwrap_err();
        assert!(
            error
                .to_string()
                .contains(&format!("unknown field `{field}`")),
            "{error}"
        );
    }
    assert!(serde_yaml::from_str::<ThemeFile>("name: legacy\ncode: red\n").is_err());
}

#[test]
fn partial_fields_fill_from_default() {
    let tmp = TempDir::new().unwrap();
    let themes = tmp.path().join(THEMES_DIR);
    fs::create_dir(&themes).unwrap();
    fs::write(
        themes.join("partial.yaml"),
        "name: partial\nh1: red\ncode: {background: red}\nhighlight: {text: magenta}\nline_number:\n  number: yellow\n  separator: blue\nfront_matter:\n  key: magenta\n",
    )
    .unwrap();

    let loaded = load_user_themes(tmp.path(), &ThemeManager::new()).unwrap();
    let theme = &loaded[0];
    assert_eq!(theme.h1, Color::Red);
    assert_eq!(theme.h2, Theme::default().h2);
    assert_eq!(theme.code.text, Theme::default().code.text);
    assert_eq!(theme.code.background, Some(Color::Red));
    assert_eq!(theme.highlight.text, Some(Color::Magenta));
    assert_eq!(
        theme.highlight.background,
        Theme::default().highlight.background
    );
    assert_eq!(theme.line_number.number, Color::Yellow);
    assert_eq!(theme.line_number.separator, Color::Blue);
    assert_eq!(theme.front_matter.key, Color::Magenta);
    assert_eq!(
        theme.front_matter.title,
        Theme::default().front_matter.title
    );
    assert_eq!(theme.front_matter.value, Theme::default().text);
    assert_eq!(
        theme.front_matter.border,
        Theme::default().front_matter.border
    );
    assert_eq!(&theme.math.text, &Theme::default().math.text);
    assert_eq!(
        theme.math.border.as_ref().expect("math border"),
        Theme::default().math.border.as_ref().expect("math border")
    );
}

#[test]
fn user_theme_can_enable_transparent_pager_status_bar() {
    let tmp = TempDir::new().unwrap();
    let themes = tmp.path().join(THEMES_DIR);
    fs::create_dir(&themes).unwrap();
    fs::write(
        themes.join("transparent.yaml"),
        "name: transparent\npager:\n  transparent: true\n",
    )
    .unwrap();

    let loaded = load_user_themes(tmp.path(), &ThemeManager::new()).unwrap();
    assert_eq!(loaded.len(), 1);
    let yaml = serde_yaml::to_value(&loaded[0]).unwrap();
    assert_eq!(
        yaml.get("pager").and_then(|pager| pager.get("transparent")),
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
        "name: warm-mono\nextends: monokai\nh1: '#ff0000'\n",
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
        "name: a\nextends: monokai\nh1: red\nhorizontal_rule: '#010203'\ntable:\n  border: '#0d0e0f'\n",
    )
    .unwrap();
    fs::write(themes.join("b.yaml"), "name: b\nextends: a\nh2: green\n").unwrap();

    let loaded = load_user_themes(tmp.path(), &ThemeManager::new()).unwrap();
    assert_eq!(loaded.len(), 2);
    let b = loaded.iter().find(|t| t.name == "b").unwrap();
    assert_eq!(b.h2, Color::Green);
    assert_eq!(b.h1, Color::Red);
    assert_eq!(b.horizontal_rule, Some(Color::Rgb { r: 1, g: 2, b: 3 }));
    assert_eq!(
        b.table.border.as_ref().expect("table border").clone(),
        Color::Rgb {
            r: 13,
            g: 14,
            b: 15
        }
    );
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
        "name: code-only\nextends: monokai\nsyntax:\n  keyword: '#abcdef'\n",
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
