use super::*;

impl PagerDocument {
    pub(in crate::pager) fn shared_layout_snapshot(
        document: &RwLock<Self>,
        width: usize,
    ) -> Result<(String, Option<LineNavigation>)> {
        loop {
            let (width, identity, mode, reflow, cached) = {
                let current = document
                    .read()
                    .map_err(|_| anyhow!("Pager document lock poisoned"))?;
                let width = current.effective_width(width);
                let cached = if current.layout_width == Some(width) || current.reflow.is_none() {
                    Some(current.content.clone())
                } else {
                    current
                        .cached_layouts
                        .iter()
                        .find(|(cached, _)| *cached == width)
                        .map(|(_, content)| content.clone())
                };
                (
                    width,
                    current.identity.clone(),
                    current.line_number_mode(),
                    current.reflow.clone(),
                    cached,
                )
            };
            let prepared = (|| {
                let mut content = match cached {
                    Some(content) => content,
                    None => reflow.context("Pager layout requires a render callback")?(width)?
                        .into_content(),
                };
                if let (Some(mode), PagerContent::LineNumbers(views)) = (mode, &mut content) {
                    views.mode = mode;
                }
                let snapshot = content.snapshot()?;
                Ok::<_, anyhow::Error>((content, snapshot))
            })();
            let mut current = document
                .write()
                .map_err(|_| anyhow!("Pager document lock poisoned"))?;
            if !Arc::ptr_eq(&identity, &current.identity) || current.line_number_mode() != mode {
                continue;
            }
            let (content, snapshot) = prepared?;
            if current.layout_width != Some(width) && current.reflow.is_some() {
                let previous = std::mem::replace(&mut current.content, content);
                if let Some(previous_width) = current.layout_width {
                    current.cache_layout(previous_width, previous);
                }
                current
                    .cached_layouts
                    .retain(|(cached, _)| *cached != width);
                current.layout_width = Some(width);
            }
            current.schedule_warmup();
            return Ok(snapshot);
        }
    }

    pub(in crate::pager) fn shared_cycle_line_number_mode(document: &RwLock<Self>) -> Result<bool> {
        loop {
            let (identity, mut views) = {
                let current = document
                    .read()
                    .map_err(|_| anyhow!("Pager document lock poisoned"))?;
                let PagerContent::LineNumbers(views) = &current.content else {
                    return Ok(false);
                };
                (current.identity.clone(), views.clone())
            };
            let previous_mode = views.mode;
            let prepared = views.cycle();
            let mut current = document
                .write()
                .map_err(|_| anyhow!("Pager document lock poisoned"))?;
            if !Arc::ptr_eq(&identity, &current.identity) {
                continue;
            }
            let PagerContent::LineNumbers(active) = &mut current.content else {
                continue;
            };
            if !active.shares_cache_with(&views) || active.mode != previous_mode {
                continue;
            }
            prepared?;
            active.mode = views.mode;
            return Ok(true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cold_layout_releases_the_document_lock_and_discards_stale_content() {
        let timeout = Duration::from_secs(5);
        let (ready_tx, ready_rx) = mpsc::channel();
        let (resume_tx, resume_rx) = mpsc::channel();
        let resume_rx = std::sync::Mutex::new(resume_rx);
        let reflow: super::super::super::rendering::Reflow = Arc::new(move |_| {
            ready_tx.send(()).unwrap();
            resume_rx.lock().unwrap().recv_timeout(timeout).unwrap();
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
        let target = document.clone();
        let worker = thread::spawn(move || PagerDocument::shared_layout_snapshot(&target, 64));
        ready_rx.recv_timeout(timeout).unwrap();
        *document
            .try_write()
            .expect("cold layout must release document lock") =
            PagerDocument::new("new".into(), OutputStyle::Disabled);
        resume_tx.send(()).unwrap();
        assert_eq!(worker.join().unwrap().unwrap().0, "new");
        assert_eq!(document.read().unwrap().content.output().unwrap(), "new");
    }

    #[test]
    fn cold_numbering_change_releases_the_document_lock() {
        let timeout = Duration::from_secs(5);
        let (ready_tx, ready_rx) = mpsc::channel();
        let (resume_tx, resume_rx) = mpsc::channel();
        let resume_rx = std::sync::Mutex::new(resume_rx);
        let views = PagerLineNumberViews::new(
            PagerLineNumberMode::Off,
            Arc::new(move |mode| {
                if mode == PagerLineNumberMode::Rendered {
                    ready_tx.send(()).unwrap();
                    resume_rx.lock().unwrap().recv_timeout(timeout).unwrap();
                }
                Ok(PagerDisplay::new(format!("{mode:?}"), vec![Some(1)]))
            }),
        );
        let document = Arc::new(RwLock::new(PagerDocument::from_content(
            PagerContent::LineNumbers(views),
            OutputStyle::Disabled,
        )));
        let target = document.clone();
        let worker = thread::spawn(move || PagerDocument::shared_cycle_line_number_mode(&target));
        ready_rx.recv_timeout(timeout).unwrap();
        assert!(
            document.try_write().is_ok(),
            "numbering change must release document lock"
        );
        resume_tx.send(()).unwrap();
        assert!(worker.join().unwrap().unwrap());
        assert_eq!(
            document.read().unwrap().line_number_mode(),
            Some(PagerLineNumberMode::Rendered)
        );
    }
}
