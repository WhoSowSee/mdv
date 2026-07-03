mod callout;
mod checkbox;
mod checkbox_override;
pub mod cli;
pub mod config;
mod custom_code_block;
mod editor;
pub mod error;
pub mod markdown;
pub mod math;
pub mod monitor;
mod pager;
pub mod renderer;
pub mod table;
mod user_themes;
pub mod terminal;
pub mod theme;
pub mod utils;

use anyhow::Result;
use clap::ArgMatches;
use cli::Cli;
use config::Config;
use markdown::MarkdownProcessor;
use renderer::TerminalRenderer;
use std::io::IsTerminal;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Main entry point for the mdv application
pub fn run(mut cli: Cli, matches: &ArgMatches) -> Result<()> {
    if cli.init_config.is_some() {
        let path = Config::write_default_config(&cli, matches)?;
        println!("Created config file: {}", path.display());
        return Ok(());
    }

    let config = Config::from_cli(&cli, matches)?;
    if let Some(Some(path)) = &cli.theme_info
        && cli.filename.is_none()
    {
        cli.filename = Some(path.to_string_lossy().into_owned());
    }

    if matches!(cli.theme_info, Some(None)) {
        let theme_manager = renderer::terminal::build_theme_manager(&config);
        print_current_themes(&config);
        println!();
        theme::list_themes(&theme_manager);
        return Ok(());
    }

    let show_current_theme = config.theme_info || cli.theme_info.is_some();

    let content = get_input_content(&cli)?;
    let stdout_is_terminal = std::io::stdout().is_terminal();
    let output = render_document(
        &content,
        &config,
        cli.do_html,
        show_current_theme,
        stdout_is_terminal,
    )?;

    let pager_active = cli.pager && stdout_is_terminal;
    if pager_active {
        let pager_file = cli
            .filename
            .as_deref()
            .filter(|filename| *filename != "-")
            .map(PathBuf::from);
        let refresh = pager_file.as_ref().map(|path| {
            let path = path.clone();
            let config = config.clone();
            let do_html = cli.do_html;
            Arc::new(move || render_document_file(&path, &config, do_html, show_current_theme))
                as pager::RefreshCallback
        });
        pager::page(output, pager_file, refresh)?;
    } else {
        print!("{}", output);
    }

    if cli.monitor_file
        && !pager_active
        && let Some(filename) = &cli.filename
    {
        monitor::watch_file(filename, &config)?;
    }

    Ok(())
}

fn render_document(
    content: &str,
    config: &Config,
    do_html: bool,
    show_current_theme: bool,
    add_leading_blank: bool,
) -> Result<String> {
    let processor = MarkdownProcessor::new(config);
    let events = processor.parse(content)?;
    let renderer = TerminalRenderer::new(config)?;

    if do_html {
        return renderer.to_html(events);
    }

    let mut output = String::new();
    if show_current_theme {
        output.push_str(&format_current_themes(config));
    }
    if add_leading_blank {
        output.push('\n');
    }
    output.push_str(&renderer.render(events)?);
    Ok(output)
}

fn render_document_file(
    path: &Path,
    config: &Config,
    do_html: bool,
    show_current_theme: bool,
) -> Result<String> {
    let mut content = std::fs::read_to_string(path)?;
    strip_leading_bom(&mut content);
    render_document(&content, config, do_html, show_current_theme, true)
}

fn format_current_themes(config: &Config) -> String {
    let mut result = String::new();
    result.push('\n');
    result.push_str(&format!("Current theme: {}\n", config.theme));
    result.push_str(&format!(
        "Current code theme: {}\n",
        current_code_theme_name(config)
    ));
    result
}

fn print_current_themes(config: &Config) {
    print!("{}", format_current_themes(config));
}

fn current_code_theme_name(config: &Config) -> String {
    config
        .code_theme
        .clone()
        .unwrap_or_else(|| config.theme.clone())
}

fn get_input_content(cli: &Cli) -> Result<String> {
    let mut content = match &cli.filename {
        Some(filename) if filename == "-" => {
            let mut content = String::new();
            io::stdin().read_to_string(&mut content)?;
            content
        }
        Some(filename) => {
            let path = Path::new(filename);
            if !path.exists() {
                anyhow::bail!("File not found: {}", filename);
            }
            std::fs::read_to_string(path)?
        }
        None => {
            if io::stdin().is_terminal() {
                anyhow::bail!(
                    "No input file: provide a file path or pipe content via stdin"
                );
            }
            let mut content = String::new();
            io::stdin().read_to_string(&mut content)?;
            content
        }
    };

    strip_leading_bom(&mut content);
    Ok(content)
}

fn strip_leading_bom(text: &mut String) {
    const UTF8_BOM: char = '\u{FEFF}';
    while text.starts_with(UTF8_BOM) {
        // Standard PowerShell adds a UTF-8 BOM when piping text.
        let bom_len = UTF8_BOM.len_utf8();
        text.drain(..bom_len);
    }
}
