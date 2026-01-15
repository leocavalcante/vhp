/// String parsing module for the VHP lexer
///
/// Handles parsing of string literals with proper escape sequence support
/// for both single-quoted and double-quoted strings.
use crate::lexer::Lexer;

impl Lexer {
    /// Parses a string literal starting from the current position.
    ///
    /// The opening quote character has not yet been consumed.
    /// Returns the string content (without quotes) on success.
    ///
    /// # Arguments
    /// * `quote` - The quote character ('\'', '"') that starts the string
    ///
    /// # Returns
    /// * `Ok(String)` - The parsed string content with escape sequences processed
    /// * `Err(String)` - Error message if the string is not properly terminated
    pub fn read_string(&mut self, quote: char) -> Result<String, String> {
        let start_line = self.line;
        self.advance(); // consume opening quote
        let mut value = String::new();

        while let Some(ch) = self.current() {
            if ch == quote {
                self.advance(); // consume closing quote
                return Ok(value);
            } else if ch == '\\' && quote == '"' {
                // Only process escape sequences in double-quoted strings
                self.advance();
                if let Some(escaped) = self.current() {
                    let (push_backslash, push_escaped) = match escaped {
                        'n' => (false, '\n'),
                        't' => (false, '\t'),
                        'r' => (false, '\r'),
                        '\\' => (false, '\\'),
                        '\'' => (false, '\''),
                        '"' => (false, '"'),
                        '$' => (false, '$'),
                        _ => (true, escaped), // Keep backslash for unrecognized escapes
                    };
                    if push_backslash {
                        value.push('\\');
                    }
                    value.push(push_escaped);
                    self.advance();
                }
            } else if ch == '\\' && quote == '\'' {
                // In single-quoted strings, only \' and \\ are escapes
                self.advance();
                if let Some(escaped) = self.current() {
                    if escaped == '\'' || escaped == '\\' {
                        value.push(escaped);
                        self.advance();
                    } else {
                        // Backslash followed by other char - keep both
                        value.push('\\');
                        value.push(escaped);
                        self.advance();
                    }
                }
            } else {
                value.push(ch);
                self.advance();
            }
        }

        Err(format!(
            "Unterminated string starting at line {}",
            start_line
        ))
    }
}
