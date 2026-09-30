use std::{fmt, sync::Arc};

/// Produces source-numbered text and its source-line map when navigation begins.
pub type SourceViewRenderer =
    Arc<dyn Fn() -> Result<(String, Vec<Option<usize>>), String> + Send + Sync>;

#[derive(Clone)]
pub(super) struct DeferredSource(pub(super) SourceViewRenderer);

impl fmt::Debug for DeferredSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("DeferredSource")
    }
}

impl PartialEq for DeferredSource {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for DeferredSource {}
