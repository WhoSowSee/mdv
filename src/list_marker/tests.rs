use super::*;
use crate::theme::Color;

#[test]
fn pretty_list_uses_large_nerd_font_icons_by_default() {
    let cfg = ListMarkerConfig {
        style: Some(PrettyListStyle::default()),
        ..Default::default()
    };

    let (icon1, _) = cfg.resolve(1).unwrap();
    let (icon2, _) = cfg.resolve(2).unwrap();
    let (icon3, _) = cfg.resolve(3).unwrap();
    let (icon4, _) = cfg.resolve(4).unwrap();
    let (icon10, _) = cfg.resolve(10).unwrap();

    assert_eq!(icon1, "\u{f444}");
    assert_eq!(icon2, "\u{f445}");
    assert_eq!(icon3, "\u{f4c3}");
    assert_eq!(icon4, "\u{f51d}");
    assert_eq!(icon10, "\u{f51d}");
}

#[test]
fn pretty_list_uses_small_nerd_font_icons() {
    let cfg = ListMarkerConfig {
        style: Some(PrettyListStyle::parse("type:nerd-font;size:small").unwrap()),
        ..Default::default()
    };

    let icons = (1..=5)
        .map(|level| cfg.resolve(level).unwrap().0)
        .collect::<Vec<_>>();

    assert_eq!(
        icons,
        [
            "\u{f09de}",
            "\u{f0a13}",
            "\u{f14dc}",
            "\u{f0a14}",
            "\u{f0a14}"
        ]
    );
}

#[test]
fn pretty_list_style_defaults_omitted_fields() {
    assert_eq!(
        PrettyListStyle::parse("size:small").unwrap(),
        PrettyListStyle {
            marker_type: PrettyListType::NerdFont,
            size: PrettyListSize::Small,
        }
    );
    assert_eq!(
        PrettyListStyle::parse("type:unicode").unwrap(),
        PrettyListStyle {
            marker_type: PrettyListType::Unicode,
            size: PrettyListSize::Large,
        }
    );
    assert!(PrettyListStyle::parse("type:ascii").is_err());
}

#[test]
fn unicode_icons_ignore_size() {
    let large = ListMarkerConfig {
        style: Some(PrettyListStyle::parse("type:unicode;size:large").unwrap()),
        ..Default::default()
    };
    let small = ListMarkerConfig {
        style: Some(PrettyListStyle::parse("type:unicode;size:small").unwrap()),
        ..Default::default()
    };

    for (level, expected) in [(1, "⦁"), (2, "▪"), (3, "⚬"), (4, "▫"), (8, "▫")] {
        assert_eq!(large.resolve(level).unwrap().0, expected);
        assert_eq!(small.resolve(level).unwrap().0, expected);
    }
}

#[test]
fn uniform_custom_icon_is_used_for_every_level() {
    let cfg = ListMarkerConfig {
        style: Some(PrettyListStyle::default()),
        uniform: Some(UniformListMarker::Icon("*".to_string())),
        ..Default::default()
    };

    for level in 1..=6 {
        assert_eq!(cfg.resolve(level).unwrap().0, "*");
    }
}

#[test]
fn custom_list_overrides_per_level() {
    let cfg = ListMarkerConfig {
        style: Some(PrettyListStyle::default()),
        overrides: ListMarkerConfig::parse_custom_list("5:&").unwrap(),
        ..Default::default()
    };

    let (icon, color) = cfg.resolve(5).unwrap();
    assert_eq!(icon, "&");
    assert_eq!(color, None);
}

#[test]
fn custom_list_parses_color() {
    let overrides = ListMarkerConfig::parse_custom_list("5:&:#ff0000").unwrap();
    let entry = overrides.get(&5).unwrap();
    assert_eq!(entry.icon, Some("&".to_string()));
    assert_eq!(
        entry.color,
        Some(Color::Rgb {
            r: 0xff,
            g: 0,
            b: 0
        })
    );
}

#[test]
fn custom_list_rejects_duplicate_levels() {
    assert!(ListMarkerConfig::parse_custom_list("1:a;1:b").is_err());
}

#[test]
fn custom_list_rejects_zero_level() {
    assert!(ListMarkerConfig::parse_custom_list("0:a").is_err());
}

#[test]
fn custom_list_rejects_empty_value() {
    assert!(ListMarkerConfig::parse_custom_list("1:").is_err());
}

#[test]
fn inactive_config_returns_none() {
    let cfg = ListMarkerConfig::default();
    assert!(cfg.resolve(1).is_none());
}

#[test]
fn custom_list_color_only_parses() {
    let overrides = ListMarkerConfig::parse_custom_list("1:red;2:#00ff00").unwrap();
    assert_eq!(overrides.get(&1).unwrap().icon, None);
    assert_eq!(overrides.get(&1).unwrap().color, Some(Color::Red));
    assert_eq!(overrides.get(&2).unwrap().icon, None);
    assert_eq!(
        overrides.get(&2).unwrap().color,
        Some(Color::Rgb {
            r: 0,
            g: 0xff,
            b: 0
        })
    );
}

#[test]
fn color_only_falls_back_to_pretty_icon() {
    let cfg = ListMarkerConfig {
        style: Some(PrettyListStyle::default()),
        overrides: ListMarkerConfig::parse_custom_list("1:red").unwrap(),
        ..Default::default()
    };
    let (icon, color) = cfg.resolve(1).unwrap();
    assert_eq!(icon, "\u{f444}");
    assert_eq!(color, Some(Color::Red));
}

#[test]
fn color_only_rejects_extra_tokens() {
    assert!(ListMarkerConfig::parse_custom_list("1:red:extra").is_err());
}

#[test]
fn custom_icon_overrides_uniform_marker() {
    let cfg = ListMarkerConfig {
        style: Some(PrettyListStyle::default()),
        uniform: Some(UniformListMarker::Level(2)),
        overrides: ListMarkerConfig::parse_custom_list("3:>").unwrap(),
    };

    assert_eq!(cfg.resolve(2).unwrap().0, "\u{f445}");
    assert_eq!(cfg.resolve(3).unwrap().0, ">");
}

#[test]
fn uniform_marker_parser_enforces_exclusive_valid_value() {
    assert_eq!(
        UniformListMarker::parse("level:4").unwrap(),
        UniformListMarker::Level(4)
    );
    assert_eq!(
        UniformListMarker::parse("icon:◆").unwrap(),
        UniformListMarker::Icon("◆".to_string())
    );
    assert!(UniformListMarker::parse("level:0").is_err());
    assert!(UniformListMarker::parse("level:5").is_err());
    assert!(UniformListMarker::parse("level:1;icon:◆").is_err());
}
