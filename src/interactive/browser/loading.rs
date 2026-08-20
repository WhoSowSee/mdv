use super::*;
use crate::interactive::discovery::start_discovery;
use std::path::Path;
use std::sync::mpsc::TryRecvError;
use std::time::Duration;

const DISCOVERY_EVENTS_PER_TICK: usize = 128;

impl BrowserState {
    pub(in crate::interactive) fn new(root: PathBuf, height: u16) -> Self {
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
            loading_started: Instant::now(),
        }
    }

    #[cfg(test)]
    pub(super) fn with_discovery_for_test(receiver: Receiver<DiscoveryEvent>, height: u16) -> Self {
        let mut browser = Self::for_test(Vec::new(), height);
        browser.loaded = false;
        browser.receiver = Some(receiver);
        browser
    }

    pub(in crate::interactive) fn poll_discovery(&mut self) {
        let selected_path = self
            .selected_document()
            .map(|document| document.path.clone());
        let Some(receiver) = self.receiver.take() else {
            return;
        };
        let mut documents_changed = false;
        let mut finished = false;

        for _ in 0..DISCOVERY_EVENTS_PER_TICK {
            match receiver.try_recv() {
                Ok(DiscoveryEvent::Document(document)) => {
                    self.documents.push(document);
                    documents_changed = true;
                }
                Ok(DiscoveryEvent::Error(error)) => self.errors.push(error),
                Ok(DiscoveryEvent::Finished) => {
                    self.loaded = true;
                    finished = true;
                    break;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.loaded = true;
                    finished = true;
                    break;
                }
            }
        }

        if !finished {
            self.receiver = Some(receiver);
        }
        if documents_changed {
            self.documents
                .sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
            self.update_filter();
            self.restore_selection(selected_path.as_deref());
        }
    }

    pub(in crate::interactive) fn refresh(&mut self) {
        self.documents.clear();
        self.filtered.clear();
        self.document_selection = 0;
        self.filter_selection = 0;
        self.errors.clear();
        self.show_error = false;
        self.loaded = false;
        self.loading_started = Instant::now();
        self.receiver = Some(start_discovery(self.root.clone()));
    }

    pub(in crate::interactive) fn loading_elapsed(&self) -> Option<Duration> {
        (!self.loaded).then(|| self.loading_started.elapsed())
    }

    fn restore_selection(&mut self, selected_path: Option<&Path>) {
        let selected = selected_path
            .and_then(|path| {
                self.documents
                    .iter()
                    .position(|document| document.path == path)
            })
            .and_then(|document_index| {
                self.visible_indices()
                    .iter()
                    .position(|index| *index == document_index)
            });

        if let Some(selected) = selected {
            *self.selection_mut() = selected;
        } else {
            self.clamp_selection();
        }
    }
}
