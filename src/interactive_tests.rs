use crate::interactive::browser::BrowserState;
use crate::interactive::discovery::{DocumentEntry, discover_paths, filter_documents};
use crate::interactive::{InteractiveTarget, select_interactive_target};
use std::fs;
use tempfile::TempDir;

#[test]
fn directory_arguments_open_the_browser_without_an_explicit_flag() {
    let directory = TempDir::new().unwrap();
    let target = select_interactive_target(
        Some(directory.path().to_string_lossy().as_ref()),
        false,
        false,
        true,
    )
    .unwrap();

    assert_eq!(
        target,
        Some(InteractiveTarget::Directory(
            directory.path().canonicalize().unwrap()
        ))
    );
}

#[test]
fn regular_files_require_the_interactive_flag() {
    let file = tempfile::NamedTempFile::new().unwrap();
    let filename = file.path().to_string_lossy();

    assert_eq!(
        select_interactive_target(Some(filename.as_ref()), false, false, true).unwrap(),
        None
    );
    assert_eq!(
        select_interactive_target(Some(filename.as_ref()), true, false, true).unwrap(),
        Some(InteractiveTarget::File(file.path().canonicalize().unwrap()))
    );
}

#[test]
fn pager_mode_never_selects_an_interactive_target() {
    assert_eq!(
        select_interactive_target(None, false, true, true).unwrap(),
        None
    );
}

#[test]
fn discovery_toggles_git_ignored_files_and_respects_other_filters() {
    let directory = TempDir::new().unwrap();
    fs::create_dir(directory.path().join("notes")).unwrap();
    fs::create_dir(directory.path().join("node_modules")).unwrap();
    fs::create_dir_all(directory.path().join(".git").join("info")).unwrap();
    fs::write(directory.path().join("README.md"), "# Visible").unwrap();
    fs::write(
        directory.path().join("notes").join("guide.markdown"),
        "# Guide",
    )
    .unwrap();
    fs::write(directory.path().join("ignored.md"), "# Ignored").unwrap();
    fs::write(
        directory.path().join("locally-ignored.md"),
        "# Locally ignored",
    )
    .unwrap();
    fs::write(directory.path().join(".hidden.md"), "# Hidden").unwrap();
    fs::write(
        directory.path().join("node_modules").join("package.md"),
        "# Package",
    )
    .unwrap();
    fs::write(directory.path().join(".gitignore"), "ignored.md\n").unwrap();
    fs::write(
        directory.path().join(".git").join("info").join("exclude"),
        "locally-ignored.md\n",
    )
    .unwrap();

    let discover = |include_git_ignored| {
        let result = discover_paths(directory.path(), include_git_ignored);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        result
            .documents
            .into_iter()
            .map(|document| document.relative_path)
            .collect::<Vec<_>>()
    };

    assert_eq!(discover(false), ["README.md", "notes/guide.markdown"]);
    assert_eq!(
        discover(true),
        [
            "README.md",
            "ignored.md",
            "locally-ignored.md",
            "notes/guide.markdown",
        ]
    );
}

#[test]
fn filtering_is_case_insensitive_and_diacritic_insensitive() {
    let documents = [
        DocumentEntry::for_test("docs/résumé.md"),
        DocumentEntry::for_test("docs/result.md"),
        DocumentEntry::for_test("notes.md"),
    ];

    let matches = filter_documents(&documents, "RESUME");

    assert_eq!(matches, [0]);
}

#[test]
fn browser_navigation_crosses_page_boundaries_without_losing_selection() {
    let documents = (0..7)
        .map(|index| DocumentEntry::for_test(&format!("document-{index}.md")))
        .collect();
    let mut browser = BrowserState::for_test(documents, 20);

    for _ in 0..4 {
        browser.move_down();
    }

    assert_eq!(browser.selected_path().unwrap(), "document-4.md");
    assert_eq!(browser.page(), 1);
}

#[test]
fn confirming_a_single_filter_match_applies_the_filter_without_opening_it() {
    let documents = [
        DocumentEntry::for_test("alpha.md"),
        DocumentEntry::for_test("beta.md"),
    ];
    let mut browser = BrowserState::for_test(documents.into(), 20);

    browser.begin_filter();
    browser.set_filter("bet");

    browser.confirm_filter();
    assert_eq!(
        browser.filter_state(),
        crate::interactive::browser::FilterState::Applied
    );
    assert_eq!(browser.selected_path(), Some("beta.md"));
}
