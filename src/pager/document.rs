use super::*;

pub(crate) struct PagerDocument {
    pub(in crate::pager) output: String,
    pub(in crate::pager) source: String,
    pub(in crate::pager) title: Option<String>,
    status_bar_transparent: bool,
}

impl PagerDocument {
    pub(crate) fn new(output: String, source: String) -> Self {
        Self {
            output,
            source,
            title: None,
            status_bar_transparent: false,
        }
    }

    pub(crate) fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub(crate) const fn with_status_bar_transparent(mut self, transparent: bool) -> Self {
        self.status_bar_transparent = transparent;
        self
    }

    pub(crate) const fn status_bar_transparent(&self) -> bool {
        self.status_bar_transparent
    }
}

pub(crate) type RefreshCallback = Arc<dyn Fn() -> Result<PagerDocument> + Send + Sync>;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum PagerScreen {
    Alternate,
    InPlace,
}
