use super::*;

impl MathParser {
    pub(super) fn new(input: &str, mode: MathMode) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            mode,
        }
    }

    pub(super) fn parse_until(&mut self, stop: Option<char>) -> String {
        let mut out = String::new();
        while let Some(ch) = self.peek() {
            if stop == Some(ch) {
                self.pos += 1;
                break;
            }

            self.pos += 1;
            match ch {
                '\\' => out.push_str(&self.parse_command()),
                '^' => out.push_str(&self.parse_script(ScriptKind::Sup)),
                '_' => out.push_str(&self.parse_script(ScriptKind::Sub)),
                '{' => out.push_str(&self.parse_until(Some('}'))),
                '}' => out.push('}'),
                '&' => out.push_str(self.align_separator()),
                '~' => out.push(' '),
                '\n' | '\r' => out.push_str(self.line_break()),
                _ => out.push(ch),
            }
        }
        out
    }

    pub(super) fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    pub(super) fn parse_atom(&mut self) -> String {
        match self.peek() {
            Some('{') => {
                self.pos += 1;
                self.parse_until(Some('}'))
            }
            Some('\\') => {
                self.pos += 1;
                self.parse_command()
            }
            Some(ch) => {
                self.pos += 1;
                ch.to_string()
            }
            None => String::new(),
        }
    }

    pub(super) fn parse_script(&mut self, kind: ScriptKind) -> String {
        let atom = self.parse_atom();
        if atom.is_empty() {
            return String::new();
        }
        convert_script(&atom, kind)
    }

    pub(super) fn parse_command(&mut self) -> String {
        let name = self.read_command_name();
        if name.is_empty() {
            return "\\".to_string();
        }

        if let Some(literal) = literal_command(&name) {
            return literal.to_string();
        }

        if let Some(space) = spacing_command(&name) {
            return space.to_string();
        }

        match name.as_str() {
            "\\" => self.line_break().to_string(),
            "displaystyle" | "textstyle" | "scriptstyle" | "scriptscriptstyle" | "limits"
            | "nolimits" => String::new(),
            "frac" => {
                let numerator = self.parse_group();
                let denominator = self.parse_group();
                render_fraction(&numerator, &denominator)
            }
            "sqrt" => {
                let index = self.parse_optional_bracket();
                let radicand = self.parse_group();
                render_sqrt(index.as_deref(), &radicand)
            }
            "binom" => {
                let upper = self.parse_group();
                let lower = self.parse_group();
                render_binom(&upper, &lower)
            }
            "left" | "right" => self.parse_delimiter(),
            "begin" => {
                let env = self.parse_raw_group();
                let content = self.consume_until_end_env(&env);
                render_environment(&env, &content, self.mode)
            }
            "end" => {
                self.parse_raw_group();
                String::new()
            }
            "text" | "mathrm" | "mathbf" | "mathbb" | "mathcal" | "mathsf" | "mathit"
            | "operatorname" => {
                let content = self.parse_group();
                render_text_command(&name, &content)
            }
            _ => command_symbol(&name)
                .map(|symbol| symbol.to_string())
                .unwrap_or_else(|| format!("\\{}", name)),
        }
    }

    pub(super) fn read_command_name(&mut self) -> String {
        let mut name = String::new();
        match self.peek() {
            Some(ch) if ch.is_ascii_alphabetic() => {
                while let Some(next) = self.peek() {
                    if next.is_ascii_alphabetic() {
                        name.push(next);
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
            }
            Some(ch) => {
                name.push(ch);
                self.pos += 1;
            }
            None => {}
        }
        name
    }

    pub(super) fn parse_group(&mut self) -> String {
        if self.peek() != Some('{') {
            return String::new();
        }
        self.pos += 1;
        self.parse_until(Some('}'))
    }

    pub(super) fn parse_raw_group(&mut self) -> String {
        if self.peek() != Some('{') {
            return String::new();
        }
        self.pos += 1;
        let mut name = String::new();
        while let Some(ch) = self.peek() {
            self.pos += 1;
            if ch == '}' {
                break;
            }
            name.push(ch);
        }
        name
    }

    pub(super) fn parse_optional_bracket(&mut self) -> Option<String> {
        if self.peek() != Some('[') {
            return None;
        }
        self.pos += 1;
        let mut content = String::new();
        while let Some(ch) = self.peek() {
            self.pos += 1;
            if ch == ']' {
                break;
            }
            content.push(ch);
        }
        if content.is_empty() {
            None
        } else {
            Some(content)
        }
    }

    pub(super) fn parse_delimiter(&mut self) -> String {
        match self.peek() {
            Some('.') => {
                self.pos += 1;
                String::new()
            }
            Some('\\') => {
                self.pos += 1;
                let name = self.read_command_name();
                delimiter_symbol(&name)
                    .map(|symbol| symbol.to_string())
                    .unwrap_or_else(|| format!("\\{}", name))
            }
            Some(ch) => {
                self.pos += 1;
                ch.to_string()
            }
            None => String::new(),
        }
    }

    pub(super) fn consume_until_end_env(&mut self, env: &str) -> String {
        if env.is_empty() {
            return String::new();
        }

        let remaining: String = self.chars[self.pos..].iter().collect();
        let end_marker = format!("\\end{{{}}}", env);

        if let Some(idx) = remaining.find(&end_marker) {
            let content = remaining[..idx].to_string();
            let consumed_chars = remaining[..idx].chars().count() + end_marker.chars().count();
            self.pos = self.pos.saturating_add(consumed_chars);
            content
        } else {
            self.pos = self.chars.len();
            remaining
        }
    }

    pub(super) fn line_break(&self) -> &'static str {
        match self.mode {
            MathMode::Inline => " ",
            MathMode::Display => "\n",
        }
    }

    pub(super) fn align_separator(&self) -> &'static str {
        match self.mode {
            MathMode::Inline => " ",
            MathMode::Display => " ",
        }
    }
}
