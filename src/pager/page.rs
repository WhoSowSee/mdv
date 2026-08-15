use super::*;

pub(crate) fn page(
    document: PagerDocument,
    file: Option<PathBuf>,
    refresh: Option<RefreshCallback>,
    screen: PagerScreen,
) -> Result<()> {
    let editor = EditorCommand::from_env();
    let editor_enabled = !matches!(editor, Ok(None)) && file.is_some();
    let document = Arc::new(RwLock::new(document));
    let mut pending_message = None;

    loop {
        let editor_requested = Arc::new(AtomicBool::new(false));
        let pager = Pager::new();
        let (output, title, status_bar_transparent) = {
            let document = document
                .read()
                .map_err(|_| anyhow!("Pager document lock poisoned"))?;
            (
                document.output.clone(),
                document.title.clone(),
                document.status_bar_transparent(),
            )
        };
        let help_panel =
            build_help_panel(editor_enabled, refresh.is_some(), status_bar_transparent)?;
        let footer = PagerFooter::new(title.as_deref(), file.as_deref(), status_bar_transparent);
        pager.set_text(output)?;
        pager.set_prompt_renderer(move |context| footer.render(context))?;
        pager.set_search_prompt("Find: ")?;
        pager.remove_hook(Hook::PostPagerExit, 1)?;
        pager.set_input_classifier(Box::new(PagerInputClassifier {
            default: HashedEventRegister::default(),
            editor_requested: editor_requested.clone(),
            editor_enabled,
            help_panel: help_panel.clone(),
            pager: pager.clone(),
            document: document.clone(),
            refresh: refresh.clone(),
            reload_in_progress: Arc::new(AtomicBool::new(false)),
        }))?;
        if let Some(message) = pending_message.take() {
            pager.send_message(message)?;
        }

        let watcher = match (&file, &refresh) {
            (Some(path), Some(refresh)) => Some(ActiveWatcher::start(
                path,
                pager.clone(),
                refresh.clone(),
                document.clone(),
            )?),
            _ => None,
        };

        match screen {
            PagerScreen::Alternate => minus::dynamic_paging(pager)?,
            PagerScreen::InPlace => minus::dynamic_paging_in_place(pager)?,
        }
        drop(watcher);

        if !editor_requested.load(Ordering::SeqCst) {
            return Ok(());
        }

        let Some(file) = &file else {
            return Ok(());
        };
        let editor_opened = match &editor {
            Ok(Some(editor)) => match editor.open(file) {
                Ok(()) => true,
                Err(error) => {
                    pending_message = Some(single_line_message(&format!(
                        "Failed to open editor: {error}"
                    )));
                    false
                }
            },
            Err(error) => {
                pending_message = Some(single_line_message(&format!(
                    "Failed to open editor: {error}"
                )));
                false
            }
            Ok(None) => return Ok(()),
        };

        if editor_opened && let Some(refresh) = &refresh {
            match refresh() {
                Ok(refreshed) => replace_document(&document, refreshed)?,
                Err(error) => {
                    pending_message = Some(single_line_message(&format!(
                        "Failed to refresh file: {error:#}"
                    )));
                }
            }
        }
    }
}
