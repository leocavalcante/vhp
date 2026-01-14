# PCRE Regular Expression Support

## Status: Planned

## Overview

Implement PCRE (Perl Compatible Regular Expressions) functions for pattern matching, replacement, and splitting. This is essential for text processing, validation, and data extraction in PHP applications.

## Current Status

PCRE functions are not implemented. Any use of `preg_match`, `preg_replace`, etc. will fail.

## Background

Regular expressions are a fundamental part of PHP's string manipulation capabilities. They're used extensively in:
- Input validation (email, URLs, phone numbers)
- Data extraction and parsing
- Search and replace operations
- Text processing and transformation
- Log analysis
- HTML/XML parsing (with limitations)

## Requirements

### Core Functions to Implement

1. **preg_match**
   ```php
   preg_match($pattern, $subject, &$matches = null, $flags = 0, $offset = 0): int|false
   ```
   - Perform a regular expression match
   - Return 1 if match, 0 if no match, false on error
   - Populate $matches array with captured groups

2. **preg_match_all**
   ```php
   preg_match_all($pattern, $subject, &$matches = null, $flags = 0, $offset = 0): int|false
   ```
   - Perform global regular expression match
   - Return number of matches
   - Support PREG_PATTERN_ORDER and PREG_SET_ORDER flags

3. **preg_replace**
   ```php
   preg_replace($pattern, $replacement, $subject, $limit = -1, &$count = null): string|array|null
   ```
   - Perform a regular expression search and replace
   - Support string patterns, arrays of patterns
   - Support callback replacements
   - Support $limit for number of replacements

4. **preg_replace_callback**
   ```php
   preg_replace_callback($pattern, $callback, $subject, $limit = -1, &$count = null): string|array|null
   ```
   - Perform a regular expression search and replace using a callback

5. **preg_split**
   ```php
   preg_split($pattern, $subject, $limit = -1, $flags = 0): array|false
   ```
   - Split string by a regular expression
   - Support PREG_SPLIT_NO_EMPTY, PREG_SPLIT_DELIM_CAPTURE, PREG_SPLIT_OFFSET_CAPTURE

6. **preg_grep**
   ```php
   preg_grep($pattern, $array, $flags = 0): array|false
   ```
   - Return array entries that match the pattern
   - Support PREG_GREP_INVERT flag

7. **preg_quote**
   ```php
   preg_quote($str, $delimiter = null): string
   ```
   - Quote regular expression characters

8. **preg_last_error**
   ```php
   preg_last_error(): int
   ```
   - Return the error code of the last PCRE regex execution

9. **preg_last_error_msg**
   ```php
   preg_last_error_msg(): string
   ```
   - Return the error message of the last PCRE regex execution

### PCRE Pattern Support

Support common PCRE features:

1. **Basic Patterns**
   - Literal characters
   - Character classes: `[a-z]`, `[0-9]`, `[^abc]`
   - Metacharacters: `.`, `^`, `$`, `*`, `+`, `?`, `|`, `\`
   - Escaping: `\\`, `\.`, `\d`, `\D`, `\w`, `\W`, `\s`, `\S`

2. **Quantifiers**
   - `*`, `+`, `?`
   - `{n}`, `{n,}`, `{n,m}`
   - Lazy quantifiers: `*?`, `+?`, `??`, `{n,m}?`

3. **Anchors**
   - `^`, `$`
   - `\b`, `\B`
   - `\A`, `\Z`, `\z`

4. **Groups**
   - Capturing groups: `(...)`
   - Non-capturing groups: `(?:...)`
   - Named groups: `(?P<name>...)`, `(?'name'...)`, `(?<name>...)`
   - Backreferences: `\1`, `\g{1}`, `\k<name>`

5. **Assertions**
   - Lookahead: `(?=...)`, `(?!...)`
   - Lookbehind: `(?<=...)`, `(?<!...)`
   - Atomic grouping: `(?>...)`

6. **Modifiers**
   - `i`: Case-insensitive
   - `m`: Multiline mode
   - `s`: Dot matches newline
   - `x`: Extended mode (ignore whitespace and comments)
   - `u`: Unicode support
   - `D`: Dollar end only
   - `U`: Non-greedy matching
   - `X`: Extra PCRE features
   - `J`: Allow duplicate names

### Error Handling

Handle PCRE errors:
- `PREG_INTERNAL_ERROR`
- `PREG_BACKTRACK_LIMIT_ERROR`
- `PREG_RECURSION_LIMIT_ERROR`
- `PREG_BAD_UTF8_ERROR`
- `PREG_BAD_UTF8_OFFSET_ERROR`
- `PREG_JIT_STACKLIMIT_ERROR`
- `PREG_NO_ERROR`

## Implementation Plan

### Option A: Use Rust regex crate (Recommended)

**Pros:**
- Well-tested, high-performance Rust implementation
- Actively maintained
- Good Unicode support
- Easy to integrate

**Cons:**
- Not 100% PCRE compatible (some advanced features missing)
- May need wrapper layer for PHP compatibility

**Files:**
- `runtime/builtins/pcre.rs` (new)
- Update `runtime/builtins/mod.rs` to export

**Implementation Steps:**

1. **Add Dependency**
   ```toml
   [dependencies]
   regex = "1.10"
   ```

2. **Create PCRE Module** (`runtime/builtins/pcre.rs`)

   ```rust
   use regex::{Regex, RegexBuilder};

   pub fn preg_match(
       pattern: Value,
       subject: Value,
       matches: Option<&mut Value>,
       flags: i64,
       offset: i64,
   ) -> Result<Value, String> {
       // Convert pattern to Regex
       // Apply flags
       // Execute match
       // Populate matches array
       // Return result
   }

   // Similar for other functions
   ```

3. **Pattern Translation**
   - Convert PHP-style patterns to Rust regex patterns
   - Handle delimiters: `/pattern/flags`, `#pattern#flags`, etc.
   - Convert PHP modifiers to regex flags
   - Translate PCRE-specific syntax to Rust regex where possible

4. **Matches Array Structure**
   ```php
   preg_match('/(\w+)\s+(\w+)/', 'hello world', $matches);
   // $matches[0] = "hello world" (full match)
   // $matches[1] = "hello" (group 1)
   // $matches[2] = "world" (group 2)
   ```

5. **Register Built-in Functions**
   ```rust
   pub fn register_pcre_functions(builtins: &mut Builtins) {
       builtins.register("preg_match", preg_match);
       builtins.register("preg_match_all", preg_match_all);
       // ... other functions
   }
   ```

### Option B: Use PCRE2 C library

**Pros:**
- 100% PCRE compatible
- Supports all PCRE features

**Cons:**
- Native dependency (requires building/linking PCRE2)
- More complex build setup
- Requires FFI bindings

**Not recommended** unless Option A doesn't provide sufficient compatibility.

### Phase 1: Core Functions

**Tasks:**
- [ ] Add regex crate dependency
- [ ] Create `runtime/builtins/pcre.rs`
- [ ] Implement preg_quote
- [ ] Implement preg_match
- [ ] Implement preg_match_all
- [ ] Add basic tests

### Phase 2: Replace Functions

**Tasks:**
- [ ] Implement preg_replace
- [ ] Implement preg_replace_callback
- [ ] Implement preg_replace_callback_array
- [ ] Add replacement tests
- [ ] Test callback replacements

### Phase 3: Split and Filter

**Tasks:**
- [ ] Implement preg_split
- [ ] Implement preg_grep
- [ ] Add split/filter tests
- [ ] Test all flag combinations

### Phase 4: Error Handling

**Tasks:**
- [ ] Implement preg_last_error
- [ ] Implement preg_last_error_msg
- [ ] Add error handling to all functions
- [ ] Test error conditions

### Phase 5: Advanced Features

**Tasks:**
- [ ] Support named groups
- [ ] Support backreferences
- [ ] Support lookahead/lookbehind
- [ ] Support Unicode (utf8 flag)
- [ ] Test advanced patterns

### Phase 6: Tests

**File:** `tests/pcre/` (new directory)

Test coverage:
- Basic pattern matching
- Character classes
- Quantifiers
- Anchors
- Groups and captures
- Named groups
- Backreferences
- Flags (i, m, s, u, etc.)
- Replacement patterns
- Callback replacements
- Split with different flags
- Grep with invert flag
- Error handling
- Edge cases (empty strings, invalid patterns, etc.)

**Example test:**
```
--TEST--
preg_match basic usage
--FILE--
<?php
$pattern = '/(\w+)\s+(\w+)/';
$subject = 'hello world';
preg_match($pattern, $subject, $matches);
print_r($matches);
--EXPECT--
Array
(
    [0] => hello world
    [1] => hello
    [2] => world
)
```

## Dependencies

- Rust regex crate: `regex = "1.10"` (for Option A)
- Or PCRE2 C library (for Option B, not recommended)

## Testing Strategy

1. **Unit Tests**: Each PCRE function
2. **Integration Tests**: Combined use of multiple functions
3. **Compatibility Tests**: Match PHP 8.x behavior for common patterns
4. **Performance Tests**: Ensure acceptable performance

## Success Criteria

- All basic PCRE functions work correctly
- Most common patterns work (90%+ of typical use cases)
- Flags are properly supported
- Error handling works
- Performance is acceptable
- All tests pass

## Performance Considerations

- Cache compiled regex patterns where possible
- Avoid recompiling patterns on each call
- Use efficient string handling
- Consider JIT compilation for frequently used patterns

## Compatibility Notes

- Some advanced PCRE features may not work with Rust regex crate
- Need to document any incompatibilities
- Consider implementing PCRE2 integration for full compatibility if needed

## Open Questions

- Should we cache compiled patterns? (PHP does this internally)
- How to handle JIT compilation?
- Should we support preg_filter function?

## References

- PHP PCRE documentation: https://www.php.net/manual/en/book.pcre.php
- Rust regex crate: https://docs.rs/regex/
- PCRE2 documentation: https://www.pcre.org/current/doc/html/

## Related Plans

- String Functions Extended (htmlentities, htmlspecialchars, etc.)
