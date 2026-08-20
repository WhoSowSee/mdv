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
