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
