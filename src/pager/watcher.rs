use super::*;

pub(super) struct ActiveWatcher {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl ActiveWatcher {
    pub(super) fn start(
        path: &Path,
        pager: Pager,
        refresh: RefreshCallback,
        document: Arc<RwLock<PagerDocument>>,
    ) -> Result<Self> {
        let target = comparable_path(path)?;
        let directory = target
            .parent()
            .context("Markdown file has no parent directory")?
            .to_path_buf();
        let (event_tx, event_rx) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(event_tx)
            .context("Failed to initialize pager file watcher")?;
        watcher
            .watch(&directory, RecursiveMode::NonRecursive)
            .with_context(|| format!("Failed to watch {}", directory.display()))?;

        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = stop.clone();
        let thread = thread::spawn(move || {
            let _watcher = watcher;
            let debounce = Duration::from_millis(100);
            let poll_interval = Duration::from_millis(25);
            let mut refresh_deadline = None;

            while !thread_stop.load(Ordering::SeqCst) {
                match event_rx.recv_timeout(poll_interval) {
                    Ok(Ok(event)) if event_targets_file(&event, &target) => {
                        refresh_deadline = Some(Instant::now() + debounce);
                    }
                    Ok(Ok(_)) | Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Ok(Err(error)) => {
                        if pager
                            .send_message(single_line_message(&format!(
                                "File watcher error: {error}"
                            )))
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }

                if refresh_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                    match refresh().and_then(|refreshed| {
                        apply_refreshed_document(&pager, &document, refreshed)
                    }) {
                        Ok(()) => {}
                        Err(error) => {
                            if pager
                                .send_message(single_line_message(&format!(
                                    "Failed to refresh file: {error:#}"
                                )))
                                .is_err()
                            {
                                break;
                            }
                        }
                    }
                    refresh_deadline = None;
                }
            }
        });

        Ok(Self {
            stop,
            thread: Some(thread),
        })
    }
}

impl Drop for ActiveWatcher {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

pub(super) fn event_targets_file(event: &notify::Event, target: &Path) -> bool {
    matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_))
        && event
            .paths
            .iter()
            .filter_map(|path| comparable_path(path).ok())
            .any(|path| path == target)
}

pub(super) fn comparable_path(path: &Path) -> Result<PathBuf> {
    if let Ok(path) = path.canonicalize() {
        return Ok(path);
    }

    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };

    let Some(file_name) = absolute.file_name() else {
        return Ok(absolute);
    };
    let parent = absolute.parent().context("Path has no parent directory")?;
    let parent = parent
        .canonicalize()
        .unwrap_or_else(|_| parent.to_path_buf());
    Ok(parent.join(file_name))
}
