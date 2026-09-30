use super::{PagerDisplay, PagerLineNumberMode};
use anyhow::{Result, anyhow};
use minus::LineNavigation;
use std::{
    collections::HashSet,
    sync::{Arc, OnceLock},
};

type DisplayRenderer = Arc<dyn Fn(PagerLineNumberMode) -> Result<PagerDisplay> + Send + Sync>;
type CachedDisplay = Arc<OnceLock<Result<PagerDisplay, String>>>;

#[derive(Clone)]
pub(in crate::pager) struct PagerLineNumberViews {
    pub(super) mode: PagerLineNumberMode,
    pub(super) toc: Vec<minus::TocEntry>,
    displays: [CachedDisplay; 3],
    render: DisplayRenderer,
}

impl PagerLineNumberViews {
    pub(in crate::pager) fn new(mode: PagerLineNumberMode, render: DisplayRenderer) -> Self {
        Self {
            mode,
            toc: Vec::new(),
            displays: std::array::from_fn(|_| Arc::default()),
            render,
        }
    }

    pub(super) fn current(&self) -> Result<&PagerDisplay> {
        self.view(self.mode)
    }

    pub(in crate::pager) fn view(&self, mode: PagerLineNumberMode) -> Result<&PagerDisplay> {
        let index = match mode {
            PagerLineNumberMode::Off => 0,
            PagerLineNumberMode::Rendered => 1,
            PagerLineNumberMode::Source => 2,
        };
        self.displays[index]
            .get_or_init(|| (self.render)(mode).map_err(|error| format!("{error:#}")))
            .as_ref()
            .map_err(|error| anyhow!(error.clone()))
    }

    pub(in crate::pager) fn shares_cache_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.displays[0], &other.displays[0])
    }

    pub(in crate::pager) fn with_toc(mut self, entries: Vec<minus::TocEntry>) -> Self {
        self.toc = entries;
        self
    }

    pub(super) fn cycle(&mut self) -> Result<()> {
        let next = self.mode.next();
        self.view(next)?;
        self.mode = next;
        Ok(())
    }

    pub(super) fn snapshot(&self) -> Result<(String, Option<LineNavigation>)> {
        let current = self.current()?;
        let navigation = if current.source_lines.iter().any(Option::is_some) {
            let navigation = if self.mode == PagerLineNumberMode::Source {
                LineNavigation::from_current(current.source_lines.clone())
            } else {
                let views = self.clone();
                LineNavigation::deferred(
                    current.source_lines.clone(),
                    Arc::new(move || {
                        views
                            .view(PagerLineNumberMode::Source)
                            .map(|source| (source.output.clone(), source.source_lines.clone()))
                            .map_err(|error| format!("{error:#}"))
                    }),
                )
            };
            Some(navigation.with_toc(self.visible_toc()?))
        } else {
            None
        };
        Ok((current.output.clone(), navigation))
    }

    pub(super) fn visible_toc(&self) -> Result<Vec<minus::TocEntry>> {
        let visible: HashSet<_> = self
            .current()?
            .source_lines
            .iter()
            .flatten()
            .copied()
            .collect();
        let entries = self
            .toc
            .iter()
            .filter(|entry| visible.contains(&entry.source_line))
            .cloned()
            .collect();
        Ok(crate::pager::toc::visible_levels(entries))
    }
}

#[cfg(test)]
mod tests;
