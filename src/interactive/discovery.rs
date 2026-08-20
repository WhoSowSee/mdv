use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::time::SystemTime;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

const MARKDOWN_EXTENSIONS: &[&str] = &["md", "mdown", "mkdn", "mkd", "markdown"];
const DISCOVERY_CHANNEL_CAPACITY: usize = 256;

#[derive(Debug, Clone)]
pub(crate) struct DocumentEntry {
    pub(crate) path: PathBuf,
    pub(crate) relative_path: String,
    pub(crate) modified: SystemTime,
    filter_value: String,
}

impl DocumentEntry {
    fn new(path: PathBuf, root: &Path, modified: SystemTime) -> Self {
        let relative_path = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let filter_value = normalize(&relative_path);
        Self {
            path,
            relative_path,
            modified,
            filter_value,
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test(relative_path: &str) -> Self {
        Self {
            path: PathBuf::from(relative_path),
            relative_path: relative_path.to_string(),
            modified: SystemTime::UNIX_EPOCH,
            filter_value: normalize(relative_path),
        }
    }

    pub(crate) fn match_indices(&self, query: &str) -> Vec<usize> {
        fuzzy_match_indices(&self.relative_path, query)
    }
}

#[cfg(test)]
#[derive(Debug, Default)]
pub(crate) struct DiscoveryResult {
    pub(crate) documents: Vec<DocumentEntry>,
    pub(crate) errors: Vec<String>,
}

pub(crate) enum DiscoveryEvent {
    Document(DocumentEntry),
    Error(String),
    Finished,
}

pub(crate) fn start_discovery(root: PathBuf) -> Receiver<DiscoveryEvent> {
    let (sender, receiver) = mpsc::sync_channel(DISCOVERY_CHANNEL_CAPACITY);
    std::thread::spawn(move || {
        let completed = walk_paths(&root, |event| sender.send(event).is_ok());
        if completed {
            let _ = sender.send(DiscoveryEvent::Finished);
        }
    });
    receiver
}

#[cfg(test)]
pub(crate) fn discover_paths(root: &Path) -> DiscoveryResult {
    let mut result = DiscoveryResult::default();
    walk_paths(root, |event| {
        match event {
            DiscoveryEvent::Document(document) => result.documents.push(document),
            DiscoveryEvent::Error(error) => result.errors.push(error),
            DiscoveryEvent::Finished => {}
        }
        true
    });
    result
        .documents
        .sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    result
}

fn walk_paths(root: &Path, mut emit: impl FnMut(DiscoveryEvent) -> bool) -> bool {
    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .require_git(false)
        .follow_links(false)
        .filter_entry(|entry| entry.file_name() != "node_modules");

    for entry in builder.build() {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                if !emit(DiscoveryEvent::Error(error.to_string())) {
                    return false;
                }
                continue;
            }
        };
        let Some(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_file() || !is_markdown_path(entry.path()) {
            continue;
        }

        match entry.metadata() {
            Ok(metadata) => {
                let document = DocumentEntry::new(
                    entry.into_path(),
                    root,
                    metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                );
                if !emit(DiscoveryEvent::Document(document)) {
                    return false;
                }
            }
            Err(error) => {
                if !emit(DiscoveryEvent::Error(error.to_string())) {
                    return false;
                }
            }
        }
    }
    true
}

pub(crate) fn filter_documents(documents: &[DocumentEntry], query: &str) -> Vec<usize> {
    if query.is_empty() {
        return (0..documents.len()).collect();
    }

    let query = normalize(query);
    let matcher = SkimMatcherV2::default();
    let mut matches: Vec<_> = documents
        .iter()
        .enumerate()
        .filter_map(|(index, document)| {
            matcher
                .fuzzy_match(&document.filter_value, &query)
                .map(|score| (index, score))
        })
        .collect();
    matches.sort_by(|(left_index, left_score), (right_index, right_score)| {
        right_score
            .cmp(left_score)
            .then_with(|| left_index.cmp(right_index))
    });
    matches.into_iter().map(|(index, _)| index).collect()
}

fn fuzzy_match_indices(text: &str, query: &str) -> Vec<usize> {
    if query.is_empty() {
        return Vec::new();
    }

    let (normalized_text, original_indices) = normalize_with_indices(text);
    let matcher = SkimMatcherV2::default();
    let normalized_query = normalize(query);
    let Some((_, indices)) = matcher.fuzzy_indices(&normalized_text, &normalized_query) else {
        return Vec::new();
    };
    let mut matches: Vec<_> = indices
        .into_iter()
        .filter_map(|index| original_indices.get(index).copied())
        .collect();
    matches.sort_unstable();
    matches.dedup();
    matches
}

fn is_markdown_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            MARKDOWN_EXTENSIONS
                .iter()
                .any(|candidate| extension.eq_ignore_ascii_case(candidate))
        })
}

fn normalize(text: &str) -> String {
    text.nfd()
        .filter(|character| !is_combining_mark(*character))
        .flat_map(char::to_lowercase)
        .collect()
}

fn normalize_with_indices(text: &str) -> (String, Vec<usize>) {
    let mut normalized = String::new();
    let mut original_indices = Vec::new();
    for (original_index, character) in text.chars().enumerate() {
        for decomposed in character.to_string().nfd() {
            if is_combining_mark(decomposed) {
                continue;
            }
            for lowercase in decomposed.to_lowercase() {
                normalized.push(lowercase);
                original_indices.push(original_index);
            }
        }
    }
    (normalized, original_indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_indices_point_to_original_characters_after_normalization() {
        assert_eq!(
            fuzzy_match_indices("docs/résumé.md", "RESUME"),
            [5, 6, 7, 8, 9, 10]
        );
    }
}
