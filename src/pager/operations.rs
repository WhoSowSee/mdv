use super::*;

pub(super) fn apply_refreshed_document(
    pager: &Pager,
    document: &RwLock<PagerDocument>,
    refreshed: PagerDocument,
) -> Result<()> {
    let mut current = document
        .write()
        .map_err(|_| anyhow!("Pager document lock poisoned"))?;
    replace_preserving_line_number_mode(&mut current, refreshed);
    let (output, line_navigation) = current.display_snapshot();
    pager.set_mapped_text(output, line_navigation)?;
    Ok(())
}

pub(super) fn replace_document(
    document: &RwLock<PagerDocument>,
    refreshed: PagerDocument,
) -> Result<()> {
    let mut current = document
        .write()
        .map_err(|_| anyhow!("Pager document lock poisoned"))?;
    replace_preserving_line_number_mode(&mut current, refreshed);
    Ok(())
}

fn replace_preserving_line_number_mode(current: &mut PagerDocument, mut refreshed: PagerDocument) {
    refreshed.preserve_line_number_mode_from(current);
    *current = refreshed;
}

pub(super) fn cycle_line_number_mode(
    pager: &Pager,
    document: &RwLock<PagerDocument>,
) -> Result<bool> {
    let mut document = document
        .write()
        .map_err(|_| anyhow!("Pager document lock poisoned"))?;
    let Some((output, line_navigation)) = document.cycle_line_number_mode() else {
        return Ok(false);
    };
    pager.set_mapped_text(output, line_navigation)?;
    Ok(true)
}

pub(super) fn copy_document_contents(
    document: &RwLock<PagerDocument>,
    selected_text: Option<String>,
) -> Result<()> {
    let text = clipboard_text(document, selected_text)?;
    let mut clipboard = arboard::Clipboard::new().context("Failed to access system clipboard")?;
    clipboard
        .set_text(text)
        .context("Failed to write system clipboard")
}

pub(super) fn clipboard_text(
    document: &RwLock<PagerDocument>,
    selected_text: Option<String>,
) -> Result<String> {
    match selected_text {
        Some(text) => Ok(text),
        None => Ok(document
            .read()
            .map_err(|_| anyhow!("Pager document lock poisoned"))?
            .source
            .clone()),
    }
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
