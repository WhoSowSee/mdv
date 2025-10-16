use super::TerminalRenderer;
use crate::config::Config;
use pulldown_cmark::{Event, Options, Parser};

#[test]
fn test_renderer_creation() {
    let config = Config::default();
    let renderer = TerminalRenderer::new(&config);
    assert!(renderer.is_ok());
}

#[test]
fn test_basic_rendering() {
    let config = Config::default();
    let renderer = TerminalRenderer::new(&config).unwrap();

    let markdown = "# Hello\n\nThis is **bold** text.";
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(markdown, options);
    let events: Vec<Event> = parser.collect();

    let result = renderer.render(events);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(!output.is_empty());
}
