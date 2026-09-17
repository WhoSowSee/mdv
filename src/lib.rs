pub mod block_spacing;
mod callout;
mod checkbox;
mod checkbox_override;
pub mod cli;
pub mod config;
mod custom_code_block;
mod document;
mod editor;
pub mod error;
pub mod inline_style;
mod interactive;
mod list_marker;
pub mod markdown;
pub mod math;
pub mod monitor;
mod pager;
mod preset;
pub mod renderer;
pub mod table;
pub mod terminal;
pub mod theme;
mod user_themes;
pub mod utils;

#[cfg(test)]
mod interactive_tests;

pub use list_marker::{PrettyListStyle, UniformListMarker};

use anyhow::Result;
use clap::{ArgMatches, CommandFactory};
use cli::{Cli, CliCommand, OutputStyle};
use config::Config;
use document::{RenderOptions, format_current_themes, render_document, render_document_file};
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
    let stdout_is_terminal = io::stdout().is_terminal();
    let output_style = config.color.resolve(stdout_is_terminal);
    if matches!(cli.command, Some(CliCommand::Help)) {
        return show_help(&config, output_style);
    }
    if let Some(Some(path)) = &cli.theme_info
        && cli.filename.is_none()
    {
        cli.filename = Some(path.to_string_lossy().into_owned());
    }

    if cli.preset_info && cli.filename.is_none() {
        print!(
            "{}",
            preset::format_available_presets(&config, cli.preset.as_deref())?
        );
        return Ok(());
    }

    if matches!(cli.theme_info, Some(None)) {
        let theme_manager = renderer::terminal::build_theme_manager(&config);
        print!("{}", format_current_themes(&config));
        println!();
        theme::list_themes(&theme_manager);
        return Ok(());
    }

    let show_current_theme = config.theme_info || cli.theme_info.is_some();
    let current_preset = cli.preset.as_deref().filter(|_| cli.preset_info);

    let stdin_is_terminal = io::stdin().is_terminal();
    if let Some(target) = interactive::select_interactive_target(
        cli.filename.as_deref(),
        cli.interactive,
        cli.pager,
        stdin_is_terminal,
    )? {
        return interactive::run(target, config, output_style);
    }

    let content = get_input_content(&cli)?;
    let pager_active = cli.pager && stdout_is_terminal;
    let rendered = render_document(
        &content,
        &config,
        output_style,
        RenderOptions {
            do_html: cli.do_html,
            show_current_theme,
            current_preset,
            add_leading_blank: stdout_is_terminal,
            for_pager: pager_active,
        },
    )?;

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
            let current_preset = current_preset.map(str::to_owned);
            Arc::new(move || {
                render_document_file(
                    &path,
                    &config,
                    do_html,
                    show_current_theme,
                    current_preset.as_deref(),
                    output_style,
                )
            }) as pager::RefreshCallback
        });
        pager::page(
            rendered.into_pager_document(content),
            pager_file,
            refresh,
            pager::PagerScreen::Alternate,
        )?;
    } else {
        print!("{}", rendered.output());
    }

    if cli.monitor_file
        && !pager_active
        && let Some(filename) = &cli.filename
    {
        monitor::watch_file(filename, &config, output_style)?;
    }

    Ok(())
}

fn show_help(config: &Config, output_style: OutputStyle) -> Result<()> {
    let mut command = Cli::command();
    if let Some(bin_name) = std::env::args_os()
        .next()
        .and_then(|arg| PathBuf::from(arg).file_name().map(|name| name.to_owned()))
        .and_then(|name| name.into_string().ok())
    {
        command = command.bin_name(bin_name);
    }
    let help = command.render_long_help().to_string();
    if io::stdin().is_terminal() && io::stdout().is_terminal() {
        pager::page(
            build_help_document(config, output_style, help)?,
            None,
            None,
            pager::PagerScreen::Alternate,
        )
    } else {
        print!("{help}");
        Ok(())
    }
}

fn build_help_document(
    config: &Config,
    output_style: OutputStyle,
    help: String,
) -> Result<pager::PagerDocument> {
    let status_bar_transparent = renderer::terminal::pager_status_bar_transparent(config)?;
    Ok(pager::PagerDocument::new(help.clone(), help, output_style)
        .with_title("Help")
        .with_status_bar_transparent(status_bar_transparent))
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
                anyhow::bail!("No input file: provide a file path or pipe content via stdin");
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn help_document_uses_configured_theme_transparency() {
        let temp_dir = TempDir::new().unwrap();
        let themes_dir = temp_dir.path().join("themes");
        std::fs::create_dir(&themes_dir).unwrap();
        std::fs::write(temp_dir.path().join("config.yaml"), "theme: transparent\n").unwrap();
        std::fs::write(
            themes_dir.join("transparent.yaml"),
            "name: transparent\npager_status_bar_transparent: true\n",
        )
        .unwrap();
        let config = Config {
            theme: "transparent".to_string(),
            config_dir: Some(temp_dir.path().to_path_buf()),
            ..Config::default()
        };
        let document =
            build_help_document(&config, OutputStyle::Disabled, "help".to_string()).unwrap();

        assert!(document.status_bar_transparent());
    }
}
