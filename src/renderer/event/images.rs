use super::{CowStr, EventRenderer, Result, ThemeElement, create_style};
use crate::utils::strip_ansi;

pub(super) fn media_marker(dest_url: &str) -> &'static str {
    if let Some(marker) = media_marker_from_data_uri(dest_url) {
        return marker;
    }

    let Some(extension) = extract_media_extension(dest_url) else {
        return "[MEDIA] ";
    };

    if is_video_extension(&extension) {
        "[VIDEO] "
    } else if is_audio_extension(&extension) {
        "[AUDIO] "
    } else if is_gif_extension(&extension) {
        "[GIF] "
    } else if is_svg_extension(&extension) {
        "[SVG] "
    } else if is_image_extension(&extension) {
        "[IMAGE] "
    } else {
        "[MEDIA] "
    }
}

pub(super) fn media_marker_leading_separator(buffer: &str) -> &'static str {
    if needs_media_marker_leading_separator(buffer) {
        " "
    } else {
        ""
    }
}

fn needs_media_marker_leading_separator(buffer: &str) -> bool {
    let current_line = buffer
        .rsplit_once('\n')
        .map(|(_, line)| line)
        .unwrap_or(buffer);
    let clean_line = strip_ansi(current_line);

    let has_visible_text = clean_line
        .chars()
        .any(|ch| !ch.is_whitespace() && ch != '│' && ch != '┃');
    if !has_visible_text {
        return false;
    }

    if clean_line
        .chars()
        .next_back()
        .map(char::is_whitespace)
        .unwrap_or(false)
    {
        return false;
    }

    let Some(last_visible) = clean_line.trim_end().chars().next_back() else {
        return false;
    };

    !matches!(last_visible, '(' | '[' | '{')
}

fn media_marker_from_data_uri(dest_url: &str) -> Option<&'static str> {
    let value = dest_url.trim();
    if !value
        .get(..5)
        .map(|prefix| prefix.eq_ignore_ascii_case("data:"))
        .unwrap_or(false)
    {
        return None;
    }

    let header = value[5..]
        .split_once(',')
        .map(|(header, _)| header)
        .unwrap_or("");
    let mime = header
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();

    if mime.starts_with("video/") {
        Some("[VIDEO] ")
    } else if mime.starts_with("audio/") {
        Some("[AUDIO] ")
    } else if mime == "image/gif" {
        Some("[GIF] ")
    } else if mime == "image/svg+xml" {
        Some("[SVG] ")
    } else if mime.starts_with("image/") {
        Some("[IMAGE] ")
    } else {
        Some("[MEDIA] ")
    }
}

fn is_gif_extension(extension: &str) -> bool {
    extension == "gif"
}

fn is_svg_extension(extension: &str) -> bool {
    extension == "svg" || extension == "svgz"
}

fn is_image_extension(extension: &str) -> bool {
    matches!(
        extension,
        "apng"
            | "avif"
            | "bmp"
            | "dds"
            | "dib"
            | "emf"
            | "exr"
            | "hdr"
            | "heic"
            | "heif"
            | "ico"
            | "j2c"
            | "j2k"
            | "jfif"
            | "jp2"
            | "jpe"
            | "jpeg"
            | "jpf"
            | "jpg"
            | "jpm"
            | "jpx"
            | "jxl"
            | "pbm"
            | "pgm"
            | "png"
            | "pnm"
            | "ppm"
            | "psd"
            | "raw"
            | "svg"
            | "svgz"
            | "tga"
            | "tif"
            | "tiff"
            | "wbmp"
            | "webp"
            | "wmf"
    )
}

fn is_video_extension(extension: &str) -> bool {
    matches!(
        extension,
        "3g2"
            | "3gp"
            | "asf"
            | "avi"
            | "av1"
            | "drc"
            | "f4v"
            | "flv"
            | "h264"
            | "h265"
            | "hevc"
            | "m1v"
            | "m2ts"
            | "m2v"
            | "m4v"
            | "mkv"
            | "mov"
            | "mp4"
            | "mpe"
            | "mpeg"
            | "mpg"
            | "mpv"
            | "mts"
            | "mxf"
            | "ogm"
            | "ogv"
            | "qt"
            | "rm"
            | "rmvb"
            | "ts"
            | "vob"
            | "webm"
            | "wmv"
            | "y4m"
    )
}

fn is_audio_extension(extension: &str) -> bool {
    matches!(
        extension,
        "8svx"
            | "aac"
            | "ac3"
            | "aif"
            | "aifc"
            | "aiff"
            | "alac"
            | "amr"
            | "ape"
            | "au"
            | "caf"
            | "dts"
            | "eac3"
            | "flac"
            | "m4a"
            | "m4b"
            | "m4p"
            | "mid"
            | "midi"
            | "mka"
            | "mp1"
            | "mp2"
            | "mp3"
            | "mpa"
            | "mpc"
            | "oga"
            | "ogg"
            | "opus"
            | "ra"
            | "ram"
            | "snd"
            | "spx"
            | "tak"
            | "tta"
            | "wav"
            | "weba"
            | "wma"
            | "wv"
    )
}

fn extract_media_extension(dest_url: &str) -> Option<String> {
    let path = dest_url.split(['?', '#']).next().unwrap_or(dest_url);
    let filename = path.rsplit(['/', '\\']).next().unwrap_or(path);
    let (_, extension) = filename.rsplit_once('.')?;
    let extension = extension.trim().to_ascii_lowercase();
    if extension.is_empty() {
        None
    } else {
        Some(extension)
    }
}

impl<'a> EventRenderer<'a> {
    pub(super) fn handle_image_start(&mut self, dest_url: CowStr) -> Result<()> {
        let marker = media_marker(dest_url.as_ref());

        // If we are inside a table, write the marker into the current cell
        if let Some(ref mut table) = self.table_state {
            let style = create_style(self.theme, ThemeElement::Link);
            let image_marker = style.apply(marker, self.config.no_colors);
            let separator = media_marker_leading_separator(&table.current_cell);
            table.current_cell.push_str(separator);
            table.current_cell.push_str(&image_marker);
            self.commit_pending_heading_placeholder_if_content();
            return Ok(());
        }

        self.note_paragraph_content();

        // Ensure correct indentation/prefix when an image starts a visual line.
        // Paragraph start may have added spaces, but when inside lists/quotes
        // there may be no prefix yet. If the current line contains only
        // whitespace, normalize it and insert the proper context-aware prefix.
        let line_start_idx = self.output.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let current_line = &self.output[line_start_idx..];
        if current_line.trim().is_empty() {
            // Drop any existing leading spaces on the current visual line
            // (e.g. content indent added at paragraph start) to avoid double
            // indentation, then re-apply consistent prefix/indent.
            self.output.truncate(line_start_idx);
            self.push_indent_for_line_start();
        }

        let style = create_style(self.theme, ThemeElement::Link);
        let image_marker = style.apply(marker, self.config.no_colors);
        let separator = media_marker_leading_separator(&self.output);
        self.output.push_str(separator);
        self.output.push_str(&image_marker);
        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }

    pub(super) fn handle_image_end(&mut self) -> Result<()> {
        // Image handling is completed in start
        Ok(())
    }
}
