use crate::support::mdv_cmd;

use std::fs;
use tempfile::NamedTempFile;

#[path = "tab_indent/deep_indent.rs"]
mod deep_indent;
#[path = "tab_indent/fences.rs"]
mod fences;
#[path = "tab_indent/paragraphs.rs"]
mod paragraphs;
