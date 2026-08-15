use super::*;

pub(crate) fn parse_color_value(value: &str) -> Result<Color> {
    parse_color_spec(value)
}

pub(super) fn parse_color_spec(value: &str) -> Result<Color> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        bail!("Color cannot be an empty string.");
    }

    if trimmed.starts_with('#') {
        return parse_hex_color(trimmed);
    }

    let lower = trimmed.to_ascii_lowercase();

    if let Ok(value) = trimmed.parse::<i16>() {
        if (0..=255).contains(&value) {
            return Ok(Color::AnsiValue(value as u8));
        } else {
            bail!("ANSI value '{}' must be in the range 0..=255.", value);
        }
    }

    if let Some(inner) = lower.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
        let (r, g, b) = parse_rgb_components(inner)?;
        return Ok(Color::Rgb { r, g, b });
    }

    if trimmed.contains(',') {
        let (r, g, b) = parse_rgb_components(trimmed)?;
        return Ok(Color::Rgb { r, g, b });
    }

    if let Some(inner) = lower
        .strip_prefix("ansi(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let value = inner.trim().parse::<u8>().map_err(|_| {
            anyhow!(
                "Value '{}': expected a number in the range 0..=255 for ansi().",
                inner
            )
        })?;
        return Ok(Color::AnsiValue(value));
    }

    match lower.as_str() {
        "reset" => Ok(Color::Reset),
        name => parse_named_color(name).ok_or_else(|| anyhow!("Unknown color value '{}'.", value)),
    }
}

fn parse_hex_color(value: &str) -> Result<Color> {
    let hex = value.trim_start_matches('#');

    let (r, g, b) = match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16)
                .map_err(|_| anyhow!("Failed to parse R component from '{}'.", value))?;
            let g = u8::from_str_radix(&hex[2..4], 16)
                .map_err(|_| anyhow!("Failed to parse G component from '{}'.", value))?;
            let b = u8::from_str_radix(&hex[4..6], 16)
                .map_err(|_| anyhow!("Failed to parse B component from '{}'.", value))?;
            (r, g, b)
        }
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16)
                .map_err(|_| anyhow!("Failed to parse R component from '{}'.", value))?;
            let g = u8::from_str_radix(&hex[1..2], 16)
                .map_err(|_| anyhow!("Failed to parse G component from '{}'.", value))?;
            let b = u8::from_str_radix(&hex[2..3], 16)
                .map_err(|_| anyhow!("Failed to parse B component from '{}'.", value))?;
            (r * 17, g * 17, b * 17)
        }
        _ => bail!("Color '{}' must contain 3 or 6 hexadecimal digits.", value),
    };

    Ok(Color::Rgb { r, g, b })
}

fn parse_rgb_components(value: &str) -> Result<(u8, u8, u8)> {
    let parts: Vec<&str> = value.split(',').collect();
    if parts.len() != 3 {
        bail!(
            "Color '{}' must contain three comma-separated RGB components.",
            value
        );
    }

    let mut rgb = [0u8; 3];
    for (idx, part) in parts.iter().enumerate() {
        let component = part.trim();
        let parsed = component
            .parse::<i16>()
            .map_err(|_| anyhow!("Component '{}' must be an integer in 0..=255.", component))?;
        if !(0..=255).contains(&parsed) {
            bail!("Component '{}' is out of range 0..=255.", component);
        }
        rgb[idx] = parsed as u8;
    }

    Ok((rgb[0], rgb[1], rgb[2]))
}

fn parse_named_color(name: &str) -> Option<Color> {
    match name {
        "black" => Some(Color::Black),
        "darkred" => Some(Color::DarkRed),
        "dark_green" | "darkgreen" => Some(Color::DarkGreen),
        "darkyellow" | "dark_yellow" => Some(Color::DarkYellow),
        "darkblue" | "dark_blue" => Some(Color::DarkBlue),
        "darkmagenta" | "dark_magenta" => Some(Color::DarkMagenta),
        "darkcyan" | "dark_cyan" => Some(Color::DarkCyan),
        "grey" | "gray" => Some(Color::Grey),
        "darkgrey" | "darkgray" | "dark_grey" | "dark_gray" => Some(Color::DarkGrey),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "white" => Some(Color::White),
        _ => None,
    }
}

pub(super) fn is_none_value(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("none")
        || trimmed.eq_ignore_ascii_case("null")
}

/// Calculate overall luminosity of a theme
pub(super) fn calculate_theme_luminosity(theme: &Theme) -> f64 {
    let colors = [&theme.h1, &theme.h2, &theme.h3, &theme.h4, &theme.h5];
    let mut total_lum = 0.0;
    let mut count = 0;

    for color in colors {
        if let Some((r, g, b)) = color_to_rgb(color) {
            total_lum += calculate_luminosity(r, g, b);
            count += 1;
        }
    }

    if count > 0 {
        total_lum / count as f64
    } else {
        0.5 // Default middle luminosity
    }
}

/// Convert Color to RGB tuple if possible
fn color_to_rgb(color: &Color) -> Option<(u8, u8, u8)> {
    match color {
        Color::AnsiValue(n) => Some(ansi256_to_rgb(*n)),
        Color::Rgb { r, g, b } => Some((*r, *g, *b)),
        Color::Black => Some((0, 0, 0)),
        Color::DarkRed => Some((128, 0, 0)),
        Color::DarkGreen => Some((0, 128, 0)),
        Color::DarkYellow => Some((128, 128, 0)),
        Color::DarkBlue => Some((0, 0, 128)),
        Color::DarkMagenta => Some((128, 0, 128)),
        Color::DarkCyan => Some((0, 128, 128)),
        Color::Grey => Some((192, 192, 192)),
        Color::DarkGrey => Some((128, 128, 128)),
        Color::Red => Some((255, 0, 0)),
        Color::Green => Some((0, 255, 0)),
        Color::Yellow => Some((255, 255, 0)),
        Color::Blue => Some((0, 0, 255)),
        Color::Magenta => Some((255, 0, 255)),
        Color::Cyan => Some((0, 255, 255)),
        Color::White => Some((255, 255, 255)),
        Color::Reset => None,
    }
}
