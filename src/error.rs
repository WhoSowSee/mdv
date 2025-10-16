use thiserror::Error;

#[derive(Error, Debug)]
pub enum MdvError {
    #[error("Configuration parse error: {0}")]
    ConfigParseError(String),

    #[error("Theme error: {0}")]
    ThemeError(String),

    #[error("Rendering error: {0}")]
    RenderError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Markdown parsing error: {0}")]
    MarkdownError(String),

    #[error("Terminal error: {0}")]
    TerminalError(String),

    #[error("Monitor error: {0}")]
    MonitorError(String),

    #[error("Syntax highlighting error: {0}")]
    SyntaxError(String),
}
