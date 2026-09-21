use crate::cli::{ColorDepth, OutputStyle};
use minus::ColorDepth as Depth;

pub(crate) fn resolve(style: OutputStyle, requested: ColorDepth) -> OutputStyle {
    if style.is_disabled() {
        return style;
    }
    let depth = match requested {
        ColorDepth::Ansi16 => Depth::Ansi16,
        ColorDepth::Ansi256 => Depth::Ansi256,
        ColorDepth::TrueColor => Depth::TrueColor,
        ColorDepth::Auto => detect(&Environment::read(), terminfo_depth()),
    };
    match depth {
        Depth::Ansi16 => OutputStyle::Ansi16,
        Depth::Ansi256 => OutputStyle::Ansi256,
        Depth::TrueColor => OutputStyle::Enabled,
    }
}

#[derive(Default)]
struct Environment {
    term: String,
    colorterm: String,
    program: String,
    windows_terminal: bool,
    remote: bool,
    multiplexer: bool,
}

impl Environment {
    fn read() -> Self {
        let read = |key| std::env::var(key).unwrap_or_default().to_ascii_lowercase();
        Self {
            term: read("TERM"),
            colorterm: read("COLORTERM"),
            program: read("TERM_PROGRAM"),
            windows_terminal: std::env::var_os("WT_SESSION").is_some_and(|v| !v.is_empty()),
            remote: std::env::var_os("SSH_CONNECTION").is_some()
                || std::env::var_os("SSH_TTY").is_some(),
            multiplexer: ["TMUX", "STY"]
                .iter()
                .any(|key| std::env::var_os(key).is_some_and(|value| !value.is_empty())),
        }
    }
}

fn detect(env: &Environment, database: Option<Depth>) -> Depth {
    let term = env.term.as_str();
    if matches!(term, "dumb" | "linux" | "vt100" | "vt102" | "ansi") {
        return Depth::Ansi16;
    }
    let multiplexer = env.multiplexer
        || term.starts_with("screen")
        || term.starts_with("tmux")
        || matches!(env.program.as_str(), "tmux" | "screen");
    if term.ends_with("-direct")
        || term.ends_with("-truecolor")
        || term.ends_with("-24bit")
        || database == Some(Depth::TrueColor)
    {
        return Depth::TrueColor;
    }
    if !multiplexer {
        if matches!(env.colorterm.as_str(), "truecolor" | "24bit") {
            return Depth::TrueColor;
        }
        if !env.remote
            && (env.windows_terminal
                || matches!(
                    env.program.as_str(),
                    "wezterm" | "iterm.app" | "vscode" | "ghostty"
                ))
        {
            return Depth::TrueColor;
        }
        if matches!(
            term,
            "xterm-kitty" | "xterm-ghostty" | "wezterm" | "alacritty" | "foot" | "foot-extra"
        ) {
            return Depth::TrueColor;
        }
    }
    if let Some(depth) = database {
        return depth;
    }
    if term.ends_with("-256color") || term.ends_with("-256") {
        return Depth::Ansi256;
    }
    Depth::TrueColor
}

#[cfg(unix)]
fn terminfo_depth() -> Option<Depth> {
    use terminfo::{Database, Value, capability as cap};
    let database = Database::from_env().ok()?;
    if matches!(database.raw("Tc"), Some(Value::True))
        || (database.get::<cap::SetTrueColorForeground>().is_some()
            && database.get::<cap::SetTrueColorBackground>().is_some())
    {
        return Some(Depth::TrueColor);
    }
    database
        .get::<cap::MaxColors>()
        .map(|cap::MaxColors(colors)| {
            if colors >= 1 << 24 {
                Depth::TrueColor
            } else if colors >= 256 {
                Depth::Ansi256
            } else {
                Depth::Ansi16
            }
        })
}

#[cfg(not(unix))]
fn terminfo_depth() -> Option<Depth> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detection_respects_terminal_identity_and_explicit_capabilities() {
        for (term, colorterm, database, expected) in [
            ("linux", "truecolor", None, Depth::Ansi16),
            (
                "xterm-256color",
                "truecolor",
                Some(Depth::Ansi256),
                Depth::TrueColor,
            ),
            ("xterm-256color", "", None, Depth::Ansi256),
            ("tmux-256color", "truecolor", None, Depth::Ansi256),
            ("xterm", "", Some(Depth::Ansi16), Depth::Ansi16),
            ("", "", None, Depth::TrueColor),
            ("unknown", "", None, Depth::TrueColor),
        ] {
            let environment = Environment {
                term: term.into(),
                colorterm: colorterm.into(),
                ..Environment::default()
            };
            assert_eq!(
                detect(&environment, database),
                expected,
                "{term}/{colorterm}"
            );
        }

        let mut environment = Environment {
            term: "xterm-256color".into(),
            program: "wezterm".into(),
            windows_terminal: true,
            ..Environment::default()
        };
        assert_eq!(detect(&environment, Some(Depth::Ansi256)), Depth::TrueColor);
        environment.remote = true;
        assert_eq!(detect(&environment, Some(Depth::Ansi256)), Depth::Ansi256);
        environment.remote = false;
        environment.multiplexer = true;
        environment.colorterm = "truecolor".into();
        assert_eq!(detect(&environment, None), Depth::Ansi256);
    }
}
