use crate::{LineNavigation, PagerState, PromptError};
use std::sync::Arc;

/// Rebuilds content and source maps for the available document width.
/// Runs synchronously while pager state is locked; must not call pager state APIs.
pub type LayoutRenderer =
    Arc<dyn Fn(usize) -> Result<(String, Option<LineNavigation>), String> + Send + Sync>;

impl PagerState {
    pub(crate) fn refresh_layout(&mut self) -> Result<bool, PromptError> {
        let Some(renderer) = &self.layout_renderer else {
            return Ok(false);
        };
        let (text, navigation) = renderer(self.content_columns()).map_err(PromptError::Layout)?;
        self.replace_mapped_text(text, navigation)?;
        Ok(true)
    }
}
