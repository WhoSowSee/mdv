use super::*;

pub(super) fn apply_refreshed_document(
    pager: &Pager,
    document: &RwLock<PagerDocument>,
    refreshed: PagerDocument,
) -> Result<()> {
    let output = refreshed.output.clone();
    replace_document(document, refreshed)?;
    pager.set_text(output)?;
    Ok(())
}

pub(super) fn replace_document(
    document: &RwLock<PagerDocument>,
    refreshed: PagerDocument,
) -> Result<()> {
    *document
        .write()
        .map_err(|_| anyhow!("Pager document lock poisoned"))? = refreshed;
    Ok(())
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
