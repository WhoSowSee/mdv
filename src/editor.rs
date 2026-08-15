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
mod tests;
