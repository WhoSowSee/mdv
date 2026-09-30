use super::{
    CapturedReferenceBlock, DeferredLinkReferenceBlock, FootnoteDefinitions, FootnoteTextState,
};
use std::collections::HashMap;

#[derive(Default)]
pub(in crate::renderer::event) struct FootnoteState {
    pub(in crate::renderer::event) definitions: FootnoteDefinitions,
    pub(in crate::renderer::event) order: Vec<String>,
    pub(in crate::renderer::event) inline: Vec<String>,
    pub(in crate::renderer::event) use_count: HashMap<String, usize>,
    pub(in crate::renderer::event) suppress_output: bool,
    pub(in crate::renderer::event) text_state: FootnoteTextState,
    pub(in crate::renderer::event) text_buffer: String,
}

#[derive(Default)]
pub(in crate::renderer::event) struct CodeState {
    pub(in crate::renderer::event) active: Option<CodeBlock>,
    pub(in crate::renderer::event) max_line_number_width: usize,
    pub(in crate::renderer::event) plaintext_depth: usize,
    pub(in crate::renderer::event) captured_references: Vec<CapturedReferenceBlock>,
    pub(in crate::renderer::event) deferred_references: Vec<DeferredLinkReferenceBlock>,
}

#[derive(Default)]
pub(in crate::renderer::event) struct CodeBlock {
    pub(in crate::renderer::event) content: String,
    pub(in crate::renderer::event) language: Option<String>,
    pub(in crate::renderer::event) source_line: Option<usize>,
}

#[derive(Default)]
pub(in crate::renderer::event) struct LinkState {
    pub(in crate::renderer::event) current: Option<CurrentLink>,
    pub(in crate::renderer::event) paragraph_counter: usize,
    pub(in crate::renderer::event) paragraph: Vec<(String, String)>,
    pub(in crate::renderer::event) document: Vec<(String, String)>,
}

pub(in crate::renderer::event) enum LinkDestination {
    Url(String),
    Reference,
    Label,
}

pub(in crate::renderer::event) struct CurrentLink {
    pub(in crate::renderer::event) text: String,
    pub(in crate::renderer::event) destination: LinkDestination,
}

impl CurrentLink {
    pub(in crate::renderer::event) fn new(destination: LinkDestination) -> Self {
        Self {
            text: String::new(),
            destination,
        }
    }

    pub(in crate::renderer::event) fn into_direct(self) -> anyhow::Result<(String, String)> {
        let LinkDestination::Url(url) = self.destination else {
            anyhow::bail!("Direct link rendering requires a URL");
        };
        Ok((self.text, url))
    }
}
