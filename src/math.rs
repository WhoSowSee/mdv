use crate::utils::display_width;

#[derive(Debug, Clone, Copy)]
pub enum MathMode {
    Inline,
    Display,
}

pub fn render_math(input: &str, mode: MathMode) -> String {
    let mut parser = MathParser::new(input, mode);
    let rendered = parser.parse_until(None);
    normalize_output(rendered, mode)
}

pub fn is_math_language_hint(language_hint: &str) -> bool {
    let lower = language_hint.to_ascii_lowercase();
    for token in lower.split([' ', '\t', ',', ';', '|']) {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            continue;
        }
        if matches!(trimmed, "math" | "latex" | "tex" | "katex" | "mathjax") {
            return true;
        }
    }
    false
}

#[derive(Debug, Clone, Copy)]
enum ScriptKind {
    Sup,
    Sub,
}

struct MathParser {
    chars: Vec<char>,
    pos: usize,
    mode: MathMode,
}

impl MathParser {
    fn new(input: &str, mode: MathMode) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            mode,
        }
    }

    fn parse_until(&mut self, stop: Option<char>) -> String {
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

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn parse_atom(&mut self) -> String {
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

    fn parse_script(&mut self, kind: ScriptKind) -> String {
        let atom = self.parse_atom();
        if atom.is_empty() {
            return String::new();
        }
        convert_script(&atom, kind)
    }

    fn parse_command(&mut self) -> String {
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

    fn read_command_name(&mut self) -> String {
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

    fn parse_group(&mut self) -> String {
        if self.peek() != Some('{') {
            return String::new();
        }
        self.pos += 1;
        self.parse_until(Some('}'))
    }

    fn parse_raw_group(&mut self) -> String {
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

    fn parse_optional_bracket(&mut self) -> Option<String> {
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

    fn parse_delimiter(&mut self) -> String {
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

    fn consume_until_end_env(&mut self, env: &str) -> String {
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

    fn line_break(&self) -> &'static str {
        match self.mode {
            MathMode::Inline => " ",
            MathMode::Display => "\n",
        }
    }

    fn align_separator(&self) -> &'static str {
        match self.mode {
            MathMode::Inline => " ",
            MathMode::Display => " ",
        }
    }
}

fn normalize_output(mut rendered: String, mode: MathMode) -> String {
    match mode {
        MathMode::Inline => {
            let mut collapsed = String::new();
            let mut prev_space = false;
            for ch in rendered.chars() {
                if ch.is_whitespace() {
                    if !prev_space {
                        collapsed.push(' ');
                        prev_space = true;
                    }
                } else {
                    collapsed.push(ch);
                    prev_space = false;
                }
            }
            collapsed.trim().to_string()
        }
        MathMode::Display => {
            let mut lines: Vec<String> = rendered
                .lines()
                .map(|line| line.trim_end().to_string())
                .collect();
            while matches!(lines.first(), Some(line) if line.is_empty()) {
                lines.remove(0);
            }
            while matches!(lines.last(), Some(line) if line.is_empty()) {
                lines.pop();
            }
            rendered = lines.join("\n");
            rendered
        }
    }
}

fn render_text_command(command: &str, content: &str) -> String {
    if command == "mathbb"
        && let Some(symbol) = mathbb_symbol(content.trim())
    {
        return symbol.to_string();
    }
    content.to_string()
}

fn render_fraction(numerator: &str, denominator: &str) -> String {
    let num = wrap_if_needed(numerator);
    let den = wrap_if_needed(denominator);
    format!("{}⁄{}", num, den)
}

fn render_sqrt(index: Option<&str>, radicand: &str) -> String {
    let core = wrap_if_needed(radicand);
    if let Some(index) = index {
        let idx_rendered = render_math(index, MathMode::Inline);
        let superscript = convert_script(&idx_rendered, ScriptKind::Sup);
        format!("{}√{}", superscript, core)
    } else {
        format!("√{}", core)
    }
}

fn render_binom(upper: &str, lower: &str) -> String {
    format!("C({},{})", upper.trim(), lower.trim())
}

fn wrap_if_needed(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let needs_parens = trimmed
        .chars()
        .any(|ch| ch.is_whitespace() || matches!(ch, '+' | '-' | '*' | '/' | '=' | '−'));
    if needs_parens && !is_wrapped(trimmed) {
        format!("({})", trimmed)
    } else {
        trimmed.to_string()
    }
}

fn is_wrapped(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.first() == Some(&b'(') && bytes.last() == Some(&b')')
}

fn render_environment(env: &str, content: &str, mode: MathMode) -> String {
    let normalized = env.trim();
    if normalized.is_empty() {
        return render_math(content, mode);
    }

    let lower = normalized.to_ascii_lowercase();
    match lower.as_str() {
        "align" | "align*" | "aligned" | "eqnarray" | "split" => render_align_environment(content),
        "matrix" | "pmatrix" | "bmatrix" | "vmatrix" | "vmatrix*" | "cases" | "bmatrix*" => {
            render_matrix_environment(&lower, content)
        }
        _ => render_math(content, mode),
    }
}

fn render_align_environment(content: &str) -> String {
    let rows = split_rows(content);
    if rows.is_empty() {
        return String::new();
    }

    let mut rendered_rows = Vec::new();
    let mut col_widths: Vec<usize> = Vec::new();

    for row in rows {
        let cols: Vec<String> = row
            .split('&')
            .map(|col| render_math(col, MathMode::Inline))
            .collect();
        if col_widths.len() < cols.len() {
            col_widths.resize(cols.len(), 0);
        }
        for (idx, col) in cols.iter().enumerate() {
            col_widths[idx] = col_widths[idx].max(display_width(col));
        }
        rendered_rows.push(cols);
    }

    let mut lines = Vec::new();
    let col_count = col_widths.len();
    for cols in rendered_rows {
        let mut line = String::new();
        for idx in 0..col_count {
            let col = cols.get(idx).map(String::as_str).unwrap_or("");
            let padding = col_widths
                .get(idx)
                .copied()
                .unwrap_or(0)
                .saturating_sub(display_width(col));
            line.push_str(col);
            if idx + 1 < col_count {
                line.push_str(&" ".repeat(padding + 1));
            }
        }
        lines.push(line.trim_end().to_string());
    }

    lines.join("\n")
}

fn render_matrix_environment(env: &str, content: &str) -> String {
    let (left, right) = match env {
        "pmatrix" => ("(", ")"),
        "bmatrix" | "bmatrix*" => ("[", "]"),
        "vmatrix" | "vmatrix*" => ("|", "|"),
        "cases" => ("{", ""),
        _ => ("", ""),
    };

    let rows = split_rows(content);
    if rows.is_empty() {
        return String::new();
    }

    let mut rendered_rows = Vec::new();
    let mut col_widths: Vec<usize> = Vec::new();

    for row in rows {
        let cols: Vec<String> = row
            .split('&')
            .map(|col| render_math(col, MathMode::Inline))
            .collect();
        if col_widths.len() < cols.len() {
            col_widths.resize(cols.len(), 0);
        }
        for (idx, col) in cols.iter().enumerate() {
            col_widths[idx] = col_widths[idx].max(display_width(col));
        }
        rendered_rows.push(cols);
    }

    let mut lines = Vec::new();
    let col_count = col_widths.len();
    for cols in rendered_rows {
        let mut line = String::new();
        if !left.is_empty() {
            line.push_str(left);
            line.push(' ');
        }
        for idx in 0..col_count {
            let col = cols.get(idx).map(String::as_str).unwrap_or("");
            let padding = col_widths
                .get(idx)
                .copied()
                .unwrap_or(0)
                .saturating_sub(display_width(col));
            line.push_str(col);
            if idx + 1 < col_count {
                line.push_str(&" ".repeat(padding + 2));
            }
        }
        if !right.is_empty() {
            line.push(' ');
            line.push_str(right);
        }
        lines.push(line.trim_end().to_string());
    }

    lines.join("\n")
}

fn split_rows(content: &str) -> Vec<String> {
    content
        .split("\\\\")
        .map(|row| row.trim().to_string())
        .filter(|row| !row.is_empty())
        .collect()
}

fn convert_script(text: &str, kind: ScriptKind) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        if let Some(mapped) = map_script_char(ch, kind) {
            out.push(mapped);
        } else {
            let marker = match kind {
                ScriptKind::Sup => "^",
                ScriptKind::Sub => "_",
            };
            return format!("{}({})", marker, text);
        }
    }
    out
}

fn map_script_char(ch: char, kind: ScriptKind) -> Option<char> {
    match kind {
        ScriptKind::Sup => match ch {
            '0' => Some('⁰'),
            '1' => Some('¹'),
            '2' => Some('²'),
            '3' => Some('³'),
            '4' => Some('⁴'),
            '5' => Some('⁵'),
            '6' => Some('⁶'),
            '7' => Some('⁷'),
            '8' => Some('⁸'),
            '9' => Some('⁹'),
            '+' => Some('⁺'),
            '-' | '−' => Some('⁻'),
            '=' => Some('⁼'),
            '(' => Some('⁽'),
            ')' => Some('⁾'),
            'a' => Some('ᵃ'),
            'b' => Some('ᵇ'),
            'c' => Some('ᶜ'),
            'd' => Some('ᵈ'),
            'e' => Some('ᵉ'),
            'f' => Some('ᶠ'),
            'g' => Some('ᵍ'),
            'h' => Some('ʰ'),
            'i' => Some('ⁱ'),
            'j' => Some('ʲ'),
            'k' => Some('ᵏ'),
            'l' => Some('ˡ'),
            'm' => Some('ᵐ'),
            'n' => Some('ⁿ'),
            'o' => Some('ᵒ'),
            'p' => Some('ᵖ'),
            'r' => Some('ʳ'),
            's' => Some('ˢ'),
            't' => Some('ᵗ'),
            'u' => Some('ᵘ'),
            'v' => Some('ᵛ'),
            'w' => Some('ʷ'),
            'x' => Some('ˣ'),
            'y' => Some('ʸ'),
            'z' => Some('ᶻ'),
            _ => None,
        },
        ScriptKind::Sub => match ch {
            '0' => Some('₀'),
            '1' => Some('₁'),
            '2' => Some('₂'),
            '3' => Some('₃'),
            '4' => Some('₄'),
            '5' => Some('₅'),
            '6' => Some('₆'),
            '7' => Some('₇'),
            '8' => Some('₈'),
            '9' => Some('₉'),
            '+' => Some('₊'),
            '-' | '−' => Some('₋'),
            '=' => Some('₌'),
            '(' => Some('₍'),
            ')' => Some('₎'),
            'a' => Some('ₐ'),
            'e' => Some('ₑ'),
            'h' => Some('ₕ'),
            'i' => Some('ᵢ'),
            'j' => Some('ⱼ'),
            'k' => Some('ₖ'),
            'l' => Some('ₗ'),
            'm' => Some('ₘ'),
            'n' => Some('ₙ'),
            'o' => Some('ₒ'),
            'p' => Some('ₚ'),
            'r' => Some('ᵣ'),
            's' => Some('ₛ'),
            't' => Some('ₜ'),
            'u' => Some('ᵤ'),
            'v' => Some('ᵥ'),
            'x' => Some('ₓ'),
            _ => None,
        },
    }
}

fn literal_command(name: &str) -> Option<&'static str> {
    match name {
        "%" => Some("%"),
        "$" => Some("$"),
        "#" => Some("#"),
        "_" => Some("_"),
        "{" => Some("{"),
        "}" => Some("}"),
        "&" => Some("&"),
        "^" => Some("^"),
        _ => None,
    }
}

fn spacing_command(name: &str) -> Option<&'static str> {
    match name {
        "," | ";" | ":" | "!" | " " => Some(" "),
        "quad" => Some("  "),
        "qquad" => Some("    "),
        _ => None,
    }
}

fn delimiter_symbol(name: &str) -> Option<&'static str> {
    match name {
        "(" => Some("("),
        ")" => Some(")"),
        "[" => Some("["),
        "]" => Some("]"),
        "{" => Some("{"),
        "}" => Some("}"),
        "|" => Some("|"),
        "langle" => Some("⟨"),
        "rangle" => Some("⟩"),
        "lfloor" => Some("⌊"),
        "rfloor" => Some("⌋"),
        "lceil" => Some("⌈"),
        "rceil" => Some("⌉"),
        "vert" => Some("|"),
        "Vert" => Some("‖"),
        _ => None,
    }
}

fn mathbb_symbol(letter: &str) -> Option<&'static str> {
    match letter {
        "R" => Some("ℝ"),
        "Z" => Some("ℤ"),
        "Q" => Some("ℚ"),
        "C" => Some("ℂ"),
        "N" => Some("ℕ"),
        _ => None,
    }
}

fn command_symbol(name: &str) -> Option<&'static str> {
    match name {
        "alpha" => Some("α"),
        "beta" => Some("β"),
        "gamma" => Some("γ"),
        "delta" => Some("δ"),
        "epsilon" => Some("ε"),
        "varepsilon" => Some("ϵ"),
        "zeta" => Some("ζ"),
        "eta" => Some("η"),
        "theta" => Some("θ"),
        "vartheta" => Some("ϑ"),
        "iota" => Some("ι"),
        "kappa" => Some("κ"),
        "lambda" => Some("λ"),
        "mu" => Some("μ"),
        "nu" => Some("ν"),
        "xi" => Some("ξ"),
        "pi" => Some("π"),
        "varpi" => Some("ϖ"),
        "rho" => Some("ρ"),
        "varrho" => Some("ϱ"),
        "sigma" => Some("σ"),
        "varsigma" => Some("ς"),
        "tau" => Some("τ"),
        "upsilon" => Some("υ"),
        "phi" => Some("φ"),
        "varphi" => Some("ϕ"),
        "chi" => Some("χ"),
        "psi" => Some("ψ"),
        "omega" => Some("ω"),
        "Alpha" => Some("Α"),
        "Beta" => Some("Β"),
        "Gamma" => Some("Γ"),
        "Delta" => Some("Δ"),
        "Epsilon" => Some("Ε"),
        "Zeta" => Some("Ζ"),
        "Eta" => Some("Η"),
        "Theta" => Some("Θ"),
        "Iota" => Some("Ι"),
        "Kappa" => Some("Κ"),
        "Lambda" => Some("Λ"),
        "Mu" => Some("Μ"),
        "Nu" => Some("Ν"),
        "Xi" => Some("Ξ"),
        "Pi" => Some("Π"),
        "Rho" => Some("Ρ"),
        "Sigma" => Some("Σ"),
        "Tau" => Some("Τ"),
        "Upsilon" => Some("Υ"),
        "Phi" => Some("Φ"),
        "Chi" => Some("Χ"),
        "Psi" => Some("Ψ"),
        "Omega" => Some("Ω"),
        "pm" => Some("±"),
        "mp" => Some("∓"),
        "times" => Some("×"),
        "cdot" => Some("·"),
        "ast" => Some("∗"),
        "star" => Some("⋆"),
        "div" => Some("÷"),
        "neq" | "ne" => Some("≠"),
        "leq" | "le" => Some("≤"),
        "geq" | "ge" => Some("≥"),
        "approx" => Some("≈"),
        "equiv" => Some("≡"),
        "propto" => Some("∝"),
        "infty" => Some("∞"),
        "partial" => Some("∂"),
        "nabla" => Some("∇"),
        "angle" => Some("∠"),
        "sum" => Some("∑"),
        "prod" => Some("∏"),
        "int" => Some("∫"),
        "iint" => Some("∬"),
        "iiint" => Some("∭"),
        "oint" => Some("∮"),
        "cdots" => Some("⋯"),
        "ldots" => Some("…"),
        "dots" => Some("…"),
        "to" | "rightarrow" => Some("→"),
        "leftarrow" => Some("←"),
        "leftrightarrow" => Some("↔"),
        "Rightarrow" => Some("⇒"),
        "Leftarrow" => Some("⇐"),
        "Leftrightarrow" => Some("⇔"),
        "mapsto" => Some("↦"),
        "in" => Some("∈"),
        "notin" => Some("∉"),
        "subset" => Some("⊂"),
        "subseteq" => Some("⊆"),
        "supset" => Some("⊃"),
        "supseteq" => Some("⊇"),
        "cup" => Some("∪"),
        "cap" => Some("∩"),
        "setminus" => Some("∖"),
        "forall" => Some("∀"),
        "exists" => Some("∃"),
        "neg" => Some("¬"),
        "land" => Some("∧"),
        "lor" => Some("∨"),
        "sin" => Some("sin"),
        "cos" => Some("cos"),
        "tan" => Some("tan"),
        "ln" => Some("ln"),
        "log" => Some("log"),
        "exp" => Some("exp"),
        "lim" => Some("lim"),
        "det" => Some("det"),
        _ => None,
    }
}
