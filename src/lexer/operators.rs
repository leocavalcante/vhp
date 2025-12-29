/// Operator tokenization module for the VHP lexer
///
/// Handles recognition and tokenization of all operators including:
/// - Arithmetic operators (+, -, *, /, %, **)
/// - Comparison operators (<, >, <=, >=, <=>, ==, !=, ===, !==)
/// - Assignment operators (=, +=, -=, *=, /=, %=, .=)
/// - Logical operators (&&, ||, and, or, xor)
/// - Special operators (=>, ::, ->, ??, |>)
use crate::lexer::Lexer;
use crate::token::TokenKind;

impl Lexer {
    /// Recognizes and tokenizes operators in PHP code.
    ///
    /// Handles multi-character operators by looking ahead to determine
    /// the complete operator sequence.
    ///
    /// # Arguments
    /// * `ch` - The first character of the operator
    ///
    /// # Returns
    /// A TokenKind representing the recognized operator
    pub fn read_operator(&mut self, ch: char) -> Result<TokenKind, String> {
        let line = self.line;
        let column = self.column;

        let token_kind = match ch {
            // Arithmetic operators
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

            // Concatenation and ellipsis operators
            '.' => {
                self.advance();
                if self.current() == Some('.') && self.peek(1) == Some('.') {
                    // ... ellipsis for variadic/spread
                    self.advance(); // consume second .
                    self.advance(); // consume third .
                    TokenKind::Ellipsis
                } else if self.current() == Some('=') {
                    self.advance();
                    TokenKind::ConcatAssign
                } else {
                    TokenKind::Concat
                }
            }

            // Assignment and comparison operators
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

            // Negation and not-equal operators
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

            // Less-than and related operators
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

            // Greater-than operator
            '>' => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::GreaterThan
                }
            }

            // Logical AND
            '&' => {
                self.advance();
                if self.current() == Some('&') {
                    self.advance();
                    TokenKind::And
                } else {
                    // Single & for by-reference - represented as identifier
                    TokenKind::Identifier("&".to_string())
                }
            }

            // Logical OR and pipe operator
            '|' => {
                self.advance();
                if self.current() == Some('|') {
                    self.advance();
                    TokenKind::Or
                } else if self.current() == Some('>') {
                    self.advance();
                    TokenKind::Pipe
                } else {
                    return Err(format!(
                        "Unexpected character '|' at line {}, column {}. Did you mean '|>' (pipe operator)?",
                        line, column
                    ));
                }
            }

            // Null coalesce operator
            '?' => {
                self.advance();
                if self.current() == Some('?') {
                    self.advance();
                    TokenKind::NullCoalesce
                } else {
                    TokenKind::QuestionMark
                }
            }

            // Colon and double colon (scope resolution)
            ':' => {
                self.advance();
                if self.current() == Some(':') {
                    self.advance();
                    TokenKind::DoubleColon
                } else {
                    TokenKind::Colon
                }
            }

            // Single-character punctuation (handled elsewhere in main lexer)
            _ => {
                return Err(format!(
                    "Unexpected operator character '{}' at line {}, column {}",
                    ch, line, column
                ))
            }
        };

        Ok(token_kind)
    }
}
