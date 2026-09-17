use crate::support::mdv_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

#[path = "layout/blockquotes.rs"]
mod blockquotes;
#[path = "layout/headings.rs"]
mod headings;
#[path = "layout/margins.rs"]
mod margins;
#[path = "layout/spacing.rs"]
mod spacing;
