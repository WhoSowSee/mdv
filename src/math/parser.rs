use super::ast::{MathDiagnostic, MathNode, ParsedMath};

const MAX_MATH_DEPTH: usize = 128;

pub(super) fn parse_math(input: &str) -> ParsedMath {
    let mut parser = MathParser::new(input, 0);
    let root = parser.parse_sequence(None, false);
    ParsedMath {
        root,
        diagnostics: parser.diagnostics,
    }
}

pub(super) struct MathParser<'a> {
    pub(super) input: &'a str,
    pub(super) pos: usize,
    pub(super) base_offset: usize,
    pub(super) depth: usize,
    pub(super) preserve_whitespace: bool,
    pub(super) diagnostics: Vec<MathDiagnostic>,
}

impl<'a> MathParser<'a> {
    pub(super) fn new(input: &'a str, base_offset: usize) -> Self {
        Self {
            input,
            pos: 0,
            base_offset,
            depth: 0,
            preserve_whitespace: false,
            diagnostics: Vec::new(),
        }
    }

    pub(super) fn parse_sequence(&mut self, stop: Option<char>, stop_at_right: bool) -> MathNode {
        let mut nodes = Vec::new();
        while let Some(ch) = self.peek() {
            if stop == Some(ch) {
                self.bump();
                return MathNode::sequence(nodes);
            }
            if stop_at_right && self.starts_command("right") {
                return MathNode::sequence(nodes);
            }
            if ch == '%' {
                self.skip_comment();
                continue;
            }
            if ch.is_whitespace() {
                let whitespace = self.parse_whitespace();
                push_node(&mut nodes, MathNode::Text(whitespace));
                continue;
            }

            let node = self.parse_atom_with_scripts();
            push_node(&mut nodes, node);
        }

        if stop.is_some() {
            self.diagnostic("unclosed group", true);
        }
        MathNode::sequence(nodes)
    }

    fn parse_atom_with_scripts(&mut self) -> MathNode {
        let mut base = self.parse_atom();
        let mut subscript = None;
        let mut superscript = None;

        loop {
            let saved = self.pos;
            self.skip_tex_ignored();
            if self.starts_command("limits") || self.starts_command("nolimits") {
                self.bump();
                let limits = self.read_command_name() == "limits";
                if let MathNode::Operator {
                    limits: placement, ..
                } = &mut base
                {
                    *placement = limits;
                }
                self.skip_whitespace();
                continue;
            }
            let target = match self.peek() {
                Some('_') => &mut subscript,
                Some('^') => &mut superscript,
                _ => {
                    self.pos = saved;
                    break;
                }
            };
            self.bump();
            let argument = self.parse_required_argument("script");
            if target.replace(Box::new(argument)).is_some() {
                self.diagnostic("repeated script", true);
            }
        }

        if subscript.is_some() || superscript.is_some() {
            base = MathNode::Scripts {
                base: Box::new(base),
                subscript,
                superscript,
            };
        }
        base
    }

    fn parse_atom(&mut self) -> MathNode {
        match self.peek() {
            Some('\\') => self.parse_command(),
            Some('{') => self.parse_group(),
            Some('}') => {
                self.bump();
                self.diagnostic("unexpected closing brace", true);
                MathNode::Text("}".to_string())
            }
            Some('~') => {
                self.bump();
                MathNode::Text(" ".to_string())
            }
            Some(ch @ ('^' | '_')) => {
                self.bump();
                self.diagnostic("script has no base", true);
                MathNode::Text(ch.to_string())
            }
            Some(ch) => {
                self.bump();
                MathNode::Text(ch.to_string())
            }
            None => MathNode::Text(String::new()),
        }
    }

    fn parse_group(&mut self) -> MathNode {
        if self.depth >= MAX_MATH_DEPTH {
            self.diagnostic("maximum group depth exceeded", true);
            return MathNode::Unsupported(self.consume_balanced_raw('{', '}'));
        }
        self.bump();
        self.depth += 1;
        let node = self.parse_sequence(Some('}'), false);
        self.depth = self.depth.saturating_sub(1);
        node
    }

    pub(super) fn parse_required_argument(&mut self, command: &str) -> MathNode {
        self.skip_tex_ignored();
        if self.peek().is_none() {
            self.diagnostic(&format!("missing argument for \\{command}"), true);
            return MathNode::Text(String::new());
        }
        if self.depth >= MAX_MATH_DEPTH {
            self.diagnostic("maximum argument depth exceeded", true);
            return MathNode::Unsupported(self.consume_atom_raw());
        }
        if self.peek() == Some('{') {
            self.parse_group()
        } else {
            self.depth += 1;
            let argument = self.parse_atom();
            self.depth = self.depth.saturating_sub(1);
            argument
        }
    }

    pub(super) fn parse_text_argument(&mut self) -> MathNode {
        let previous = self.preserve_whitespace;
        self.preserve_whitespace = true;
        let argument = self.parse_required_argument("text");
        self.preserve_whitespace = previous;
        argument
    }

    fn parse_optional_argument(&mut self) -> Option<MathNode> {
        let saved = self.pos;
        self.skip_tex_ignored();
        if self.peek() != Some('[') {
            self.pos = saved;
            return None;
        }
        if self.depth >= MAX_MATH_DEPTH {
            self.diagnostic("maximum optional argument depth exceeded", true);
            return Some(MathNode::Unsupported(self.consume_balanced_raw('[', ']')));
        }
        let raw = self.consume_balanced_raw('[', ']');
        let inner = raw.strip_prefix('[')?.strip_suffix(']')?;
        let inner_offset = self
            .base_offset
            .saturating_add(self.pos.saturating_sub(raw.len()))
            .saturating_add(1);
        let mut parser = MathParser {
            input: inner,
            pos: 0,
            base_offset: inner_offset,
            depth: self.depth.saturating_add(1),
            preserve_whitespace: self.preserve_whitespace,
            diagnostics: Vec::new(),
        };
        let root = parser.parse_sequence(None, false);
        self.diagnostics.extend(parser.diagnostics);
        Some(root)
    }

    pub(super) fn parse_required_raw_group_with_offset(
        &mut self,
        command: &str,
    ) -> (String, usize) {
        self.skip_tex_ignored();
        if self.peek() != Some('{') {
            self.diagnostic(&format!("missing argument for \\{command}"), true);
            return (String::new(), self.base_offset + self.pos);
        }
        let content_offset = self.base_offset + self.pos + 1;
        let raw = self.consume_balanced_raw('{', '}');
        let content = raw
            .strip_prefix('{')
            .and_then(|value| value.strip_suffix('}'))
            .unwrap_or(&raw)
            .to_string();
        (content, content_offset)
    }

    fn unknown_command(&mut self, start: usize) -> MathNode {
        if self.peek() == Some('*') {
            self.bump();
        }
        loop {
            let saved = self.pos;
            self.skip_tex_ignored();
            if !matches!(self.peek(), Some('{') | Some('[')) {
                self.pos = saved;
                break;
            }
            let (open, close) = if self.peek() == Some('{') {
                ('{', '}')
            } else {
                ('[', ']')
            };
            self.consume_balanced_raw(open, close);
        }
        let source = self.input[start..self.pos].to_string();
        self.diagnostic_at(start, &format!("unsupported command in {source}"), false);
        MathNode::Unsupported(source)
    }
}

fn push_node(nodes: &mut Vec<MathNode>, node: MathNode) {
    if let (Some(MathNode::Text(current)), MathNode::Text(next)) = (nodes.last_mut(), &node) {
        current.push_str(next);
    } else {
        nodes.push(node);
    }
}

mod commands;
mod environment;
mod input;
