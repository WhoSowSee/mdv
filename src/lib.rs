pub mod cli;
pub mod config;
pub mod error;
pub mod markdown;
pub mod monitor;
pub mod renderer;
pub mod table;
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
use std::path::Path;

/// Main entry point for the mdv application
pub fn run(mut cli: Cli, matches: &ArgMatches) -> Result<()> {
    let config = Config::from_cli(&cli, matches)?;

    if let Some(Some(path)) = &cli.theme_info {
        if cli.filename.is_none() {
            cli.filename = Some(path.to_string_lossy().into_owned());
        }
    }

    if matches!(cli.theme_info, Some(None)) {
        print_current_themes(&config);
        println!();
        theme::list_themes();
        return Ok(());
    }

    let show_current_theme = config.theme_info || cli.theme_info.is_some();

    let content = get_input_content(&cli)?;

    let processor = MarkdownProcessor::new(&config);
    let events = processor.parse(&content)?;

    let renderer = TerminalRenderer::new(&config)?;

    if cli.do_html {
        let events_clone = processor.parse(&content)?; // Re-parse for HTML
        let html_output = renderer.to_html(events_clone)?;
        print!("{}", html_output);
    } else {
        if show_current_theme {
            print_current_themes(&config);
        }

        // Add a leading blank line before content for readability
        if std::io::stdout().is_terminal() {
            println!();
        }
        let output = renderer.render(events)?;
        print!("{}", output);
    }

    if cli.monitor_file {
        if let Some(filename) = &cli.filename {
            monitor::watch_file(filename, &config)?;
        }
    }

    Ok(())
}

fn print_current_themes(config: &Config) {
    println!();
    println!("Current theme: {}", config.theme);
    println!("Current code theme: {}", current_code_theme_name(config));
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
