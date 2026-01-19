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

    /// Reads a heredoc or nowdoc string.
    pub fn read_heredoc_nowdoc(&mut self, is_nowdoc: bool) -> Result<String, String> {
        let start_line = self.line;
        let mut marker = String::new();

        if is_nowdoc {
            while let Some(ch) = self.current() {
                if ch == '\'' {
                    self.advance();
                    break;
                } else if ch.is_alphanumeric() || ch == '_' {
                    marker.push(ch);
                    self.advance();
                } else {
                    return Err(format!("Expected nowdoc identifier at line {}", start_line));
                }
            }
        } else {
            while let Some(ch) = self.current() {
                if ch.is_alphanumeric() || ch == '_' {
                    marker.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        if marker.is_empty() {
            return Err(format!(
                "Expected heredoc/nowdoc identifier at line {}",
                start_line
            ));
        }

        while let Some(ch) = self.current() {
            if ch == '\n' {
                self.advance();
                break;
            }
            self.advance();
        }

        let mut chars: Vec<char> = Vec::new();

        while let Some(ch) = self.current() {
            if ch == '\n' {
                let mut pos = self.pos + 1;
                let mut is_end_marker = true;
                for expected_ch in marker.chars() {
                    if let Some(&actual_ch) = self.input.get(pos) {
                        if actual_ch != expected_ch {
                            is_end_marker = false;
                            break;
                        }
                        pos += 1;
                    } else {
                        is_end_marker = false;
                        break;
                    }
                }

                if is_end_marker {
                    if let Some(&next_ch) = self.input.get(pos) {
                        if next_ch == ';'
                            || next_ch == '\n'
                            || (!next_ch.is_alphanumeric() && next_ch != '_')
                        {
                            self.pos = pos;
                            if is_nowdoc {
                                return Ok(chars.iter().collect());
                            } else {
                                return Ok(self.process_heredoc_content(&chars));
                            }
                        }
                    }
                }

                chars.push(ch);
                self.advance();
            } else {
                chars.push(ch);
                self.advance();
            }
        }

        Err(format!(
            "Unterminated heredoc/nowdoc starting at line {} (missing closing marker: {})",
            start_line, marker
        ))
    }

    fn process_heredoc_content(&self, chars: &[char]) -> String {
        let mut result = String::new();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            if ch == '\\' && i + 1 < chars.len() {
                let escaped = chars[i + 1];
                let (push_backslash, push_escaped) = match escaped {
                    'n' => (false, '\n'),
                    't' => (false, '\t'),
                    'r' => (false, '\r'),
                    '\\' => (false, '\\'),
                    '\'' => (false, '\''),
                    '"' => (false, '"'),
                    '$' => (false, '$'),
                    _ => (true, escaped),
                };
                if push_backslash {
                    result.push('\\');
                }
                result.push(push_escaped);
                i += 2;
            } else if ch == '$' {
                if i + 1 < chars.len() {
                    let next_ch = chars[i + 1];
                    if next_ch == '{' {
                        result.push_str(&self.parse_heredoc_brace_var(chars, &mut i));
                    } else if next_ch.is_alphanumeric() || next_ch == '_' {
                        result.push_str(&self.parse_heredoc_simple_var(chars, &mut i));
                    } else if next_ch == '$' {
                        result.push('$');
                        i += 2;
                    } else {
                        result.push(ch);
                        i += 1;
                    }
                } else {
                    result.push(ch);
                    i += 1;
                }
            } else {
                result.push(ch);
                i += 1;
            }
        }

        result
    }

    fn parse_heredoc_brace_var(&self, chars: &[char], i: &mut usize) -> String {
        *i += 2;
        let mut var_name = String::new();
        let mut in_array = false;
        let mut brace_depth = 1;

        while *i < chars.len() {
            match chars[*i] {
                '{' if !in_array => {
                    brace_depth += 1;
                    var_name.push('{');
                    *i += 1;
                }
                '}' if !in_array => {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        *i += 1;
                        return format!("\x00${{{}}}\x00", var_name);
                    } else {
                        var_name.push('}');
                        *i += 1;
                    }
                }
                '[' if !in_array => {
                    in_array = true;
                    var_name.push('[');
                    *i += 1;
                }
                ']' if in_array => {
                    in_array = false;
                    var_name.push(']');
                    *i += 1;
                }
                _ => {
                    var_name.push(chars[*i]);
                    *i += 1;
                }
            }
        }

        chars.iter().collect()
    }

    fn parse_heredoc_simple_var(&self, chars: &[char], i: &mut usize) -> String {
        *i += 1;
        let mut var_name = String::new();
        let mut chars_consumed = 0;

        while *i + chars_consumed < chars.len() {
            let ch = chars[*i + chars_consumed];
            if ch.is_alphanumeric() || ch == '_' {
                var_name.push(ch);
                chars_consumed += 1;
            } else {
                break;
            }
        }

        *i += chars_consumed;

        if var_name.is_empty() {
            return "$".to_string();
        }

        if *i < chars.len() && chars[*i] == '[' {
            let mut array_access = String::new();
            let mut bracket_depth = 1;
            *i += 1;

            while *i < chars.len() && bracket_depth > 0 {
                match chars[*i] {
                    '[' => {
                        bracket_depth += 1;
                        array_access.push('[');
                        *i += 1;
                    }
                    ']' => {
                        bracket_depth -= 1;
                        if bracket_depth == 0 {
                            array_access.push(']');
                            *i += 1;
                        } else {
                            array_access.push(']');
                            *i += 1;
                        }
                    }
                    _ => {
                        array_access.push(chars[*i]);
                        *i += 1;
                    }
                }
            }

            return format!("\x00${}{}\x00", var_name, array_access);
        }

        if *i + 1 < chars.len() && chars[*i] == '-' && chars[*i + 1] == '>' {
            let mut prop_access = String::from("->");
            *i += 2;

            while *i < chars.len() {
                let ch = chars[*i];
                if ch.is_alphanumeric() || ch == '_' {
                    prop_access.push(ch);
                    *i += 1;
                } else {
                    break;
                }
            }

            return format!("\x00${}{}\x00", var_name, prop_access);
        }

        format!("\x00${}\x00", var_name)
    }
}
