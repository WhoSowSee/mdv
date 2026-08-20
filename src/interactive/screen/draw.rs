use super::*;

pub(super) fn draw_browser(stdout: &mut Stdout, app: &App) -> Result<()> {
    let browser = &app.browser;
    let no_colors = app.config.no_colors;
    if browser.show_error() {
        return draw_browser_error(stdout, browser, app.width, app.height, no_colors);
    }

    if browser.filter_state() == FilterState::Editing {
        let filter_text = truncate_plain(
            &format!("Find: {}", browser.query()),
            app.width.saturating_sub(3) as usize,
        );
        let filter = browser_filter_prompt_text(&filter_text, no_colors);
        write_line(stdout, 1, &filter)?;
    } else {
        write_line(
            stdout,
            1,
            &browser_logo_line(browser.loading_elapsed(), no_colors),
        )?;
    }

    let header = browser_header(browser, no_colors);
    write_line(stdout, 3, &format!("   {header}"))?;

    let visible = browser.visible_indices();
    let page_start = browser.page() * browser.per_page();
    let page_end = (page_start + browser.per_page()).min(visible.len());
    let available_width = app.width.saturating_sub(8) as usize;
    if page_start == page_end {
        let message = if browser.is_loaded() {
            "No files found."
        } else {
            "Looking for local files..."
        };
        write_line(
            stdout,
            5,
            &format!(
                "   {}",
                styled(message, Some(rgb(98, 98, 98)), None, false, no_colors)
            ),
        )?;
    } else {
        for (row, document_index) in visible[page_start..page_end].iter().enumerate() {
            let document = &browser.documents()[*document_index];
            let y = 5 + (row * 3) as u16;
            let selected = browser_item_selected(browser, row);
            let title_color = if selected {
                BROWSER_ACCENT
            } else {
                rgb(221, 221, 221)
            };
            let date_color = if selected {
                BROWSER_SELECTED_DATE
            } else {
                rgb(98, 98, 98)
            };
            let title = truncate_plain(&sanitize_display(&document.relative_path), available_width);
            let title = if browser.filter_state() == FilterState::Editing
                || browser.section() == BrowserSection::Filter
            {
                filtered_title(document, &title, browser.query(), title_color, no_colors)
            } else {
                styled(&title, Some(title_color), None, false, no_colors)
            };
            let date_text = relative_time(document.modified)?;
            let date = styled(&date_text, Some(date_color), None, false, no_colors);
            let prefix = item_prefix(selected, no_colors);
            write_line(stdout, y, &format!("{prefix}{title}"))?;
            write_line(stdout, y + 1, &format!("{prefix}{date}"))?;
        }
    }

    let (pagination_y, help_y) = browser_footer_rows(app.height, browser);
    if browser.page_count() > 1 {
        write_line(
            stdout,
            pagination_y,
            &format!("   {}", pagination(browser, app.width as usize, no_colors)),
        )?;
    }
    draw_browser_help(stdout, browser, help_y, app.width as usize, no_colors)?;
    if browser.filter_state() == FilterState::Editing {
        let filter_text = truncate_plain(
            &format!("Find: {}", browser.query()),
            app.width.saturating_sub(3) as usize,
        );
        queue!(
            stdout,
            Show,
            MoveTo(browser_filter_cursor_x(&filter_text, app.width), 1)
        )?;
    }
    Ok(())
}

pub(super) fn draw_browser_error(
    stdout: &mut Stdout,
    browser: &BrowserState,
    width: u16,
    height: u16,
    no_colors: bool,
) -> Result<()> {
    let title = styled(
        " ERROR ",
        Some(rgb(255, 253, 245)),
        Some(rgb(237, 86, 122)),
        false,
        no_colors,
    );
    write_line(stdout, 1, &format!("   {title}"))?;
    for (index, error) in browser
        .errors()
        .iter()
        .take(height.saturating_sub(6) as usize)
        .enumerate()
    {
        let error = truncate_plain(&sanitize_display(error), width.saturating_sub(6) as usize);
        write_line(stdout, 3 + index as u16, &format!("   {error}"))?;
    }
    let prompt = styled(
        "press any key to return",
        Some(rgb(92, 92, 92)),
        None,
        false,
        no_colors,
    );
    write_line(stdout, height.saturating_sub(2), &format!("   {prompt}"))?;
    Ok(())
}
