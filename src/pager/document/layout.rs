use super::*;

impl PagerDocument {
    pub(super) fn effective_width(&self, width: usize) -> usize {
        self.width_limit
            .map_or(width, |limit| limit.min(width))
            .max(1)
    }

    pub(super) fn cache_layout(&mut self, width: usize, content: PagerContent) {
        self.cached_layouts.retain(|(cached, _)| *cached != width);
        self.cached_layouts.push_back((width, content));
        while self.cached_layouts.len() > 2 {
            self.cached_layouts.pop_front();
        }
    }

    #[cfg(test)]
    pub(in crate::pager) fn layout_snapshot(
        &mut self,
        width: usize,
    ) -> Result<(String, Option<LineNavigation>)> {
        let width = self.effective_width(width);
        if self.layout_width != Some(width)
            && let Some(reflow) = &self.reflow
        {
            let mut content = if let Some(index) = self
                .cached_layouts
                .iter()
                .position(|(cached, _)| *cached == width)
            {
                self.cached_layouts.remove(index).unwrap().1
            } else {
                reflow(width)?.into_content()
            };
            if let (Some(mode), PagerContent::LineNumbers(views)) =
                (self.line_number_mode(), &mut content)
            {
                views.mode = mode;
            }
            let previous = std::mem::replace(&mut self.content, content);
            if let Some(previous_width) = self.layout_width {
                self.cache_layout(previous_width, previous);
            }
            self.layout_width = Some(width);
        }
        self.display_snapshot()
    }

    pub(in crate::pager) fn prepare_sidebar(
        document: Arc<RwLock<Self>>,
        columns: usize,
    ) -> Option<JoinHandle<()>> {
        if columns < 72 {
            return None;
        }
        let current = document.read().ok()?;
        let width = current.effective_width(columns - 36);
        if current.layout_width == Some(width)
            || current
                .cached_layouts
                .iter()
                .any(|(cached, _)| *cached == width)
        {
            return None;
        }
        let reflow = current.reflow.clone()?;
        let mode = current.line_number_mode();
        drop(current);
        Some(thread::spawn(move || {
            let Ok(rendered) = reflow(width) else {
                return;
            };
            let mut content = rendered.into_content();
            if let (Some(mode), PagerContent::LineNumbers(views)) = (mode, &mut content) {
                views.mode = mode;
            }
            if content.output().is_err() {
                return;
            }
            let Ok(mut current) = document.write() else {
                return;
            };
            if current.layout_width != Some(width)
                && current
                    .reflow
                    .as_ref()
                    .is_some_and(|active| Arc::ptr_eq(active, &reflow))
            {
                current.cache_layout(width, content);
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn background_layout_does_not_replace_refreshed_document() {
        let (ready_tx, ready_rx) = mpsc::channel();
        let (continue_tx, continue_rx) = mpsc::channel();
        let continue_rx = std::sync::Mutex::new(continue_rx);
        let reflow: super::super::super::rendering::Reflow = Arc::new(move |_| {
            ready_tx.send(()).unwrap();
            continue_rx.lock().unwrap().recv().unwrap();
            Ok(super::super::super::rendering::RenderedOutput::new(
                "old narrow".into(),
                crate::theme::PagerTheme::default(),
                OutputStyle::Disabled,
            ))
        });
        let document = Arc::new(RwLock::new(
            PagerDocument::new("old".into(), OutputStyle::Disabled).with_reflow(
                Some(reflow),
                Some(100),
                None,
            ),
        ));
        let worker = PagerDocument::prepare_sidebar(document.clone(), 100).unwrap();
        ready_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        *document.write().unwrap() = PagerDocument::new("new".into(), OutputStyle::Disabled);
        continue_tx.send(()).unwrap();
        worker.join().unwrap();
        let current = document.read().unwrap();
        assert!(current.cached_layouts.is_empty());
        assert_eq!(current.content.output().unwrap(), "new");
    }

    #[test]
    fn sidebar_layouts_preserve_frames_numbering_and_cache() {
        let source = "# Title\n\n## First\n\nBody.\n\n### Child\n\n## Second\n\n> [!NOTE]\n> A long paragraph with several words that must wrap inside the callout frame when the contents panel reduces the document width.\n>\n> - Another long line inside the same callout that must retain both vertical borders.\n";
        let mut config: crate::config::Config =
            serde_yaml::from_str("cols: 100\ncallout_style: pretty\n").unwrap();
        config.cols_from_cli = true;
        let mut document = crate::document::render_document(
            source,
            &config,
            OutputStyle::Disabled,
            crate::document::RenderOptions {
                prepare_pager_views: true,
                ..Default::default()
            },
        )
        .unwrap()
        .into_pager_document();
        let count = Arc::new(AtomicUsize::new(0));
        let renders = count.clone();
        let reflow = document.reflow.take().unwrap();
        document.reflow = Some(Arc::new(move |width| {
            renders.fetch_add(1, Ordering::SeqCst);
            reflow(width)
        }));
        let full = document.layout_snapshot(100).unwrap().0;
        document.layout_snapshot(120).unwrap();
        assert_eq!(count.load(Ordering::SeqCst), 0);
        let narrow = document.layout_snapshot(64).unwrap().0;
        assert_ne!(full, narrow);
        for (output, width) in [(&full, 100), (&narrow, 64)] {
            let rows = output
                .lines()
                .filter(|line| line.contains('╭') || line.contains('│') || line.contains('╰'))
                .collect::<Vec<_>>();
            assert!(rows.len() >= 3);
            let frame_width = crate::utils::display_width(rows[0]);
            assert!(frame_width <= width);
            for line in rows {
                assert_eq!(crate::utils::display_width(line), frame_width, "{output}");
                if line.contains('│') {
                    assert!(
                        line.trim().starts_with('│') && line.trim().ends_with('│'),
                        "{line}"
                    );
                }
            }
        }
        assert_eq!(document.layout_snapshot(100).unwrap().0, full);
        for _ in 0..3 {
            assert!(document.cycle_line_number_mode().unwrap());
            let mode = document.line_number_mode();
            let PagerContent::LineNumbers(views) = &document.content else {
                panic!("pager views missing");
            };
            let toc = views.visible_toc().unwrap();
            let entries = toc
                .iter()
                .map(|e| (e.title.as_str(), e.source_line))
                .collect::<Vec<_>>();
            assert_eq!(entries, [("First", 3), ("Child", 7), ("Second", 9)]);
            assert!(toc.iter().all(|e| {
                views
                    .current()
                    .unwrap()
                    .source_lines
                    .contains(&Some(e.source_line))
            }));
            assert!(document.display_snapshot().unwrap().1.is_some());
            for width in [64, 100] {
                document.layout_snapshot(width).unwrap();
                assert_eq!(document.line_number_mode(), mode);
            }
        }
        assert_eq!(count.load(Ordering::SeqCst), 1);
        for width in 65..68 {
            document.layout_snapshot(width).unwrap();
        }
        assert_eq!(document.cached_layouts.len(), 2);
    }
}
