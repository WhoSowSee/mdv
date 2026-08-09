//! Static paging runner.
use crate::minus_core::init;
use crate::{Pager, error::MinusError};

/// Pages preloaded content, writing it directly when paging is unnecessary.
///
/// [`Pager::set_run_no_overflow`] can force the pager to open for content that fits on screen.
///
/// # Panics
/// Panics if another pager is running.
///
/// # Errors
/// Returns errors raised during setup or paging.
#[cfg_attr(docsrs, doc(cfg(feature = "static_output")))]
#[allow(clippy::needless_pass_by_value)]
pub fn page_all(pager: Pager) -> Result<(), MinusError> {
    init::init_core(&pager, crate::RunMode::Static)
}
