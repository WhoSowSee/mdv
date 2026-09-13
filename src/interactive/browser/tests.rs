use super::*;
use std::sync::mpsc;

#[test]
fn discovery_updates_the_browser_before_the_scan_finishes() {
    let (sender, receiver) = mpsc::channel();
    let mut browser = BrowserState::with_discovery_for_test(receiver, 20);

    sender
        .send(DiscoveryEvent::Document(DocumentEntry::for_test("zeta.md")))
        .unwrap();
    browser.poll_discovery();

    assert_eq!(browser.selected_path(), Some("zeta.md"));
    assert!(!browser.is_loaded());

    sender
        .send(DiscoveryEvent::Document(DocumentEntry::for_test(
            "alpha.md",
        )))
        .unwrap();
    browser.poll_discovery();

    let paths: Vec<_> = browser
        .documents()
        .iter()
        .map(|document| document.relative_path.as_str())
        .collect();
    assert_eq!(paths, ["alpha.md", "zeta.md"]);
    assert_eq!(browser.selected_path(), Some("zeta.md"));

    sender.send(DiscoveryEvent::Finished).unwrap();
    browser.poll_discovery();

    assert!(browser.is_loaded());
}

#[test]
fn rediscovery_replaces_documents_only_after_it_finishes() {
    let (sender, receiver) = mpsc::channel();
    let mut browser = BrowserState::for_test(vec![DocumentEntry::for_test("old.md")], 20);
    browser.loaded = false;
    browser.receiver = Some(receiver);
    browser.replacement_documents = Some(Vec::new());

    sender
        .send(DiscoveryEvent::Document(DocumentEntry::for_test("new.md")))
        .unwrap();
    browser.poll_discovery();

    assert_eq!(browser.selected_path(), Some("old.md"));

    sender.send(DiscoveryEvent::Finished).unwrap();
    browser.poll_discovery();

    assert_eq!(browser.selected_path(), Some("new.md"));
}

#[test]
fn rapid_ignore_toggles_defer_discovery() {
    let mut browser = BrowserState::for_test(Vec::new(), 20);

    browser.toggle_ignored_files();
    browser.toggle_ignored_files();

    assert!(browser.receiver.is_none());
    assert!(browser.discovery_starts_at.is_some());
}
