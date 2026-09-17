use crate::support::mdv_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

#[path = "links_tables/basic.rs"]
mod basic;
#[path = "links_tables/code_blocks.rs"]
mod code_blocks;
#[path = "links_tables/references.rs"]
mod references;
#[path = "links_tables/smart_indent.rs"]
mod smart_indent;
#[path = "links_tables/truncation.rs"]
mod truncation;
