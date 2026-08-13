use crate::Pager;
use crate::error::MinusError;
use crate::minus_core::init;

/// Runs a pager that accepts content and configuration updates.
///
/// # Panics
/// Panics if another pager is running.
///
/// # Errors
/// Returns errors raised during setup or paging.
#[cfg_attr(docsrs, doc(cfg(feature = "dynamic_output")))]
#[allow(clippy::needless_pass_by_value)]
pub fn dynamic_paging(pager: Pager) -> Result<(), MinusError> {
    init::init_core(&pager, crate::RunMode::Dynamic, true)
}

/// Runs the dynamic pager inside the caller's active terminal screen buffer.
///
/// The caller remains responsible for entering and leaving that screen buffer.
///
/// # Panics
/// Panics if another pager is running.
///
/// # Errors
/// Returns errors raised during setup or paging.
#[cfg_attr(docsrs, doc(cfg(feature = "dynamic_output")))]
#[allow(clippy::needless_pass_by_value)]
pub fn dynamic_paging_in_place(pager: Pager) -> Result<(), MinusError> {
    init::init_core(&pager, crate::RunMode::Dynamic, false)
}
