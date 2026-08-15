use super::*;

pub(super) const BUILTIN_THEME_FILES: [(&str, &str); 9] = [
    (
        "terminal",
        include_str!("../../assets/config/themes/terminal.yaml"),
    ),
    (
        "monokai",
        include_str!("../../assets/config/themes/monokai.yaml"),
    ),
    (
        "solarized-dark",
        include_str!("../../assets/config/themes/solarized-dark.yaml"),
    ),
    ("nord", include_str!("../../assets/config/themes/nord.yaml")),
    (
        "tokyonight",
        include_str!("../../assets/config/themes/tokyonight.yaml"),
    ),
    (
        "kanagawa",
        include_str!("../../assets/config/themes/kanagawa.yaml"),
    ),
    (
        "gruvbox",
        include_str!("../../assets/config/themes/gruvbox.yaml"),
    ),
    (
        "material-ocean",
        include_str!("../../assets/config/themes/material-ocean.yaml"),
    ),
    (
        "catppuccin",
        include_str!("../../assets/config/themes/catppuccin.yaml"),
    ),
];

pub(super) static BUILTIN_THEMES: LazyLock<HashMap<String, Theme>> = LazyLock::new(|| {
    BUILTIN_THEME_FILES
        .iter()
        .map(|(name, source)| {
            let theme = parse_embedded_theme(name, source)
                .unwrap_or_else(|error| panic!("invalid embedded theme '{name}': {error:#}"));
            ((*name).to_string(), theme)
        })
        .collect()
});
