mod operators;
mod strings;

use crate::token::{Token, TokenKind};

/// Lexical analyzer for VHP (Vibe-coded Hypertext Preprocessor)
///
/// The lexer converts source code into a sequence of tokens. It handles:
/// - PHP/HTML mode switching
/// - Comment skipping
/// - String and number parsing
/// - Keyword recognition
/// - Operator tokenization
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
    in_php: bool,
}

impl Lexer {
    /// Creates a new lexer for the given input source code.
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
            in_php: false,
        }
    }

    /// Returns the current character without advancing.
    fn current(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    /// Peeks at a character at the given offset from the current position.
    fn peek(&self, offset: usize) -> Option<char> {
        self.input.get(self.pos + offset).copied()
    }

    /// Advances to the next character and returns the current one.
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

    /// Skips whitespace characters without consuming them into tokens.
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Checks if the input at the current position matches the given string.
    fn matches_str(&self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        for (i, ch) in chars.iter().enumerate() {
            if self.peek(i) != Some(*ch) {
                return false;
            }
        }
        true
    }

    /// Advances the position by n characters.
    fn advance_by(&mut self, n: usize) {
        for _ in 0..n {
            self.advance();
        }
    }

    /// Reads a numeric literal (integer or float).
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

    /// Reads an identifier or keyword.
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

    /// Reads a variable name (after the $ is consumed).
    fn read_variable(&mut self) -> String {
        self.advance(); // consume '$'
        self.read_identifier()
    }

    /// Reads HTML content until a PHP tag is encountered.
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

    /// Determines if an identifier is a keyword or a regular identifier.
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
            "fn" => TokenKind::Fn,
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
            "enum" => TokenKind::Enum,
            "clone" => TokenKind::Clone,
            "fiber" => TokenKind::Fiber,
            "with" => TokenKind::With,
            "abstract" => TokenKind::Abstract,
            "final" => TokenKind::Final,
            "try" => TokenKind::Try,
            "catch" => TokenKind::Catch,
            "finally" => TokenKind::Finally,
            "throw" => TokenKind::Throw,
            _ => TokenKind::Identifier(ident.to_string()),
        }
    }

    /// Main tokenization loop. Processes the input and returns a vector of tokens.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while self.current().is_some() {
            if !self.in_php {
                // Outside PHP tags - read HTML
                self.handle_non_php_mode(&mut tokens)?;
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

                // Check for comments
                if self.matches_str("//") {
                    self.skip_single_line_comment();
                    continue;
                }

                if self.matches_str("/*") {
                    self.skip_multi_line_comment();
                    continue;
                }

                // Check for hash comment or attribute
                if self.current() == Some('#') {
                    if self.peek(1) == Some('[') {
                        // This is an attribute start
                        let attr_line = self.line;
                        let attr_column = self.column;
                        self.advance(); // consume '#'
                        tokens.push(Token::new(TokenKind::Hash, attr_line, attr_column));
                        // The '[' will be handled in the next iteration
                        continue;
                    } else {
                        // This is a single-line comment
                        while let Some(ch) = self.current() {
                            if ch == '\n' {
                                break;
                            }
                            self.advance();
                        }
                        continue;
                    }
                }

                let ch = self.current().unwrap();
                let token_kind = self.tokenize_php_element(ch, line, column)?;
                tokens.push(Token::new(token_kind, line, column));
            }
        }

        tokens.push(Token::new(TokenKind::Eof, self.line, self.column));
        Ok(tokens)
    }

    /// Handles tokenization when outside PHP tags (HTML mode).
    fn handle_non_php_mode(&mut self, tokens: &mut Vec<Token>) -> Result<(), String> {
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
        Ok(())
    }

    /// Skips a single-line comment.
    fn skip_single_line_comment(&mut self) {
        while let Some(ch) = self.current() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    /// Skips a multi-line comment.
    fn skip_multi_line_comment(&mut self) {
        self.advance_by(2);
        while self.current().is_some() {
            if self.matches_str("*/") {
                self.advance_by(2);
                break;
            }
            self.advance();
        }
    }

    /// Tokenizes a single PHP element (variable, operator, string, etc.)
    fn tokenize_php_element(
        &mut self,
        ch: char,
        line: usize,
        column: usize,
    ) -> Result<TokenKind, String> {
        match ch {
            // Variables
            '$' => {
                let name = self.read_variable();
                if name.is_empty() {
                    return Err(format!(
                        "Expected variable name after '$' at line {}, column {}",
                        line, column
                    ));
                }
                Ok(TokenKind::Variable(name))
            }

            // Punctuation
            ';' => {
                self.advance();
                Ok(TokenKind::Semicolon)
            }
            ',' => {
                self.advance();
                Ok(TokenKind::Comma)
            }
            '(' => {
                self.advance();
                Ok(TokenKind::LeftParen)
            }
            ')' => {
                self.advance();
                Ok(TokenKind::RightParen)
            }
            '{' => {
                self.advance();
                Ok(TokenKind::LeftBrace)
            }
            '}' => {
                self.advance();
                Ok(TokenKind::RightBrace)
            }
            '[' => {
                self.advance();
                Ok(TokenKind::LeftBracket)
            }
            ']' => {
                self.advance();
                Ok(TokenKind::RightBracket)
            }

            // Operators
            '+' | '-' | '*' | '/' | '%' | '.' | '=' | '!' | '<' | '>' | '&' | '|' | '?' | ':' => {
                self.read_operator(ch)
            }

            // Strings
            '"' | '\'' => {
                let s = self.read_string(ch)?;
                Ok(TokenKind::String(s))
            }

            // Numbers
            _ if ch.is_ascii_digit() => Ok(self.read_number()),

            // Identifiers and keywords
            _ if ch.is_alphabetic() || ch == '_' => {
                let ident = self.read_identifier();
                Ok(self.keyword_or_identifier(&ident))
            }

            _ => Err(format!(
                "Unexpected character '{}' at line {}, column {}",
                ch, line, column
            )),
        }
    }
}
