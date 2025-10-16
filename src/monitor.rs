use crate::config::Config;
use crate::error::MdvError;
use crate::markdown::MarkdownProcessor;
use crate::renderer::TerminalRenderer;
use anyhow::Result;
use notify::{Event as NotifyEvent, EventKind, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// Watch a single file for changes and re-render on change
pub fn watch_file(filename: &str, config: &Config) -> Result<()> {
    let path = PathBuf::from(filename);
    if !path.exists() {
        return Err(MdvError::MonitorError(format!("File not found: {}", filename)).into());
    }

    println!("Monitoring file: {} (Press Ctrl+C to stop)", filename);

    let renderer = TerminalRenderer::new(config)?;

    let (tx, rx) = mpsc::channel();
    let mut watcher =
        notify::recommended_watcher(tx).map_err(|e| MdvError::MonitorError(e.to_string()))?;

    watcher
        .watch(&path, RecursiveMode::NonRecursive)
        .map_err(|e| MdvError::MonitorError(e.to_string()))?;

    render_file(&path, config, &renderer)?;

    let mut last_render = Instant::now();
    let debounce_duration = Duration::from_millis(100);

    loop {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(event) => {
                if let Ok(event) = event {
                    if should_trigger_render(&event) {
                        let now = Instant::now();
                        if now.duration_since(last_render) > debounce_duration {
                            println!("\n--- File changed, re-rendering ---\n");
                            if let Err(e) = render_file(&path, config, &renderer) {
                                eprintln!("Error rendering file: {}", e);
                            }
                            last_render = now;
                        }
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                break;
            }
        }
    }

    Ok(())
}

fn should_trigger_render(event: &NotifyEvent) -> bool {
    matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_))
}

fn render_file(path: &Path, config: &Config, renderer: &TerminalRenderer) -> Result<()> {
    let content = std::fs::read_to_string(path)?;

    let processor = MarkdownProcessor::new(config);
    let events = processor.parse(&content)?;

    let output = renderer.render(events)?;

    print!("{}", output);
    Ok(())
}
