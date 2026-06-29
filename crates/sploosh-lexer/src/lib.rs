//! UTF-8 lexer for Sploosh §2 and §16.1.

use sploosh_ast::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Ident,
    Keyword(Keyword),
    IntLit,
    FloatLit,
    StringLit,
    CharLit,
    Lifetime,
    DocComment,
    At,
    Hash,
    Bang,
    Dot,
    DotDot,
    DotDotEq,
    Comma,
    Colon,
    ColonColon,
    Semi,
    Arrow,
    FatArrow,
    Pipe,
    PipePipe,
    PipeGt,
    Amp,
    AmpAmp,
    Eq,
    EqEq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Question,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Fn,
    Let,
    Const,
    Type,
    Struct,
    Enum,
    Trait,
    Impl,
    Mod,
    Use,
    Pub,
    Extern,
    If,
    Else,
    Match,
    For,
    In,
    While,
    Loop,
    Break,
    Continue,
    Return,
    SelfValue,
    SelfType,
    True,
    False,
    As,
    Actor,
    Spawn,
    Async,
    Await,
    Select,
    Move,
    Onchain,
    Offchain,
    Emit,
}

impl Keyword {
    pub fn from_reserved(text: &str) -> Option<Self> {
        Some(match text {
            "fn" => Self::Fn,
            "let" => Self::Let,
            "const" => Self::Const,
            "type" => Self::Type,
            "struct" => Self::Struct,
            "enum" => Self::Enum,
            "trait" => Self::Trait,
            "impl" => Self::Impl,
            "mod" => Self::Mod,
            "use" => Self::Use,
            "pub" => Self::Pub,
            "extern" => Self::Extern,
            "if" => Self::If,
            "else" => Self::Else,
            "match" => Self::Match,
            "for" => Self::For,
            "in" => Self::In,
            "while" => Self::While,
            "loop" => Self::Loop,
            "break" => Self::Break,
            "continue" => Self::Continue,
            "return" => Self::Return,
            "self" => Self::SelfValue,
            "Self" => Self::SelfType,
            "true" => Self::True,
            "false" => Self::False,
            "as" => Self::As,
            "actor" => Self::Actor,
            "spawn" => Self::Spawn,
            "async" => Self::Async,
            "await" => Self::Await,
            "select" => Self::Select,
            "move" => Self::Move,
            "onchain" => Self::Onchain,
            "offchain" => Self::Offchain,
            "emit" => Self::Emit,
            _ => return None,
        })
    }
}

pub fn is_contextual_keyword(text: &str) -> bool {
    matches!(
        text,
        "send" | "recv" | "storage" | "mut" | "dyn" | "ref" | "crate" | "super" | "where"
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

pub fn lex(source: &str) -> Result<Vec<Token>, Vec<LexError>> {
    let mut lexer = Lexer {
        source,
        pos: 0,
        tokens: Vec::new(),
        errors: Vec::new(),
    };
    lexer.run();
    if lexer.errors.is_empty() {
        Ok(lexer.tokens)
    } else {
        Err(lexer.errors)
    }
}

struct Lexer<'a> {
    source: &'a str,
    pos: usize,
    tokens: Vec<Token>,
    errors: Vec<LexError>,
}

impl Lexer<'_> {
    fn run(&mut self) {
        while let Some(ch) = self.peek() {
            match ch {
                c if c.is_ascii_whitespace() => {
                    self.bump();
                }
                '/' if self.peek_next() == Some('/') => self.comment(),
                // Block comments are intentionally omitted (§2.2); a lone `/` is division.
                '/' => self.single(TokenKind::Slash),
                c if is_ident_start(c) => self.ident_or_keyword(),
                '0'..='9' => self.number(),
                '"' => self.string(),
                '\'' => self.lifetime_or_char(),
                '@' => self.single(TokenKind::At),
                '#' => self.single(TokenKind::Hash),
                '!' if self.peek_next() == Some('=') => self.double(TokenKind::Ne),
                '!' => self.single(TokenKind::Bang),
                '.' if self.starts_with("..=") => self.take(3, TokenKind::DotDotEq),
                '.' if self.starts_with("..") => self.take(2, TokenKind::DotDot),
                '.' => self.single(TokenKind::Dot),
                ',' => self.single(TokenKind::Comma),
                ':' if self.peek_next() == Some(':') => self.double(TokenKind::ColonColon),
                ':' => self.single(TokenKind::Colon),
                ';' => self.single(TokenKind::Semi),
                '-' if self.peek_next() == Some('>') => self.double(TokenKind::Arrow),
                '-' => self.single(TokenKind::Minus),
                '=' if self.peek_next() == Some('>') => self.double(TokenKind::FatArrow),
                '=' if self.peek_next() == Some('=') => self.double(TokenKind::EqEq),
                '=' => self.single(TokenKind::Eq),
                '|' if self.peek_next() == Some('>') => self.double(TokenKind::PipeGt),
                '|' if self.peek_next() == Some('|') => self.double(TokenKind::PipePipe),
                '|' => self.single(TokenKind::Pipe),
                '&' if self.peek_next() == Some('&') => self.double(TokenKind::AmpAmp),
                '&' => self.single(TokenKind::Amp),
                '<' if self.peek_next() == Some('=') => self.double(TokenKind::Le),
                '<' => self.single(TokenKind::Lt),
                '>' if self.peek_next() == Some('=') => self.double(TokenKind::Ge),
                '>' => self.single(TokenKind::Gt),
                '+' => self.single(TokenKind::Plus),
                '*' => self.single(TokenKind::Star),
                '%' => self.single(TokenKind::Percent),
                '?' => self.single(TokenKind::Question),
                '(' => self.single(TokenKind::LParen),
                ')' => self.single(TokenKind::RParen),
                '{' => self.single(TokenKind::LBrace),
                '}' => self.single(TokenKind::RBrace),
                '[' => self.single(TokenKind::LBracket),
                ']' => self.single(TokenKind::RBracket),
                other => {
                    let start = self.pos;
                    self.bump();
                    self.error(format!("unexpected character `{other}`"), start, self.pos);
                }
            }
        }
    }

    fn comment(&mut self) {
        let start = self.pos;
        self.pos += 2;
        let doc = self.peek() == Some('/');
        if doc {
            self.bump();
        }
        while let Some(ch) = self.peek() {
            if ch == '\n' || ch == '\r' {
                break;
            }
            self.bump();
        }
        if doc {
            self.push(TokenKind::DocComment, start, self.pos);
        }
    }

    fn ident_or_keyword(&mut self) {
        let start = self.pos;
        self.bump();
        while matches!(self.peek(), Some(c) if is_ident_continue(c)) {
            self.bump();
        }
        let text = &self.source[start..self.pos];
        let kind = Keyword::from_reserved(text).map_or(TokenKind::Ident, TokenKind::Keyword);
        self.push(kind, start, self.pos);
    }

    fn number(&mut self) {
        let start = self.pos;
        let mut base = 10;
        if self.starts_with("0x") || self.starts_with("0o") || self.starts_with("0b") {
            base = match self.source.as_bytes()[self.pos + 1] {
                b'x' => 16,
                b'o' => 8,
                b'b' => 2,
                _ => 10,
            };
            self.pos += 2;
        }

        self.digits(base);
        let mut kind = TokenKind::IntLit;
        if base == 10 && self.peek() == Some('.') && self.peek_next() != Some('.') {
            kind = TokenKind::FloatLit;
            self.bump();
            let after_dot = self.pos;
            self.digits(10);
            if self.pos == after_dot {
                self.error("float literal needs digits after `.`", start, self.pos);
            }
        }
        if base == 10 && matches!(self.peek(), Some('e' | 'E')) {
            kind = TokenKind::FloatLit;
            self.bump();
            if matches!(self.peek(), Some('+' | '-')) {
                self.bump();
            }
            let exp_start = self.pos;
            self.digits(10);
            if self.pos == exp_start {
                self.error("float exponent needs digits", start, self.pos);
            }
        }
        if let Some(suffix) = self.numeric_suffix() {
            kind = if suffix.starts_with('f') {
                TokenKind::FloatLit
            } else {
                kind
            };
        }
        self.validate_numeric_body(start);
        self.push(kind, start, self.pos);
    }

    fn digits(&mut self, base: u8) {
        while matches!(self.peek(), Some(c) if c == '_' || valid_digit(c, base)) {
            self.bump();
        }
    }

    fn numeric_suffix(&mut self) -> Option<String> {
        let rest = &self.source[self.pos..];
        let suffixes = [
            "i128", "u128", "u256", "i64", "u64", "f64", "i32", "u32", "f32", "i16", "u16", "i8",
            "u8",
        ];
        for suffix in suffixes {
            if rest.starts_with(suffix) {
                self.pos += suffix.len();
                return Some(suffix.to_string());
            }
        }
        None
    }

    fn validate_numeric_body(&mut self, start: usize) {
        let text = &self.source[start..self.pos];
        let suffixes = [
            "i128", "u128", "u256", "i64", "u64", "f64", "i32", "u32", "f32", "i16", "u16", "i8",
            "u8",
        ];
        let body = suffixes
            .iter()
            .find_map(|suffix| text.strip_suffix(suffix))
            .unwrap_or(text);
        let chars: Vec<char> = body.chars().collect();
        for (index, ch) in chars.iter().enumerate() {
            if *ch != '_' {
                continue;
            }
            let valid_prev = index > 0 && chars[index - 1].is_ascii_hexdigit();
            let valid_next = chars
                .get(index + 1)
                .is_some_and(|next| next.is_ascii_hexdigit());
            if !valid_prev || !valid_next {
                self.error(
                    "numeric separators must appear between digits",
                    start,
                    self.pos,
                );
                break;
            }
        }
    }

    fn string(&mut self) {
        let start = self.pos;
        self.bump();
        while let Some(ch) = self.peek() {
            match ch {
                '"' => {
                    self.bump();
                    self.push(TokenKind::StringLit, start, self.pos);
                    return;
                }
                '\\' => self.escape(start),
                _ => {
                    self.bump();
                }
            }
        }
        self.error("unterminated string literal", start, self.pos);
    }

    fn lifetime_or_char(&mut self) {
        let start = self.pos;
        self.bump();
        if matches!(self.peek(), Some(c) if is_ident_start(c)) {
            let ident_start = self.pos;
            self.bump();
            while matches!(self.peek(), Some(c) if is_ident_continue(c)) {
                self.bump();
            }
            if self.peek() != Some('\'') {
                self.push(TokenKind::Lifetime, start, self.pos);
                return;
            }
            if self.pos - ident_start != 1 {
                self.error(
                    "character literal contains more than one scalar",
                    start,
                    self.pos + 1,
                );
            }
            self.bump();
            self.push(TokenKind::CharLit, start, self.pos);
            return;
        }
        if self.peek() == Some('\\') {
            self.escape(start);
        } else if self
            .peek()
            .is_some_and(|c| c != '\'' && c != '\n' && c != '\r')
        {
            self.bump();
        }
        if self.peek() == Some('\'') {
            self.bump();
            self.push(TokenKind::CharLit, start, self.pos);
        } else {
            self.error("unterminated character literal", start, self.pos);
        }
    }

    fn escape(&mut self, start: usize) {
        self.bump();
        match self.peek() {
            Some('n' | 'r' | 't' | '\\' | '"' | '\'' | '0') => {
                self.bump();
            }
            Some('x') => {
                self.bump();
                let esc_start = self.pos;
                for _ in 0..2 {
                    if matches!(self.peek(), Some(c) if c.is_ascii_hexdigit()) {
                        self.bump();
                    }
                }
                if self.pos - esc_start != 2 {
                    self.error("ASCII byte escape needs two hex digits", start, self.pos);
                    return;
                }
                if let Ok(value) = u8::from_str_radix(&self.source[esc_start..self.pos], 16)
                    && value > 0x7f
                {
                    self.error("ASCII byte escape must be in 0x00..0x7F", start, self.pos);
                }
            }
            Some('u') => {
                self.bump();
                if self.peek() != Some('{') {
                    self.error("unicode escape needs `{`", start, self.pos);
                    return;
                }
                self.bump();
                let hex_start = self.pos;
                while matches!(self.peek(), Some(c) if c.is_ascii_hexdigit()) {
                    self.bump();
                }
                let count = self.pos - hex_start;
                if count == 0 || count > 6 || self.peek() != Some('}') {
                    self.error(
                        "unicode escape needs 1-6 hex digits and `}`",
                        start,
                        self.pos,
                    );
                    return;
                }
                if let Ok(value) = u32::from_str_radix(&self.source[hex_start..self.pos], 16)
                    && char::from_u32(value).is_none()
                {
                    self.error(
                        "unicode escape must be a valid scalar value",
                        start,
                        self.pos,
                    );
                }
                self.bump();
            }
            Some('\n' | '\r') => {
                self.bump();
                while matches!(self.peek(), Some(' ' | '\t')) {
                    self.bump();
                }
            }
            _ => self.error("invalid escape sequence", start, self.pos),
        }
    }

    fn starts_with(&self, text: &str) -> bool {
        self.source[self.pos..].starts_with(text)
    }

    fn peek(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn peek_next(&self) -> Option<char> {
        let mut chars = self.source[self.pos..].chars();
        chars.next()?;
        chars.next()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn single(&mut self, kind: TokenKind) {
        let start = self.pos;
        self.bump();
        self.push(kind, start, self.pos);
    }

    fn double(&mut self, kind: TokenKind) {
        let start = self.pos;
        self.bump();
        self.bump();
        self.push(kind, start, self.pos);
    }

    fn take(&mut self, len: usize, kind: TokenKind) {
        let start = self.pos;
        self.pos += len;
        self.push(kind, start, self.pos);
    }

    fn push(&mut self, kind: TokenKind, start: usize, end: usize) {
        self.tokens.push(Token {
            kind,
            lexeme: self.source[start..end].to_string(),
            span: Span::new(start, end),
        });
    }

    fn error(&mut self, message: impl Into<String>, start: usize, end: usize) {
        self.errors.push(LexError {
            message: message.into(),
            span: Span::new(start, end),
        });
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_ident_continue(ch: char) -> bool {
    is_ident_start(ch) || ch.is_ascii_digit()
}

fn valid_digit(ch: char, base: u8) -> bool {
    match base {
        2 => matches!(ch, '0' | '1'),
        8 => matches!(ch, '0'..='7'),
        10 => ch.is_ascii_digit(),
        16 => ch.is_ascii_hexdigit(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contextual_keywords_remain_identifiers() {
        let tokens = lex("send rx.recv(); storage::get(key)").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Ident);
        assert_eq!(tokens[3].kind, TokenKind::Ident);
        assert_eq!(tokens[7].kind, TokenKind::Ident);
    }

    #[test]
    fn lexes_division_and_keeps_comments() {
        let tokens = lex("a / b").unwrap();
        assert_eq!(tokens[1].kind, TokenKind::Slash);
        // `//` still wins over `/` via the guarded arm: only the doc comment survives.
        let doc = lex("/// d\na").unwrap();
        assert_eq!(doc[0].kind, TokenKind::DocComment);
        assert!(
            lex("a // trailing\nb")
                .unwrap()
                .iter()
                .all(|t| t.kind != TokenKind::Slash)
        );
    }

    #[test]
    fn lexes_suffixed_numbers() {
        let tokens = lex("42u32 3.14f32 1e10 42f64 0xFFu8").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::IntLit);
        assert_eq!(tokens[1].kind, TokenKind::FloatLit);
        assert_eq!(tokens[2].kind, TokenKind::FloatLit);
        assert_eq!(tokens[3].kind, TokenKind::FloatLit);
        assert_eq!(tokens[4].kind, TokenKind::IntLit);
    }

    #[test]
    fn rejects_bad_numeric_separators() {
        let err = lex("1__2").unwrap_err();
        assert!(err[0].message.contains("numeric separators"));
    }

    #[test]
    fn validates_escape_ranges() {
        assert!(lex(r#""\x7F""#).is_ok());
        assert!(lex(r#""\x80""#).is_err());
        assert!(lex(r#""\u{10FFFF}""#).is_ok());
        assert!(lex(r#""\u{D800}""#).is_err());
        assert!(lex(r#""\u{110000}""#).is_err());
    }
}
