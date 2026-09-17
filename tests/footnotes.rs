use crate::support::mdv_cmd;
use mdv::utils::strip_ansi;
use std::fs;
use tempfile::NamedTempFile;

#[path = "footnotes/attached.rs"]
mod attached;
#[path = "footnotes/ordering.rs"]
mod ordering;
#[path = "footnotes/placement.rs"]
mod placement;
#[path = "footnotes/validation.rs"]
mod validation;
