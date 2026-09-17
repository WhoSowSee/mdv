use super::*;
use crate::cli::OutputStyle;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use notify::{EventKind, event::CreateKind};
use std::sync::atomic::AtomicUsize;
use tempfile::TempDir;

#[test]
fn editor_key_accepts_supported_layouts_without_control_modifiers() {
    for character in ['E', 'e', '\u{0423}', '\u{0443}'] {
        let event = Event::Key(KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE));

        assert!(is_editor_key(&event), "character: {character}");
    }

    assert!(is_editor_key(&Event::Key(KeyEvent::new(
        KeyCode::Char('E'),
        KeyModifiers::SHIFT,
    ))));
    assert!(!is_editor_key(&Event::Key(KeyEvent::new(
        KeyCode::Char('r'),
        KeyModifiers::NONE,
    ))));
    assert!(!is_editor_key(&Event::Key(KeyEvent::new(
        KeyCode::Char('e'),
        KeyModifiers::CONTROL,
    ))));
}

#[test]
fn modified_question_mark_does_not_open_help() {
    let event = Event::Key(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::CONTROL));

    assert!(!is_help_key(&event));
}

#[test]
fn escape_routing_respects_help_and_search_state() {
    let event = Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

    assert_eq!(
        help_input_action(&event, true, false, false),
        HelpInputAction::Dismiss
    );
    assert_eq!(
        help_input_action(&event, false, false, false),
        HelpInputAction::Forward
    );
    assert_eq!(
        help_input_action(&event, true, true, false),
        HelpInputAction::Forward
    );
}

#[test]
fn input_prompts_close_visible_help_before_reaching_minus() {
    let event = Event::Key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE));

    assert_eq!(
        help_input_action(&event, true, false, true),
        HelpInputAction::DismissAndForward
    );
    assert_eq!(
        help_input_action(&event, false, false, false),
        HelpInputAction::Forward
    );
}

#[test]
fn question_mark_always_toggles_help() {
    let event = Event::Key(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::SHIFT));

    assert_eq!(
        help_input_action(&event, false, false, false),
        HelpInputAction::Toggle
    );
    assert_eq!(
        help_input_action(&event, true, false, false),
        HelpInputAction::Toggle
    );
}

#[test]
fn copy_key_accepts_only_unmodified_c() {
    let event = Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE));
    let modified = Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));

    assert!(is_copy_key(&event));
    assert!(!is_copy_key(&modified));
}

#[test]
fn reload_key_accepts_only_unmodified_r() {
    let event = Event::Key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE));
    let modified = Event::Key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::ALT));

    assert!(is_reload_key(&event));
    assert!(!is_reload_key(&modified));
}

#[test]
fn line_number_modes_advance_from_the_active_starting_mode() {
    let cases = [
        (PagerLineNumberMode::Off, PagerLineNumberMode::Rendered),
        (PagerLineNumberMode::Rendered, PagerLineNumberMode::Source),
        (PagerLineNumberMode::Source, PagerLineNumberMode::Off),
    ];

    for (initial, expected) in cases {
        let mut document = numbered_document(initial, "initial");

        assert!(document.cycle_line_number_mode().is_some());
        assert_eq!(document.line_number_mode(), Some(expected));
    }
}

#[test]
fn line_navigation_uses_the_mdv_source_view_for_each_active_mode() {
    let cases = [
        (
            PagerLineNumberMode::Source,
            "1 current source\n",
            LineNavigation::from_current(vec![Some(1)]),
        ),
        (
            PagerLineNumberMode::Rendered,
            "1 current rendered\n",
            LineNavigation::new("1 current source\n", vec![Some(1)], vec![Some(1)]),
        ),
        (
            PagerLineNumberMode::Off,
            "current off\n",
            LineNavigation::new("1 current source\n", vec![Some(1)], vec![Some(1)]),
        ),
    ];

    for (mode, expected_output, expected_navigation) in cases {
        let document = numbered_document(mode, "current");
        let (output, navigation) = document.display_snapshot();

        assert_eq!(output, expected_output);
        assert_eq!(navigation, Some(expected_navigation));
    }
}

#[test]
fn replacing_a_document_preserves_the_selected_line_number_mode() {
    let document = RwLock::new(numbered_document(PagerLineNumberMode::Source, "current"));

    replace_document(
        &document,
        numbered_document(PagerLineNumberMode::Off, "refreshed"),
    )
    .unwrap();

    let document = document.read().unwrap();
    assert_eq!(
        document.line_number_mode(),
        Some(PagerLineNumberMode::Source)
    );
    assert_eq!(document.display_snapshot().0, "1 refreshed source\n");
}

fn numbered_document(mode: PagerLineNumberMode, label: &str) -> PagerDocument {
    PagerDocument::from_content(
        PagerContent::LineNumbers(PagerLineNumberViews::new(
            mode,
            PagerDisplay::new(format!("{label} off\n"), vec![Some(1)]),
            PagerDisplay::new(format!("1 {label} rendered\n"), vec![Some(1)]),
            PagerDisplay::new(format!("1 {label} source\n"), vec![Some(1)]),
        )),
        format!("# {label}"),
        OutputStyle::Disabled,
    )
}

#[test]
fn watcher_event_must_target_the_current_file() {
    let temp_dir = TempDir::new().unwrap();
    let current_file = temp_dir.path().join("current.md");
    let other_file = temp_dir.path().join("other.md");
    std::fs::write(&current_file, "# Current").unwrap();
    std::fs::write(&other_file, "# Other").unwrap();
    let current_target = comparable_path(&current_file).unwrap();

    let current_event =
        notify::Event::new(EventKind::Create(CreateKind::File)).add_path(current_file.clone());
    let other_event = notify::Event::new(EventKind::Create(CreateKind::File)).add_path(other_file);

    assert!(event_targets_file(&current_event, &current_target));
    assert!(!event_targets_file(&other_event, &current_target));
}

#[test]
fn pager_messages_are_single_line() {
    assert_eq!(
        single_line_message("first\nsecond\r\nthird"),
        "first second third"
    );
}

#[test]
fn clipboard_text_prefers_the_selection() {
    let document = RwLock::new(PagerDocument::new(
        "rendered output".to_string(),
        "whole source".to_string(),
        OutputStyle::Disabled,
    ));

    assert_eq!(
        clipboard_text(&document, Some("selected text".to_string())).unwrap(),
        "selected text"
    );
}

#[test]
fn clipboard_text_uses_the_source_without_a_selection() {
    let document = RwLock::new(PagerDocument::new(
        "rendered output".to_string(),
        "whole source".to_string(),
        OutputStyle::Disabled,
    ));

    assert_eq!(clipboard_text(&document, None).unwrap(), "whole source");
}

#[test]
fn active_watcher_refreshes_modified_file() {
    let temp_dir = TempDir::new().unwrap();
    let file = temp_dir.path().join("watched.md");
    std::fs::write(&file, "# Before").unwrap();
    let refresh_count = Arc::new(AtomicUsize::new(0));
    let callback_count = refresh_count.clone();
    let document = Arc::new(std::sync::RwLock::new(PagerDocument::new(
        "rendered before".to_string(),
        "# Before".to_string(),
        OutputStyle::Disabled,
    )));
    let refresh = Arc::new(move || {
        callback_count.fetch_add(1, Ordering::SeqCst);
        Ok(PagerDocument::new(
            "rendered after".to_string(),
            "# After".to_string(),
            OutputStyle::Disabled,
        ))
    });
    let watcher = ActiveWatcher::start(&file, Pager::new(), refresh, document.clone()).unwrap();

    std::fs::write(&file, "# After").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while refresh_count.load(Ordering::SeqCst) == 0 && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(20));
    }

    drop(watcher);
    assert!(refresh_count.load(Ordering::SeqCst) >= 1);
    assert_eq!(document.read().unwrap().source, "# After");
}
