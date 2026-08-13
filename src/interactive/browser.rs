use super::discovery::{DiscoveryResult, DocumentEntry, filter_documents, start_discovery};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, TryRecvError};

const ITEM_HEIGHT: usize = 3;
const TOP_PADDING: usize = 5;
const BOTTOM_PADDING: usize = 3;
const MINI_HELP_HEIGHT: usize = 2;
const FULL_HELP_HEIGHT: usize = 5;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum FilterState {
    Unfiltered,
    Editing,
    Applied,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum BrowserSection {
    Documents,
    Filter,
}

pub(crate) struct BrowserState {
    root: PathBuf,
    documents: Vec<DocumentEntry>,
    filtered: Vec<usize>,
    document_selection: usize,
    filter_selection: usize,
    height: u16,
    filter_state: FilterState,
    section: BrowserSection,
    query: String,
    show_full_help: bool,
    show_error: bool,
    errors: Vec<String>,
    loaded: bool,
    receiver: Option<Receiver<DiscoveryResult>>,
}

impl BrowserState {
    pub(super) fn new(root: PathBuf, height: u16) -> Self {
        let receiver = Some(start_discovery(root.clone()));
        Self {
            root,
            documents: Vec::new(),
            filtered: Vec::new(),
            document_selection: 0,
            filter_selection: 0,
            height,
            filter_state: FilterState::Unfiltered,
            section: BrowserSection::Documents,
            query: String::new(),
            show_full_help: false,
            show_error: false,
            errors: Vec::new(),
            loaded: false,
            receiver,
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test(documents: Vec<DocumentEntry>, height: u16) -> Self {
        let filtered = (0..documents.len()).collect();
        Self {
            root: PathBuf::new(),
            documents,
            filtered,
            document_selection: 0,
            filter_selection: 0,
            height,
            filter_state: FilterState::Unfiltered,
            section: BrowserSection::Documents,
            query: String::new(),
            show_full_help: false,
            show_error: false,
            errors: Vec::new(),
            loaded: true,
            receiver: None,
        }
    }

    pub(super) fn poll_discovery(&mut self) {
        let Some(receiver) = self.receiver.take() else {
            return;
        };
        match receiver.try_recv() {
            Ok(result) => {
                self.documents = result.documents;
                self.errors = result.errors;
                self.loaded = true;
                self.update_filter();
                self.clamp_selection();
            }
            Err(TryRecvError::Empty) => self.receiver = Some(receiver),
            Err(TryRecvError::Disconnected) => self.loaded = true,
        }
    }

    pub(super) fn refresh(&mut self) {
        self.documents.clear();
        self.filtered.clear();
        self.document_selection = 0;
        self.filter_selection = 0;
        self.errors.clear();
        self.show_error = false;
        self.loaded = false;
        self.receiver = Some(start_discovery(self.root.clone()));
    }

    pub(super) fn set_height(&mut self, height: u16) {
        self.height = height;
        self.clamp_selection();
    }

    pub(super) fn documents(&self) -> &[DocumentEntry] {
        &self.documents
    }

    pub(crate) fn filter_state(&self) -> FilterState {
        self.filter_state
    }

    pub(super) fn section(&self) -> BrowserSection {
        self.section
    }

    pub(super) fn query(&self) -> &str {
        &self.query
    }

    pub(super) fn filtered_count(&self) -> usize {
        self.filtered.len()
    }

    pub(super) fn is_loaded(&self) -> bool {
        self.loaded
    }

    pub(super) fn show_full_help(&self) -> bool {
        self.show_full_help
    }

    pub(super) fn toggle_help(&mut self) {
        self.show_full_help = !self.show_full_help;
        self.clamp_selection();
    }

    pub(super) fn show_error(&self) -> bool {
        self.show_error
    }

    pub(super) fn errors(&self) -> &[String] {
        &self.errors
    }

    pub(super) fn add_error(&mut self, error: impl Into<String>) {
        self.errors.push(error.into());
    }

    pub(super) fn open_error(&mut self) {
        if !self.errors.is_empty() {
            self.show_error = true;
        }
    }

    pub(super) fn close_error(&mut self) {
        self.show_error = false;
    }

    pub(crate) fn begin_filter(&mut self) {
        self.filter_state = FilterState::Editing;
        self.filtered = filter_documents(&self.documents, &self.query);
        self.filter_selection = 0;
    }

    pub(super) fn push_filter_char(&mut self, character: char) {
        self.query.push(character);
        self.update_filter();
    }

    pub(super) fn pop_filter_char(&mut self) {
        self.query.pop();
        self.update_filter();
    }

    #[cfg(test)]
    pub(crate) fn set_filter(&mut self, query: &str) {
        self.query.clear();
        self.query.push_str(query);
        self.update_filter();
    }

    pub(super) fn cancel_filter(&mut self) {
        self.filter_state = FilterState::Unfiltered;
        self.section = BrowserSection::Documents;
        self.query.clear();
        self.filtered.clear();
        self.filter_selection = 0;
        self.clamp_selection();
    }

    pub(crate) fn confirm_filter(&mut self) {
        match self.filtered.as_slice() {
            [] => {
                self.cancel_filter();
            }
            _ if self.query.is_empty() => {
                self.cancel_filter();
            }
            _ => {
                self.filter_state = FilterState::Applied;
                self.section = BrowserSection::Filter;
                self.filter_selection = 0;
            }
        }
    }

    pub(super) fn next_section(&mut self) {
        if self.filter_state == FilterState::Applied {
            self.section = match self.section {
                BrowserSection::Documents => BrowserSection::Filter,
                BrowserSection::Filter => BrowserSection::Documents,
            };
            self.clamp_selection();
        }
    }

    pub(super) fn visible_indices(&self) -> Vec<usize> {
        if self.filter_state == FilterState::Editing || self.section == BrowserSection::Filter {
            self.filtered.clone()
        } else {
            (0..self.documents.len()).collect()
        }
    }

    pub(super) fn selected_document(&self) -> Option<&DocumentEntry> {
        let visible = self.visible_indices();
        let index = *visible.get(self.selection())?;
        self.documents.get(index)
    }

    #[cfg(test)]
    pub(crate) fn selected_path(&self) -> Option<&str> {
        self.selected_document()
            .map(|document| document.relative_path.as_str())
    }

    pub(super) fn selected_index_on_page(&self) -> usize {
        self.selection() % self.per_page()
    }

    pub(crate) fn page(&self) -> usize {
        self.selection() / self.per_page()
    }

    pub(super) fn page_count(&self) -> usize {
        self.visible_indices()
            .len()
            .max(1)
            .div_ceil(self.per_page())
    }

    pub(super) fn per_page(&self) -> usize {
        let help_height = if self.show_full_help {
            FULL_HELP_HEIGHT
        } else {
            MINI_HELP_HEIGHT
        };
        let available =
            usize::from(self.height).saturating_sub(TOP_PADDING + BOTTOM_PADDING + help_height);
        (available / ITEM_HEIGHT).max(1)
    }

    pub(super) fn move_up(&mut self) {
        let selection = self.selection_mut();
        *selection = selection.saturating_sub(1);
    }

    pub(crate) fn move_down(&mut self) {
        let last = self.visible_indices().len().saturating_sub(1);
        let selection = self.selection_mut();
        *selection = (*selection + 1).min(last);
    }

    pub(super) fn go_top(&mut self) {
        *self.selection_mut() = 0;
    }

    pub(super) fn go_bottom(&mut self) {
        *self.selection_mut() = self.visible_indices().len().saturating_sub(1);
    }

    pub(super) fn page_back(&mut self) {
        let per_page = self.per_page();
        let selection = self.selection_mut();
        *selection = selection.saturating_sub(per_page);
    }

    pub(super) fn page_forward(&mut self) {
        let per_page = self.per_page();
        let last = self.visible_indices().len().saturating_sub(1);
        let selection = self.selection_mut();
        *selection = (*selection + per_page).min(last);
    }

    fn update_filter(&mut self) {
        if self.filter_state != FilterState::Unfiltered {
            self.filtered = filter_documents(&self.documents, &self.query);
            self.filter_selection = self
                .filter_selection
                .min(self.filtered.len().saturating_sub(1));
        }
    }

    fn clamp_selection(&mut self) {
        let last = self.visible_indices().len().saturating_sub(1);
        let selection = self.selection_mut();
        *selection = (*selection).min(last);
    }

    fn selection(&self) -> usize {
        if self.filter_state == FilterState::Editing || self.section == BrowserSection::Filter {
            self.filter_selection
        } else {
            self.document_selection
        }
    }

    fn selection_mut(&mut self) -> &mut usize {
        if self.filter_state == FilterState::Editing || self.section == BrowserSection::Filter {
            &mut self.filter_selection
        } else {
            &mut self.document_selection
        }
    }
}
