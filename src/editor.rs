use anyhow::{Result, bail};
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditorKind {
    Terminal,
    Gui,
}

#[derive(Debug)]
struct EditorDefinition {
    aliases: &'static [&'static str],
    default_kind: EditorKind,
    terminal_args: &'static [&'static str],
    gui_args: &'static [&'static str],
}

const GUI_EDITORS: &[&str] = &[
    "akelpad",
    "android-studio",
    "anjuta",
    "appcode",
    "apostrophe",
    "aqua",
    "aqua64",
    "arduino-ide",
    "atom",
    "bbedit",
    "bluefish",
    "bowpad",
    "brackets",
    "clion",
    "clion64",
    "code",
    "code-insiders",
    "codeblocks",
    "codium",
    "coteditor",
    "cudatext",
    "cursor",
    "datagrip",
    "datagrip64",
    "dataspell",
    "devenv",
    "eclipse",
    "editplus",
    "eview",
    "evim",
    "featherpad",
    "fleet",
    "fvim",
    "geany",
    "gedit",
    "ghostwriter",
    "gnome-text-editor",
    "goland",
    "goland64",
    "goneovim",
    "idle",
    "idle3",
    "idea",
    "idea64",
    "jedit",
    "kate",
    "kdevelop",
    "kiro",
    "komodo",
    "komodo-edit",
    "kwrite",
    "l3afpad",
    "lapce",
    "leafpad",
    "lite-xl",
    "macvim",
    "marktext",
    "mate",
    "medit",
    "metapad",
    "mousepad",
    "mps",
    "mvim",
    "netbeans",
    "neovide",
    "notepad",
    "notepad++",
    "notepad3",
    "notepad4",
    "nova",
    "nvim-qt",
    "obsidian",
    "phpstorm",
    "phpstorm64",
    "pluma",
    "positron",
    "pspad",
    "pulsar",
    "pycharm",
    "pycharm-community",
    "pycharm-professional",
    "pycharm64",
    "qtcreator",
    "retext",
    "rider",
    "rider64",
    "rstudio",
    "rubymine",
    "rubymine64",
    "rustrover",
    "rustrover64",
    "scite",
    "spyder",
    "studio",
    "studio64",
    "subl",
    "sublime_text",
    "textadept",
    "textedit",
    "textmate",
    "texmaker",
    "texstudio",
    "thonny",
    "trae",
    "uedit64",
    "ultraedit",
    "vimr",
    "vscodium",
    "webstorm",
    "webstorm64",
    "windsurf",
    "writerside",
    "xed",
    "zed",
    "zed-nightly",
    "zed-preview",
    "zeditor",
];

const VIM_EDITORS: &[&str] = &[
    "ex",
    "rview",
    "rvim",
    "vi",
    "view",
    "vim",
    "vim.basic",
    "vim.gtk3",
    "vim.nox",
    "vim.tiny",
    "vimdiff",
];

const GVIM_EDITORS: &[&str] = &["gex", "gview", "gvim", "gvimdiff", "rgview", "rgvim"];

const TERMINAL_EDITORS: &[&str] = &[
    "amp", "dav", "dte", "e3", "ed", "elvis", "helix", "hx", "jed", "jmacs", "joe", "jpico",
    "jstar", "kak", "kakoune", "kilo", "mcedit", "mg", "micro", "nano", "ne", "nvi", "nvim",
    "nvimdiff", "ox", "pico", "red", "slap", "tilde", "vile", "vis", "zile",
];

const EDITOR_REGISTRY: &[EditorDefinition] = &[
    EditorDefinition {
        aliases: GUI_EDITORS,
        default_kind: EditorKind::Gui,
        terminal_args: &[],
        gui_args: &[],
    },
    EditorDefinition {
        aliases: &["emacs"],
        default_kind: EditorKind::Gui,
        terminal_args: &["-nw", "--no-window-system"],
        gui_args: &[],
    },
    EditorDefinition {
        aliases: &["emacsclient"],
        default_kind: EditorKind::Gui,
        terminal_args: &["-nw", "-t", "--no-window-system", "--tty"],
        gui_args: &[],
    },
    EditorDefinition {
        aliases: VIM_EDITORS,
        default_kind: EditorKind::Terminal,
        terminal_args: &[],
        gui_args: &["-g", "-y", "--gui"],
    },
    EditorDefinition {
        aliases: GVIM_EDITORS,
        default_kind: EditorKind::Gui,
        terminal_args: &["-v"],
        gui_args: &[],
    },
    EditorDefinition {
        aliases: TERMINAL_EDITORS,
        default_kind: EditorKind::Terminal,
        terminal_args: &[],
        gui_args: &[],
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EditorCommand {
    program: String,
    args: Vec<String>,
    kind: EditorKind,
}

impl EditorCommand {
    pub fn from_env() -> Result<Option<Self>> {
        let mdv_editor = std::env::var("MDV_EDITOR").ok();
        let editor = std::env::var("EDITOR").ok();
        let mode = std::env::var("MDV_EDITOR_MODE").ok();

        Self::from_values(mdv_editor.as_deref(), editor.as_deref(), mode.as_deref())
    }

    fn from_values(
        mdv_editor: Option<&str>,
        editor: Option<&str>,
        mode: Option<&str>,
    ) -> Result<Option<Self>> {
        let raw = [mdv_editor, editor]
            .into_iter()
            .flatten()
            .find(|value| !value.trim().is_empty());
        let Some(raw) = raw else {
            return Ok(None);
        };
        let Some(parts) = split_command(raw) else {
            return Ok(None);
        };
        let mut parts = parts.into_iter();
        let Some(program) = parts.next() else {
            return Ok(None);
        };
        let args: Vec<String> = parts.collect();
        let kind = resolve_editor_kind(&program, &args, mode)?;

        Ok(Some(Self {
            program,
            args,
            kind,
        }))
    }

    pub fn open(&self, path: &Path) -> std::io::Result<()> {
        let mut command = Command::new(&self.program);
        command
            .args(&self.args)
            .arg(path)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        match self.kind {
            EditorKind::Gui => {
                let mut child = command.spawn()?;
                std::thread::spawn(move || {
                    let _ = child.wait();
                });
            }
            EditorKind::Terminal => {
                let status = command.status()?;
                if !status.success() {
                    return Err(std::io::Error::other(format!(
                        "editor exited with status {status}"
                    )));
                }
            }
        }

        Ok(())
    }
}

#[cfg(windows)]
fn split_command(raw: &str) -> Option<Vec<String>> {
    Some(winsplit::split(raw))
}

#[cfg(not(windows))]
fn split_command(raw: &str) -> Option<Vec<String>> {
    shell_words::split(raw).ok()
}

fn detect_editor(program: &str, args: &[String]) -> Option<EditorKind> {
    let executable = normalize_editor_name(program);
    let definition = EDITOR_REGISTRY
        .iter()
        .find(|definition| definition.aliases.contains(&executable.as_str()))?;

    if args
        .iter()
        .any(|arg| definition.terminal_args.contains(&arg.as_str()))
    {
        return Some(EditorKind::Terminal);
    }
    if args
        .iter()
        .any(|arg| definition.gui_args.contains(&arg.as_str()))
    {
        return Some(EditorKind::Gui);
    }

    Some(definition.default_kind)
}

fn resolve_editor_kind(program: &str, args: &[String], mode: Option<&str>) -> Result<EditorKind> {
    let detected_kind = detect_editor(program, args);
    let automatic_kind = detected_kind.unwrap_or(EditorKind::Terminal);
    let Some(mode) = mode.map(str::trim).filter(|mode| !mode.is_empty()) else {
        return Ok(automatic_kind);
    };

    if mode.eq_ignore_ascii_case("tui") {
        return Ok(EditorKind::Terminal);
    }
    if mode.eq_ignore_ascii_case("gui") {
        if detected_kind == Some(EditorKind::Terminal) {
            bail!(
                "MDV_EDITOR_MODE=gui conflicts with terminal editor '{}'",
                normalize_editor_name(program)
            );
        }
        return Ok(EditorKind::Gui);
    }

    bail!("invalid MDV_EDITOR_MODE value '{mode}'; expected 'tui' or 'gui'")
}

fn normalize_editor_name(program: &str) -> String {
    let file_name = program.rsplit(['/', '\\']).next().unwrap_or(program);
    let normalized = file_name.to_ascii_lowercase();

    [".exe", ".cmd", ".bat", ".com"]
        .into_iter()
        .find_map(|extension| normalized.strip_suffix(extension))
        .unwrap_or(&normalized)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn mdv_editor_has_priority() {
        let editor = EditorCommand::from_values(Some("nvim -f"), Some("code"), None)
            .unwrap()
            .unwrap();

        assert_eq!(editor.program, "nvim");
        assert_eq!(editor.args, ["-f"]);
        assert_eq!(editor.kind, EditorKind::Terminal);
    }

    #[test]
    fn editor_is_used_when_mdv_editor_is_empty() {
        let editor = EditorCommand::from_values(Some("  "), Some("code --reuse-window"), None)
            .unwrap()
            .unwrap();

        assert_eq!(editor.program, "code");
        assert_eq!(editor.args, ["--reuse-window"]);
        assert_eq!(editor.kind, EditorKind::Gui);
    }

    #[test]
    fn empty_editor_values_disable_opening() {
        assert_eq!(
            EditorCommand::from_values(Some(""), Some(" \t"), None).unwrap(),
            None
        );
    }

    #[test]
    fn quoted_program_path_is_preserved() {
        let editor = EditorCommand::from_values(
            Some(r#""C:\Program Files\Notepad++\notepad++.exe" -multiInst"#),
            None,
            None,
        )
        .unwrap()
        .unwrap();

        assert_eq!(
            editor.program,
            r#"C:\Program Files\Notepad++\notepad++.exe"#
        );
        assert_eq!(editor.args, ["-multiInst"]);
        assert_eq!(editor.kind, EditorKind::Gui);
    }

    #[cfg(windows)]
    #[test]
    fn unquoted_windows_program_path_is_preserved() {
        let editor = EditorCommand::from_values(Some(r"C:\Tools\nvim.exe --clean"), None, None)
            .unwrap()
            .unwrap();

        assert_eq!(editor.program, r"C:\Tools\nvim.exe");
        assert_eq!(editor.args, ["--clean"]);
    }

    #[test]
    fn editor_registry_aliases_are_unique() {
        let mut aliases = HashSet::new();

        for definition in EDITOR_REGISTRY {
            for alias in definition.aliases {
                assert!(aliases.insert(*alias), "duplicate editor alias: {alias}");
            }
        }
    }

    #[test]
    fn known_gui_editor_aliases_are_detected() {
        for program in GUI_EDITORS {
            let editor = EditorCommand::from_values(Some(program), None, None)
                .unwrap()
                .unwrap();
            assert_eq!(editor.kind, EditorKind::Gui, "program: {program}");
        }
    }

    #[test]
    fn known_terminal_editor_aliases_are_detected() {
        for program in TERMINAL_EDITORS {
            assert_eq!(
                detect_editor(program, &[]),
                Some(EditorKind::Terminal),
                "program: {program}"
            );
        }
    }

    #[test]
    fn unregistered_editors_use_safe_terminal_default() {
        for program in ["custom-editor", "personal-editor", "wrapper"] {
            let editor = EditorCommand::from_values(Some(program), None, None)
                .unwrap()
                .unwrap();
            assert_eq!(editor.kind, EditorKind::Terminal, "program: {program}");
        }
    }

    #[test]
    fn editor_aliases_are_normalized_from_paths_and_launcher_extensions() {
        let editors = [
            r"C:\Program Files\Microsoft VS Code\bin\code.cmd",
            r"C:\Tools\Zed\zed.exe",
            r"C:\Tools\VSCodium\codium.BAT",
            "/Applications/Sublime Text.app/Contents/SharedSupport/bin/subl",
            "/usr/local/bin/zeditor",
        ];

        for program in editors {
            assert_eq!(
                detect_editor(program, &[]),
                Some(EditorKind::Gui),
                "program: {program}"
            );
        }
    }

    #[test]
    fn hybrid_editors_use_arguments_to_select_terminal_mode() {
        let terminal_commands = [
            "emacs -nw",
            "emacs --no-window-system",
            "emacsclient -nw",
            "emacsclient -t",
            "emacsclient --no-window-system",
            "emacsclient --tty",
            "gvim -v",
        ];

        for command in terminal_commands {
            let editor = EditorCommand::from_values(Some(command), None, None)
                .unwrap()
                .unwrap();
            assert_eq!(editor.kind, EditorKind::Terminal, "command: {command}");
        }
    }

    #[test]
    fn hybrid_editors_default_to_gui_or_honor_gui_mode() {
        let gui_commands = [
            "emacs",
            "emacsclient",
            "emacsclient --create-frame",
            "vim -g",
            "vim -y",
            "vimdiff -g",
        ];

        for command in gui_commands {
            let editor = EditorCommand::from_values(Some(command), None, None)
                .unwrap()
                .unwrap();
            assert_eq!(editor.kind, EditorKind::Gui, "command: {command}");
        }
    }

    #[test]
    fn explicit_editor_modes_override_auto_detection() {
        let cases = [
            ("code --wait", "tui", EditorKind::Terminal),
            ("custom-editor", "gui", EditorKind::Gui),
        ];

        for (command, mode, expected) in cases {
            let editor = EditorCommand::from_values(Some(command), None, Some(mode))
                .unwrap()
                .unwrap();

            assert_eq!(editor.kind, expected, "command: {command}, mode: {mode}");
        }
    }

    #[test]
    fn explicit_gui_mode_rejects_detected_terminal_editors() {
        let cases = [("nvim", "nvim"), ("emacsclient --tty", "emacsclient")];

        for (command, executable) in cases {
            let error = EditorCommand::from_values(Some(command), None, Some("gui")).unwrap_err();

            assert_eq!(
                error.to_string(),
                format!("MDV_EDITOR_MODE=gui conflicts with terminal editor '{executable}'"),
                "command: {command}"
            );
        }
    }

    #[test]
    fn editor_mode_defaults_to_auto_and_requires_an_editor_command() {
        for mode in [None, Some("  ")] {
            let editor = EditorCommand::from_values(Some("code"), None, mode)
                .unwrap()
                .unwrap();

            assert_eq!(editor.kind, EditorKind::Gui, "mode: {mode:?}");
        }

        assert_eq!(
            EditorCommand::from_values(None, None, Some("invalid")).unwrap(),
            None
        );
    }

    #[test]
    fn invalid_editor_mode_is_rejected() {
        let error = EditorCommand::from_values(Some("code"), None, Some("desktop")).unwrap_err();

        assert_eq!(
            error.to_string(),
            "invalid MDV_EDITOR_MODE value 'desktop'; expected 'tui' or 'gui'"
        );
    }
}
