use crate::token::{Token, TokenKind};

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
    in_php: bool,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
            in_php: false,
        }
    }

    fn current(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.input.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.current();
        if let Some(c) = ch {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn matches_str(&self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        for (i, ch) in chars.iter().enumerate() {
            if self.peek(i) != Some(*ch) {
                return false;
            }
        }
        true
    }

    fn advance_by(&mut self, n: usize) {
        for _ in 0..n {
            self.advance();
        }
    }

    fn read_string(&mut self, quote: char) -> Result<String, String> {
        let start_line = self.line;
        self.advance(); // consume opening quote
        let mut value = String::new();

        while let Some(ch) = self.current() {
            if ch == quote {
                self.advance(); // consume closing quote
                return Ok(value);
            } else if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.current() {
                    let escaped_char = match escaped {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '\'' => '\'',
                        '"' => '"',
                        '$' => '$',
                        _ => escaped,
                    };
                    value.push(escaped_char);
                    self.advance();
                }
            } else {
                value.push(ch);
                self.advance();
            }
        }

        Err(format!("Unterminated string starting at line {}", start_line))
    }

    fn read_number(&mut self) -> TokenKind {
        let mut value = String::new();
        let mut is_float = false;

        while let Some(ch) = self.current() {
            if ch.is_ascii_digit() {
                value.push(ch);
                self.advance();
            } else if ch == '.' && !is_float {
                if let Some(next) = self.peek(1) {
                    if next.is_ascii_digit() {
                        is_float = true;
                        value.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if is_float {
            TokenKind::Float(value.parse().unwrap_or(0.0))
        } else {
            TokenKind::Integer(value.parse().unwrap_or(0))
        }
    }

    fn read_identifier(&mut self) -> String {
        let mut value = String::new();
        while let Some(ch) = self.current() {
            if ch.is_alphanumeric() || ch == '_' {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        value
    }

    fn read_variable(&mut self) -> String {
        self.advance(); // consume '$'
        self.read_identifier()
    }

    fn read_html(&mut self) -> String {
        let mut html = String::new();
        while let Some(ch) = self.current() {
            if self.matches_str("<?php") || self.matches_str("<?=") {
                break;
            }
            html.push(ch);
            self.advance();
        }
        html
    }

    fn keyword_or_identifier(&self, ident: &str) -> TokenKind {
        match ident.to_lowercase().as_str() {
            "echo" => TokenKind::Echo,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            "xor" => TokenKind::Xor,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "elseif" => TokenKind::Elseif,
            "while" => TokenKind::While,
            "do" => TokenKind::Do,
            "for" => TokenKind::For,
            "foreach" => TokenKind::Foreach,
            "as" => TokenKind::As,
            "switch" => TokenKind::Switch,
            "case" => TokenKind::Case,
            "default" => TokenKind::Default,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "function" => TokenKind::Function,
            "return" => TokenKind::Return,
            "match" => TokenKind::Match,
            "class" => TokenKind::Class,
            "new" => TokenKind::New,
            "public" => TokenKind::Public,
            "private" => TokenKind::Private,
            "protected" => TokenKind::Protected,
            "extends" => TokenKind::Extends,
            "parent" => TokenKind::Parent,
            "interface" => TokenKind::Interface,
            "implements" => TokenKind::Implements,
            "trait" => TokenKind::Trait,
            "use" => TokenKind::Use,
            "insteadof" => TokenKind::Insteadof,
            "readonly" => TokenKind::Readonly,
            _ => TokenKind::Identifier(ident.to_string()),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while self.current().is_some() {
            if !self.in_php {
                // Outside PHP tags - read HTML
                if self.matches_str("<?php") {
                    let line = self.line;
                    let column = self.column;
                    self.advance_by(5);
                    self.in_php = true;
                    tokens.push(Token::new(TokenKind::OpenTag, line, column));
                } else if self.matches_str("<?=") {
                    // Short echo tag
                    let line = self.line;
                    let column = self.column;
                    self.advance_by(3);
                    self.in_php = true;
                    tokens.push(Token::new(TokenKind::OpenTag, line, column));
                    tokens.push(Token::new(TokenKind::Echo, line, column + 3));
                } else {
                    let line = self.line;
                    let column = self.column;
                    let html = self.read_html();
                    if !html.is_empty() {
                        tokens.push(Token::new(TokenKind::Html(html), line, column));
                    }
                }
            } else {
                // Inside PHP tags
                self.skip_whitespace();

                if self.current().is_none() {
                    break;
                }

                let line = self.line;
                let column = self.column;

                // Check for close tag
                if self.matches_str("?>") {
                    self.advance_by(2);
                    self.in_php = false;
                    tokens.push(Token::new(TokenKind::CloseTag, line, column));
                    continue;
                }

                // Check for single-line comment
                if self.matches_str("//") {
                    while let Some(ch) = self.current() {
                        if ch == '\n' {
                            break;
                        }
                        self.advance();
                    }
                    continue;
                }

                // Check for multi-line comment
                if self.matches_str("/*") {
                    self.advance_by(2);
                    while self.current().is_some() {
                        if self.matches_str("*/") {
                            self.advance_by(2);
                            break;
                        }
                        self.advance();
                    }
                    continue;
                }

                // Check for hash comment or attribute
                if self.current() == Some('#') {
                    // Check if this is an attribute (#[) or a comment (#)
                    if self.peek(1) == Some('[') {
                        // This is an attribute start
                        self.advance(); // consume '#'
                        tokens.push(Token::new(TokenKind::Hash, line, column));
                        // The '[' will be handled in the next iteration
                    } else {
                        // This is a single-line comment
                        while let Some(ch) = self.current() {
                            if ch == '\n' {
                                break;
                            }
                            self.advance();
                        }
                    }
                    continue;
                }

                let ch = self.current().unwrap();

                let token_kind = match ch {
                    // Variables
                    '$' => {
                        let name = self.read_variable();
                        if name.is_empty() {
                            return Err(format!(
                                "Expected variable name after '$' at line {}, column {}",
                                line, column
                            ));
                        }
                        TokenKind::Variable(name)
                    }

                    // Punctuation
                    ';' => {
                        self.advance();
                        TokenKind::Semicolon
                    }
                    ',' => {
                        self.advance();
                        TokenKind::Comma
                    }
                    '(' => {
                        self.advance();
                        TokenKind::LeftParen
                    }
                    ')' => {
                        self.advance();
                        TokenKind::RightParen
                    }
                    '{' => {
                        self.advance();
                        TokenKind::LeftBrace
                    }
                    '}' => {
                        self.advance();
                        TokenKind::RightBrace
                    }
                    '[' => {
                        self.advance();
                        TokenKind::LeftBracket
                    }
                    ']' => {
                        self.advance();
                        TokenKind::RightBracket
                    }
                    ':' => {
                        self.advance();
                        if self.current() == Some(':') {
                            self.advance();
                            TokenKind::DoubleColon
                        } else {
                            TokenKind::Colon
                        }
                    }

                    // Operators - multi-character first
                    '+' => {
                        self.advance();
                        if self.current() == Some('+') {
                            self.advance();
                            TokenKind::Increment
                        } else if self.current() == Some('=') {
                            self.advance();
                            TokenKind::PlusAssign
                        } else {
                            TokenKind::Plus
                        }
                    }
                    '-' => {
                        self.advance();
                        if self.current() == Some('-') {
                            self.advance();
                            TokenKind::Decrement
                        } else if self.current() == Some('=') {
                            self.advance();
                            TokenKind::MinusAssign
                        } else if self.current() == Some('>') {
                            self.advance();
                            TokenKind::Arrow
                        } else {
                            TokenKind::Minus
                        }
                    }
                    '*' => {
                        self.advance();
                        if self.current() == Some('*') {
                            self.advance();
                            TokenKind::Pow
                        } else if self.current() == Some('=') {
                            self.advance();
                            TokenKind::MulAssign
                        } else {
                            TokenKind::Mul
                        }
                    }
                    '/' => {
                        self.advance();
                        if self.current() == Some('=') {
                            self.advance();
                            TokenKind::DivAssign
                        } else {
                            TokenKind::Div
                        }
                    }
                    '%' => {
                        self.advance();
                        if self.current() == Some('=') {
                            self.advance();
                            TokenKind::ModAssign
                        } else {
                            TokenKind::Mod
                        }
                    }
                    '.' => {
                        self.advance();
                        if self.current() == Some('=') {
                            self.advance();
                            TokenKind::ConcatAssign
                        } else {
                            TokenKind::Concat
                        }
                    }
                    '=' => {
                        self.advance();
                        if self.current() == Some('=') {
                            self.advance();
                            if self.current() == Some('=') {
                                self.advance();
                                TokenKind::Identical
                            } else {
                                TokenKind::Equal
                            }
                        } else if self.current() == Some('>') {
                            self.advance();
                            TokenKind::DoubleArrow
                        } else {
                            TokenKind::Assign
                        }
                    }
                    '!' => {
                        self.advance();
                        if self.current() == Some('=') {
                            self.advance();
                            if self.current() == Some('=') {
                                self.advance();
                                TokenKind::NotIdentical
                            } else {
                                TokenKind::NotEqual
                            }
                        } else {
                            TokenKind::Not
                        }
                    }
                    '<' => {
                        self.advance();
                        if self.current() == Some('=') {
                            self.advance();
                            if self.current() == Some('>') {
                                self.advance();
                                TokenKind::Spaceship
                            } else {
                                TokenKind::LessEqual
                            }
                        } else {
                            TokenKind::LessThan
                        }
                    }
                    '>' => {
                        self.advance();
                        if self.current() == Some('=') {
                            self.advance();
                            TokenKind::GreaterEqual
                        } else {
                            TokenKind::GreaterThan
                        }
                    }
                    '&' => {
                        self.advance();
                        if self.current() == Some('&') {
                            self.advance();
                            TokenKind::And
                        } else {
                            // Single & for by-reference
                            TokenKind::Identifier("&".to_string())
                        }
                    }
                    '|' => {
                        self.advance();
                        if self.current() == Some('|') {
                            self.advance();
                            TokenKind::Or
                        } else {
                            return Err(format!(
                                "Unexpected character '|' at line {}, column {} (bitwise operators not yet supported)",
                                line, column
                            ));
                        }
                    }
                    '?' => {
                        self.advance();
                        if self.current() == Some('?') {
                            self.advance();
                            TokenKind::NullCoalesce
                        } else {
                            TokenKind::QuestionMark
                        }
                    }

                    // Strings
                    '"' | '\'' => {
                        let s = self.read_string(ch)?;
                        TokenKind::String(s)
                    }

                    // Numbers
                    _ if ch.is_ascii_digit() => self.read_number(),

                    // Identifiers and keywords
                    _ if ch.is_alphabetic() || ch == '_' => {
                        let ident = self.read_identifier();
                        self.keyword_or_identifier(&ident)
                    }

                    _ => {
                        return Err(format!(
                            "Unexpected character '{}' at line {}, column {}",
                            ch, line, column
                        ))
                    }
                };

                tokens.push(Token::new(token_kind, line, column));
            }
        }

        tokens.push(Token::new(TokenKind::Eof, self.line, self.column));
        Ok(tokens)
    }
}
