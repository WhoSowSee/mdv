use super::*;

impl MathParser<'_> {
    pub(super) fn consume_balanced_raw(&mut self, open: char, close: char) -> String {
        let start = self.pos;
        if self.peek() != Some(open) {
            return String::new();
        }
        let mut depth = 0usize;
        while let Some(character) = self.bump() {
            if character == '\\' {
                self.bump();
                continue;
            }
            if character == '%' {
                self.skip_comment();
                continue;
            }
            if character == open {
                depth += 1;
            } else if character == close {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return self.input[start..self.pos].to_string();
                }
            }
        }
        self.diagnostic("unclosed argument", true);
        self.input[start..].to_string()
    }

    pub(super) fn consume_atom_raw(&mut self) -> String {
        let start = self.pos;
        match self.peek() {
            Some('\\') => {
                self.bump();
                self.read_command_name();
            }
            Some('{') => {
                self.consume_balanced_raw('{', '}');
            }
            Some(_) => {
                self.bump();
            }
            None => {}
        }
        self.input[start..self.pos].to_string()
    }

    pub(super) fn read_command_name(&mut self) -> String {
        let mut name = String::new();
        match self.peek() {
            Some(character) if character.is_ascii_alphabetic() => {
                while let Some(next) = self.peek() {
                    if !next.is_ascii_alphabetic() {
                        break;
                    }
                    name.push(next);
                    self.bump();
                }
            }
            Some(character) => {
                name.push(character);
                self.bump();
            }
            None => {}
        }
        name
    }

    pub(super) fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    pub(super) fn bump(&mut self) -> Option<char> {
        let character = self.peek()?;
        self.pos += character.len_utf8();
        Some(character)
    }

    pub(super) fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.bump();
        }
    }

    pub(super) fn skip_tex_ignored(&mut self) {
        loop {
            let before = self.pos;
            self.skip_whitespace();
            if self.peek() == Some('%') {
                self.skip_comment();
            }
            if self.pos == before {
                break;
            }
        }
    }

    pub(super) fn parse_whitespace(&mut self) -> String {
        if !self.preserve_whitespace {
            self.skip_whitespace();
            return " ".to_string();
        }

        let mut whitespace = String::new();
        while let Some(character) = self.peek().filter(|character| character.is_whitespace()) {
            self.bump();
            match character {
                '\r' => {
                    if self.peek() == Some('\n') {
                        self.bump();
                    }
                    whitespace.push(' ');
                }
                '\n' | '\t' => whitespace.push(' '),
                other => whitespace.push(other),
            }
        }
        whitespace
    }

    pub(super) fn skip_comment(&mut self) {
        skip_comment(self.input, &mut self.pos);
    }

    pub(super) fn starts_command(&self, name: &str) -> bool {
        let Some(tail) = self.input.get(self.pos..) else {
            return false;
        };
        let Some(rest) = tail
            .strip_prefix('\\')
            .and_then(|tail| tail.strip_prefix(name))
        else {
            return false;
        };
        !rest
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_alphabetic())
    }

    pub(super) fn diagnostic(&mut self, message: &str, fatal: bool) {
        self.diagnostic_at(self.pos, message, fatal);
    }

    pub(super) fn diagnostic_at(&mut self, offset: usize, message: &str, fatal: bool) {
        self.diagnostics.push(MathDiagnostic {
            offset: self.base_offset + offset,
            message: message.to_string(),
            fatal,
        });
    }
}

pub(super) fn skip_comment(input: &str, cursor: &mut usize) {
    *cursor = input[*cursor..]
        .find('\n')
        .map_or(input.len(), |offset| *cursor + offset + 1);
}
