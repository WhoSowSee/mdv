use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn resolve_syntax<'s>(
        &'s self,
        language_hint: Option<&str>,
        code: &str,
    ) -> &'s SyntaxReference {
        let mut seen: Vec<String> = Vec::new();

        if let Some(lang) = language_hint {
            let candidates = Self::split_language_hint(lang);
            if let Some(hit) = self.try_lookup(&candidates, &mut seen) {
                return hit;
            }

            if !self.config.code_guessing {
                return self.syntax_set.find_syntax_plain_text();
            }
        }

        if !self.config.code_guessing {
            return self.syntax_set.find_syntax_plain_text();
        }

        if let Some(first_line_match) = self.syntax_set.find_syntax_by_first_line(code) {
            return first_line_match;
        }

        if let Some(guessed) = detect_source_code(code, None)
            && let Some(hit) = self.try_lookup(&[guessed], &mut seen)
        {
            return hit;
        }

        self.syntax_set.find_syntax_plain_text()
    }

    pub(super) fn resolve_language_label(raw_hint: &str, syntax: &SyntaxReference) -> String {
        let syntax_name = syntax.name.trim();
        let syntax_name_lower = syntax_name.to_ascii_lowercase();

        if let Some(label) = Self::custom_language_label(raw_hint, &syntax_name_lower) {
            return label;
        }

        if syntax_name_lower.contains("plain text") {
            return Self::fallback_language_label(raw_hint).unwrap_or_else(|| "Text".to_string());
        }

        syntax_name.to_string()
    }

    pub(super) fn find_custom_code_block(
        &self,
        hint: &str,
    ) -> Option<&crate::custom_code_block::CustomCodeBlock> {
        let key = hint.to_ascii_lowercase();
        self.config.custom_code_blocks.get(&key).or_else(|| {
            self.config
                .custom_code_blocks
                .values()
                .find(|b| b.aliases.contains(&key))
        })
    }

    pub(super) fn code_block_icon_for_hint(&self, hint: &str, fallback_label: &str) -> String {
        if let Some(block) = self.find_custom_code_block(hint)
            && let Some(icon) = block.icon.as_ref()
        {
            return icon.clone();
        }
        let built_in = Self::default_code_block_icon_for_label(hint).to_string();
        if built_in != Self::DEFAULT_CODE_BLOCK_ICON {
            return built_in;
        }
        let built_in = Self::default_code_block_icon_for_label(fallback_label).to_string();
        if built_in != Self::DEFAULT_CODE_BLOCK_ICON {
            return built_in;
        }
        self.config
            .custom_code_default_icon
            .clone()
            .unwrap_or(built_in)
    }

    pub(in crate::renderer::event) fn format_code_block_label(
        &self,
        hint: &str,
        base_label: &str,
    ) -> Option<String> {
        match (
            self.config.code_block_style.show_name,
            self.config.code_block_style.show_icon,
        ) {
            (false, false) => None,
            (true, false) => Some(base_label.to_string()),
            (false, true) => {
                let icon = self.code_block_icon_for_hint(hint, base_label);
                Some(self.clamp_code_block_icon(&icon, base_label, false))
            }
            (true, true) => {
                let icon = self.code_block_icon_for_hint(hint, base_label);
                let clamped = self.clamp_code_block_icon(&icon, base_label, true);
                Some(format!("{} {}", clamped, base_label))
            }
        }
    }

    pub(super) fn clamp_code_block_icon(
        &self,
        icon: &str,
        base_label: &str,
        include_name: bool,
    ) -> String {
        let terminal_width = self.config.get_content_width();
        let context_width = self.compute_code_block_context_width();
        let layout_overhead = match self.config.code_block_style.style {
            CodeBlockStyle::Basic => BASIC_CODE_BLOCK_INDENT,
            CodeBlockStyle::Simple => 2,
            CodeBlockStyle::Pretty => 6,
        };
        let max_label_width = terminal_width.saturating_sub(context_width + layout_overhead);
        let min_icon_width = display_width(icon.trim_end());

        let separator_width = usize::from(include_name);
        let label_width = if include_name {
            display_width(base_label)
        } else {
            0
        };
        let max_icon_width = max_label_width
            .saturating_sub(separator_width + label_width)
            .max(min_icon_width);

        let mut result = icon.to_string();
        while display_width(&result) > max_icon_width && result.ends_with(' ') {
            result.pop();
        }
        result
    }

    const DEFAULT_CODE_BLOCK_ICON: &'static str = " ";

    pub(super) fn default_code_block_icon_for_label(label: &str) -> &'static str {
        match label.to_ascii_lowercase().as_str() {
            "rust" | "rs" => "",
            "python" | "py" => "",
            "javascript" | "js" | "jsx" => "",
            "typescript" | "ts" | "tsx" => "󰛦",
            "go" | "golang" => "",
            "java" => "",
            "c" => "",
            "c++" | "cpp" | "cxx" => "",
            "c#" | "csharp" | "cs" => "",
            "ruby" => "",
            "php" => "",
            "html" => "",
            "css" => "",
            "scss" | "sass" => "",
            "markdown" | "md" | "mdx" => " ",
            "json" => "",
            "yaml" | "yml" => "",
            "toml" => "",
            "sql" => "",
            "shell" | "bash" | "sh" | "zsh" | "fish" => "",
            "powershell" | "ps1" | "ps" | "pwsh" => "",
            "cmd" | "bat" => "",
            "lua" => "",
            "vim" | "vimscript" => "",
            "docker" | "dockerfile" => "",
            "nix" => "",
            "haskell" => "",
            "kotlin" => "󱈙",
            "swift" => "",
            "dart" => "",
            "scala" => "",
            "elixir" => "",
            "erlang" => "",
            "clojure" => "",
            "f#" | "fsharp" | "fs" => "",
            "perl" => "",
            "objective-c" | "objc" => "",
            "text" | "plain" | "plaintext" => "󰦪",
            "diff" => "",
            "makefile" => "",
            "cmake" => "",
            "groovy" => "",
            "r" => "",
            "asm" => "",
            "vue" => "",
            "svelte" => "",
            "tailwind" | "tw" => "󱏿",
            "zig" => "",
            "ocaml" => "",
            "nim" => "",
            "csv" => "",
            "xml" => "󰗀",
            "ini" | "conf" | "config" => "",
            "log" => "󰌱",
            "graphql" | "gql" => "",
            "redis" => "",
            "julia" | "jl" => "",
            "d" | "dlang" => "",
            "crystal" | "cr" => "",
            "elm" => "",
            "haxe" | "hx" => "",
            "fortran" | "f90" => "",
            "cobol" => "",
            "ada" => "",
            "pascal" | "pas" | "delphi" => "",
            "racket" | "scheme" | "lisp" | "rkt" => "",
            "reason" | "reasonml" | "rescript" => "",
            "solidity" | "sol" => "",
            "v" | "vlang" => "",
            "astro" => "",
            "nextjs" | "next" => "",
            "nuxtjs" | "nuxt" => "󱄆",
            "angular" => "",
            "react" => "󰜈",
            "jinja" | "django" => "",
            "pug" => "",
            "jade" => "",
            "less" => "",
            "stylus" | "styl" => "",
            "postcss" => "",
            "protobuf" | "proto" => "",
            "terraform" | "hcl" | "tf" => "󱁢",
            "pulumi" => "",
            "ansible" => "󱂚",
            "nginx" => "",
            "apache" => "",
            "latex" | "tex" => "",
            "typst" => "",
            "mermaid" => "󱁉",
            "plantuml" => "",
            "postgresql" | "postgres" => "",
            "mysql" => "",
            "mongodb" | "mongo" => "",
            "sqlite" => "",
            "cassandra" | "cql" => "󰆼",
            "cypher" | "neo4j" => "",
            "llvm" | "llvm-ir" => "",
            "opencl" | "cl" => "",
            "tcl" => "󱜧",
            "awk" => "󱎸",
            "sed" => "󰛔",
            "gherkin" | "cucumber" => "",
            "qml" => "",
            _ => Self::DEFAULT_CODE_BLOCK_ICON,
        }
    }

    pub(super) fn fallback_language_label(raw_hint: &str) -> Option<String> {
        let tokens = Self::split_language_hint(raw_hint);
        for token in tokens {
            if token.is_empty() {
                continue;
            }

            if Self::is_plain_language(&token) {
                return Some("Text".to_string());
            }

            let label = Self::humanize_language_token(&token);
            if !label.is_empty() {
                return Some(label);
            }
        }

        None
    }

    pub(super) fn custom_language_label(raw_hint: &str, syntax_name_lower: &str) -> Option<String> {
        if let Some(label) = Self::lookup_custom_label(syntax_name_lower) {
            return Some(label.to_string());
        }

        for token in Self::split_language_hint(raw_hint) {
            if let Some(label) = Self::lookup_custom_label(&token) {
                return Some(label.to_string());
            }
        }

        None
    }

    pub(super) fn lookup_custom_label(key: &str) -> Option<&'static str> {
        let normalized = key.trim().to_ascii_lowercase();
        for (candidate, label) in CUSTOM_LANGUAGE_LABELS {
            if candidate.eq_ignore_ascii_case(&normalized) {
                return Some(*label);
            }
        }
        None
    }

    pub(super) fn humanize_language_token(token: &str) -> String {
        if token.is_empty() {
            return String::new();
        }

        if token.contains(['-', '_', '/', '.']) {
            let parts: Vec<String> = token
                .split(['-', '_', '/', '.'])
                .filter(|part| !part.is_empty())
                .map(Self::humanize_language_token)
                .filter(|part| !part.is_empty())
                .collect();
            if parts.is_empty() {
                return String::new();
            }
            return parts.join(" ");
        }

        if token.len() <= 3 && token.chars().all(|c| c.is_ascii_alphabetic()) {
            return token.to_ascii_uppercase();
        }

        let mut chars = token.chars();
        if let Some(first) = chars.next() {
            let mut result = String::new();
            result.extend(first.to_uppercase());
            result.push_str(chars.as_str());
            return result;
        }

        String::new()
    }
}
