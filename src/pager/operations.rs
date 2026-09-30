use super::*;

pub(super) fn apply_refreshed_document(
    pager: &Pager,
    document: &RwLock<PagerDocument>,
    refreshed: PagerDocument,
) -> Result<()> {
    let current = replace_preserving_line_number_mode(document, refreshed)?;
    update_display(pager, &current)
}

pub(super) fn replace_document(
    document: &RwLock<PagerDocument>,
    refreshed: PagerDocument,
) -> Result<()> {
    drop(replace_preserving_line_number_mode(document, refreshed)?);
    Ok(())
}

fn replace_preserving_line_number_mode(
    document: &RwLock<PagerDocument>,
    mut refreshed: PagerDocument,
) -> Result<std::sync::RwLockWriteGuard<'_, PagerDocument>> {
    loop {
        let mode = {
            let current = document
                .read()
                .map_err(|_| anyhow!("Pager document lock poisoned"))?;
            refreshed.preserve_line_number_mode_from(&current);
            current.line_number_mode()
        };
        refreshed.prepare_current_view()?;
        let mut current = document
            .write()
            .map_err(|_| anyhow!("Pager document lock poisoned"))?;
        if current.line_number_mode() != mode {
            continue;
        }
        refreshed.inherit_warmup_from(&current);
        *current = refreshed;
        return Ok(current);
    }
}

pub(super) fn cycle_line_number_mode(
    pager: &Pager,
    document: &RwLock<PagerDocument>,
) -> Result<bool> {
    if !PagerDocument::shared_cycle_line_number_mode(document)? {
        return Ok(false);
    }
    let document = document
        .read()
        .map_err(|_| anyhow!("Pager document lock poisoned"))?;
    update_display(pager, &document)?;
    Ok(true)
}

fn update_display(pager: &Pager, document: &PagerDocument) -> Result<()> {
    if document.can_reflow() {
        pager.refresh_layout()?;
    } else {
        let (output, navigation) = document.display_snapshot()?;
        pager.set_mapped_text(output, navigation)?;
    }
    Ok(())
}

pub(super) fn copy_document_contents(text: String) -> Result<()> {
    let mut clipboard = arboard::Clipboard::new().context("Failed to access system clipboard")?;
    clipboard
        .set_text(text)
        .context("Failed to write system clipboard")
}

pub(super) fn report_operation_result(
    pager: &Pager,
    result: Result<()>,
    success_message: &str,
    failure_message: &str,
) {
    let send_result = match result {
        Ok(()) => pager.send_message_for(success_message, STATUS_MESSAGE_TIMEOUT),
        Err(error) => pager.send_message(single_line_message(&format!(
            "{failure_message}: {error:#}"
        ))),
    };
    let _ = send_result;
}

pub(super) fn single_line_message(message: &str) -> String {
    message.split_whitespace().collect::<Vec<_>>().join(" ")
}
