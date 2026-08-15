use super::*;

/// Extract language from code block
pub fn extract_code_language(kind: &CodeBlockKind) -> Option<String> {
    match kind {
        CodeBlockKind::Fenced(lang) => {
            let lang = lang.trim();
            if lang.is_empty() {
                None
            } else {
                // Handle language-specific prefixes
                let lang = if let Some(stripped) = lang.strip_prefix("language-") {
                    stripped
                } else {
                    lang
                };
                Some(lang.to_string())
            }
        }
        CodeBlockKind::Indented => None,
    }
}

/// Check if content looks like source code based on file extension or content
pub fn detect_source_code(content: &str, filename: Option<&str>) -> Option<String> {
    // Check file extension first
    if let Some(filename) = filename
        && let Some(ext) = std::path::Path::new(filename).extension()
        && let Some(ext_str) = ext.to_str()
    {
        return match ext_str.to_lowercase().as_str() {
            "rs" => Some("rust".to_string()),
            "py" => Some("python".to_string()),
            "js" => Some("javascript".to_string()),
            "ts" => Some("typescript".to_string()),
            "go" => Some("go".to_string()),
            "c" => Some("c".to_string()),
            "cpp" | "cc" | "cxx" => Some("cpp".to_string()),
            "java" => Some("java".to_string()),
            "rb" => Some("ruby".to_string()),
            "php" => Some("php".to_string()),
            "sh" | "bash" => Some("bash".to_string()),
            "sql" => Some("sql".to_string()),
            "json" => Some("json".to_string()),
            "yaml" | "yml" => Some("yaml".to_string()),
            "toml" => Some("toml".to_string()),
            "xml" => Some("xml".to_string()),
            "html" => Some("html".to_string()),
            "css" => Some("css".to_string()),
            _ => None,
        };
    }

    // Try to detect from content patterns
    let lines: Vec<&str> = content.lines().take(10).collect();

    // Look for shebangs
    if let Some(first_line) = lines.first()
        && first_line.starts_with("#!")
    {
        if first_line.contains("python") {
            return Some("python".to_string());
        } else if first_line.contains("bash") || first_line.contains("sh") {
            return Some("bash".to_string());
        } else if first_line.contains("node") {
            return Some("javascript".to_string());
        }
    }

    // Look for common patterns
    for line in &lines {
        let line = line.trim();

        // Python patterns
        if line.starts_with("def ")
            || line.starts_with("class ")
            || line.starts_with("import ")
            || line.starts_with("from ")
        {
            return Some("python".to_string());
        }

        // Rust patterns
        if line.starts_with("fn ")
            || line.starts_with("struct ")
            || line.starts_with("impl ")
            || line.starts_with("use ")
        {
            return Some("rust".to_string());
        }

        // JavaScript/TypeScript patterns
        if line.starts_with("function ")
            || line.starts_with("const ")
            || line.starts_with("let ")
            || line.starts_with("var ")
        {
            return Some("javascript".to_string());
        }

        // Go patterns
        if line.starts_with("package ") || line.starts_with("func ") {
            return Some("go".to_string());
        }
    }

    None
}
