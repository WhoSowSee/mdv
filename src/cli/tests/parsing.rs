use super::*;

fn parse_link_style(value: &str) -> LinkStyle {
    Cli::parse_from(["mdv", "-u", value])
        .link_style
        .expect("link style parsed")
}

fn parse_link_truncation(value: &str) -> LinkTruncationStyle {
    Cli::parse_from(["mdv", "-l", value])
        .link_truncation
        .expect("link truncation parsed")
}

#[test]
fn code_wrap_indent_flag_parses() {
    let cli = Cli::parse_from(["mdv", "--code-wrap-indent", "base"]);
    assert!(matches!(
        cli.code_wrap_indent.expect("code wrap indent value"),
        CodeWrapIndent::Base
    ));
}

#[test]
fn syntaxes_dir_flag_parses() {
    let cli = Cli::parse_from(["mdv", "--syntaxes-dir", "syntaxes"]);
    assert_eq!(cli.syntaxes_dir, Some(PathBuf::from("syntaxes")));
}

#[test]
fn short_flag_accepts_long_link_style_names() {
    assert!(matches!(parse_link_style("inline"), LinkStyle::Inline));
    assert!(matches!(
        parse_link_style("inlinetable"),
        LinkStyle::InlineTable
    ));
    assert!(matches!(parse_link_style("endtable"), LinkStyle::EndTable));
    assert!(matches!(
        parse_link_style("clickable"),
        LinkStyle::Clickable
    ));
    assert!(matches!(
        parse_link_style("fclickable"),
        LinkStyle::ClickableForced
    ));
    assert!(matches!(parse_link_style("fc"), LinkStyle::ClickableForced));
    assert!(matches!(parse_link_style("hide"), LinkStyle::Hide));
    assert!(matches!(parse_link_style("et"), LinkStyle::EndTable));
}

#[test]
fn table_smart_indent_flag_parses() {
    let cli = Cli::parse_from(["mdv", "--table-smart-indent"]);
    assert!(cli.table_smart_indent);

    let cli = Cli::parse_from(["mdv", "-S"]);
    assert!(cli.table_smart_indent);
}

#[test]
fn pretty_table_short_flag_parses() {
    let cli = Cli::parse_from(["mdv", "-B"]);
    assert!(cli.pretty_table);
}

#[test]
fn render_html_short_flag_parses() {
    let cli = Cli::parse_from(["mdv", "-E"]);
    assert!(cli.render_html);
}

#[test]
fn line_numbers_flags_parse() {
    let options = Cli::parse_from(["mdv", "-N", "separator;source", "README.md"])
        .line_numbers
        .flatten()
        .expect("line-number options");
    assert_eq!(options.target, LineNumberTarget::Source);
    assert!(options.separator);

    for invalid in ["unknown", "show-separator", "source;source", "source;"] {
        assert!(Cli::try_parse_from(["mdv", "--line-numbers", invalid]).is_err());
    }
}

#[test]
fn link_truncation_accepts_only_canonical_tablecut() {
    assert!(matches!(
        parse_link_truncation("tablecut"),
        LinkTruncationStyle::TableCut
    ));
    assert!(Cli::try_parse_from(["mdv", "-l", "table-cut"]).is_err());

    assert!(matches!(
        serde_yaml::from_str::<LinkTruncationStyle>("tablecut").expect("canonical tablecut value"),
        LinkTruncationStyle::TableCut
    ));
    assert!(serde_yaml::from_str::<LinkTruncationStyle>("table-cut").is_err());
}

#[test]
fn init_config_flag_parses() {
    let cli = Cli::parse_from(["mdv", "--init-config"]);
    assert!(cli.init_config.is_some());
    assert!(cli.init_config.unwrap().is_none());

    let cli = Cli::parse_from(["mdv", "--init-config", "."]);
    assert_eq!(cli.init_config.unwrap().unwrap(), PathBuf::from("."));
}

#[test]
fn pager_flag_parses() {
    let cli = Cli::parse_from(["mdv", "--pager"]);
    assert!(cli.pager);

    let cli = Cli::parse_from(["mdv", "-p"]);
    assert!(cli.pager);
}

#[test]
fn help_subcommand_does_not_steal_an_escaped_file_name() {
    let cli = Cli::parse_from(["mdv", "help"]);
    assert!(matches!(cli.command, Some(CliCommand::Help)));

    let cli = Cli::parse_from(["mdv", "--", "help"]);
    assert!(cli.command.is_none());
    assert_eq!(cli.filename.as_deref(), Some("help"));
}

#[test]
fn interactive_flag_parses() {
    let cli = Cli::parse_from(["mdv", "--interactive"]);
    assert!(cli.interactive);

    let cli = Cli::parse_from(["mdv", "-i"]);
    assert!(cli.interactive);
}

#[test]
fn interactive_conflicts_with_pager() {
    assert!(Cli::try_parse_from(["mdv", "--interactive", "--pager"]).is_err());
}
