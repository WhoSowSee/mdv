use crate::config::Config;
use crate::document::{RenderOptions, read_document_source, render_document_with_renderer};
use crate::error::MdvError;
use crate::renderer::TerminalRenderer;
use crate::terminal::OutputStyle;
use anyhow::Result;
use notify::{Event as NotifyEvent, EventKind, RecursiveMode, Watcher};
use std::io::{self, Write};
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// Watch a file, printing its initial terminal document and refreshed snapshots.
pub fn watch_file(filename: &str, config: &Config, output_style: OutputStyle) -> Result<()> {
    let path = monitor_path(filename)?;
    let content = read_document_source(path)?;
    watch_file_with_options(
        filename,
        config,
        output_style,
        RenderOptions::default(),
        &content,
    )
}

pub(crate) fn watch_file_with_options(
    filename: &str,
    config: &Config,
    output_style: OutputStyle,
    options: RenderOptions<'_>,
    initial_content: &str,
) -> Result<()> {
    let path = monitor_path(filename)?;
    let renderer = TerminalRenderer::new(config, output_style)?;

    let (tx, rx) = mpsc::channel();
    let mut watcher =
        notify::recommended_watcher(tx).map_err(|e| MdvError::MonitorError(e.to_string()))?;
    watcher
        .watch(path, RecursiveMode::NonRecursive)
        .map_err(|e| MdvError::MonitorError(e.to_string()))?;

    print_document(initial_content, config, &renderer, options)?;
    crate::output::write_stdout(&format!(
        "Monitoring file: {} (Press Ctrl+C to stop)\n",
        filename
    ))?;

    let debounce_duration = Duration::from_millis(100);
    let mut next_render = None;
    loop {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(Ok(event)) if should_trigger_render(&event) => {
                next_render = Some(Instant::now() + debounce_duration);
            }
            Ok(_) | Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }

        if next_render.is_some_and(|deadline| Instant::now() >= deadline) {
            next_render = None;
            crate::output::write_stdout("\n--- File changed, re-rendering ---\n\n")?;
            if let Err(error) = render_file(path, config, &renderer, options) {
                if error
                    .downcast_ref::<io::Error>()
                    .is_some_and(|error| error.kind() == io::ErrorKind::BrokenPipe)
                {
                    return Err(error);
                }
                writeln!(io::stderr().lock(), "Error rendering file: {}", error)?;
            }
        }
    }

    Ok(())
}

fn monitor_path(filename: &str) -> Result<&Path> {
    let path = Path::new(filename);
    if !path.exists() {
        return Err(MdvError::MonitorError(format!("File not found: {}", filename)).into());
    }

    Ok(path)
}

fn should_trigger_render(event: &NotifyEvent) -> bool {
    matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_))
}

fn render_file(
    path: &Path,
    config: &Config,
    renderer: &TerminalRenderer,
    options: RenderOptions<'_>,
) -> Result<()> {
    let content = read_document_source(path)?;
    print_document(&content, config, renderer, options)
}

fn print_document(
    content: &str,
    config: &Config,
    renderer: &TerminalRenderer,
    options: RenderOptions<'_>,
) -> Result<()> {
    let rendered = render_document_with_renderer(content, config, renderer, options)?;
    Ok(crate::output::write_stdout(rendered.output()?)?)
}
