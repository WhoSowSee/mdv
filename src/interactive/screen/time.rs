use super::*;

pub(super) fn relative_time(modified: SystemTime) -> Result<String> {
    document_time(modified, SystemTime::now())
}

pub(super) fn document_time(modified: SystemTime, now: SystemTime) -> Result<String> {
    let age = now.duration_since(modified).unwrap_or(Duration::ZERO);
    Ok(match age {
        age if age < Duration::from_secs(60) => "just now".to_string(),
        age if age < Duration::from_secs(120) => "1 minute ago".to_string(),
        age if age < Duration::from_secs(3_600) => {
            format!("{} minutes ago", age.as_secs() / 60)
        }
        age if age < Duration::from_secs(7_200) => "1 hour ago".to_string(),
        age if age < Duration::from_secs(86_400) => {
            format!("{} hours ago", age.as_secs() / 3_600)
        }
        age if age < Duration::from_secs(172_800) => "1 day ago".to_string(),
        age if age < Duration::from_secs(604_800) => {
            format!("{} days ago", age.as_secs() / 86_400)
        }
        _ => return format_local_timestamp(modified),
    })
}

pub(super) fn format_local_timestamp(modified: SystemTime) -> Result<String> {
    let timestamp = jiff::Zoned::try_from(modified)
        .context("document modification time is outside the supported range")?;
    Ok(timestamp.strftime("%d %b %Y %H:%M %Z").to_string())
}
