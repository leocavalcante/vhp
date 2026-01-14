# Essential Missing Features for PHP Drop-in Compatibility

## Status: Planned

## Overview

This document provides a comprehensive roadmap of essential missing PHP features needed to make VHP a true drop-in replacement that can run existing PHP applications without changes.

## Implementation Priority

### Tier 1: Critical for Real-World Applications

These features are blocking most PHP applications from running on VHP:

1. **[PCRE Regular Expressions](./pcre-regex.md)**
   - Usage: Input validation, data extraction, text processing
   - Impact: HIGH - Used in almost all PHP applications
   - Functions: preg_match, preg_replace, preg_split, etc.

2. **[Extended Array Functions](./array-functions-extended.md)**
   - Usage: Data manipulation in every PHP application
   - Impact: HIGH - Missing 40+ commonly used functions
   - Functions: array_slice, array_splice, array_map (already), sort, etc.

3. **[Extended File System Functions](./filesystem-functions.md)**
   - Usage: File operations, uploads, configuration
   - Impact: HIGH - Missing stream operations, directory functions
   - Functions: fopen, fread, fwrite, glob, scandir, etc.

4. **[DateTime Support](./datetime-support.md)**
   - Usage: Scheduling, logging, data analysis
   - Impact: HIGH - No date/time support currently
   - Functions: time(), date(), DateTime class, etc.

5. **[Serialization Support](./serialization-support.md)**
   - Usage: Caching, sessions, data persistence
   - Impact: HIGH - Critical for stateful applications
   - Functions: serialize(), unserialize(), __serialize(), __unserialize()

### Tier 2: Important for Complete Feature Parity

These features complete PHP compatibility and enable more advanced use cases:

6. **[Generator Execution](./generators-execution.md)**
   - Usage: Iteration, stream processing, async patterns
   - Impact: MEDIUM - Generators parsed but not executed
   - Features: yield, yield from, send(), throw()

7. **[Constants Support](./constants-support.md)**
   - Usage: Configuration, version numbers, API constants
   - Impact: MEDIUM - Basic constants work, advanced features missing
   - Features: define(), defined(), trait constants, const visibility

8. **[goto Statement](./goto-statement.md)**
    - Usage: State machines, legacy code, control flow
    - Impact: LOW - Rarely used but part of PHP spec
    - Features: goto, labels, compile-time validation

### Code Organization (Not a PHP Feature)

9. **[Code Organization](./code-organization.md)**
    - Usage: Maintainability and code quality
    - Impact: HIGH - Improves long-term project health
    - Goal: Keep all Rust files under 300-500 lines

## Additional Features to Consider

### String Functions (Extended)

**Missing Functions:**
- str_split, chunk_split, wordwrap
- str_shuffle, str_word_count
- number_format, money_format
- html_entity_decode, htmlentities, htmlspecialchars
- strip_tags, addslashes, stripslashes
- quoted_printable_encode, quoted_printable_decode
- convert_uuencode, convert_uudecode
- ctype_* functions

**Plan:** [Create separate plan after PCRE implementation]

### Mathematical Functions (Extended)

**Missing Functions:**
- log, log10, log1p, exp, expm1
- sin, cos, tan, asin, acos, atan, atan2
- sinh, cosh, tanh, asinh, acosh, atanh
- fmod, intdiv, fdiv
- hypot, deg2rad, rad2deg
- base_convert, bindec, octdec, hexdec, decbin, decoct, dechex
- is_nan, is_finite, is_infinite
- Math constants: M_PI, M_E, etc.

**Plan:** [Create separate plan]

### Error Handling

**Missing Features:**
- error_reporting() function
- trigger_error() function
- set_error_handler() function
- restore_error_handler() function
- set_exception_handler() function
- restore_exception_handler() function
- Error constants: E_ERROR, E_WARNING, E_NOTICE, etc.
- Custom error levels
- Error suppression operator (@)

**Plan:** [Create separate plan]

### Session Support

**Missing Features:**
- session_start() function
- session_destroy() function
- $_SESSION superglobal
- session configuration
- session handlers
- session ID management
- session cookie handling

**Plan:** [Create separate plan - depends on HTTP context]

### JSON Functions (Extended)

**Already Implemented:** json_encode, json_decode

**Missing Features:**
- json_last_error() function
- json_last_error_msg() function
- JSON_* constants (JSON_ERROR_*)
- Encoding/decoding options

**Plan:** [Small extension to existing JSON support]

### Variable Functions

**Missing Functions:**
- call_user_func() function
- call_user_func_array() function
- Forward calls: $func()

**Plan:** [Simple implementation]

### Reflection Functions (Extended)

**Already Implemented:** Class/method/property/parameter attributes

**Missing Functions:**
- class_exists() function
- interface_exists() function
- trait_exists() function
- function_exists() function
- method_exists() function
- property_exists() function
- get_class() function
- get_parent_class() function
- is_a() function
- is_subclass_of() function

**Plan:** [Create separate plan]

### HTTP/Cookie Support

**Missing Features:**
- header() function
- setcookie() function
- setrawcookie() function
- headers_sent() function
- headers_list() function
- $_COOKIE superglobal
- $_SERVER superglobal
- $_GET, $_POST superglobals

**Plan:** [Create separate plan - needs HTTP context]

### Database Extensions

**Missing Features:**
- PDO extension
- MySQLi extension
- PostgreSQL extension
- SQLite extension

**Plan:** [Future phase - after core features complete]

### Stream Wrappers

**Missing Features:**
- Stream context creation (stream_context_create)
- Custom stream wrappers
- stream_filter_* functions
- file://, http://, https:// protocols

**Plan:** [Create separate plan - complex feature]

### XML Support

**Missing Features:**
- SimpleXML extension
- XMLReader extension
- XMLWriter extension
- DOM extension

**Plan:** [Future phase - after core features complete]

## Implementation Order Recommendation

Based on impact and dependencies:

1. **Phase 1: Core Library Functions** (4-6 weeks)
   - PCRE Regular Expressions
   - Extended Array Functions
   - Extended File System Functions

2. **Phase 2: Date/Time and Serialization** (3-4 weeks)
   - DateTime Support
   - Serialization Support

3. **Phase 3: Language Features** (2-3 weeks)
   - Constants Support
   - Generator Execution
   - goto Statement

4. **Phase 4: Extended Features** (4-6 weeks)
   - Extended String Functions
   - Extended Math Functions
   - Error Handling
   - Reflection Functions
   - Variable Functions

5. **Phase 5: Web Support** (4-6 weeks)
   - JSON Functions Extended
   - HTTP/Cookie Support
   - Session Support
   - Stream Wrappers

6. **Phase 6: Database and XML** (Future)
   - PDO extension
   - SimpleXML/DOM
   - Other database extensions

## Testing Strategy

### Compatibility Testing

For each feature:
1. Extract test cases from PHP test suite
2. Create .vhpt test files matching PHP format
3. Run VHP and PHP, compare outputs
4. Document any differences

### Real-World Application Testing

After completing phases 1-3:
1. Test with popular PHP applications:
   - Laravel framework
   - WordPress (subset)
   - PHPUnit
   - Composer (subset)
2. Identify missing features
3. Prioritize based on usage

## Dependencies

### External Crates

Recommended dependencies:
- `regex = "1.10"` - For PCRE
- `chrono = "0.4"` - For DateTime
- `chrono-tz = "0.8"` - For timezone support
- `glob = "0.3"` - For glob() function
- `serde_json = "1.0"` - For JSON (may already have)
- `tempfile = "3.8"` - For temporary files

### Internal Dependencies

- Generators require: Existing function/call system
- Serialization requires: Existing class system
- DateTime requires: Existing class system
- File streams require: Existing Value system
- Constants require: Existing compiler/VM

## Success Criteria

VHP is considered a true drop-in replacement when:

1. **All Tier 1 features are implemented**
2. **Top 100 most-used PHP functions are supported**
3. **Can run common frameworks (Laravel, Symfony, etc.) with minimal changes**
4. **95%+ of tests from popular PHP libraries pass**
5. **Performance is within 50% of PHP for common operations**

## Metrics to Track

- Number of built-in functions implemented
- % of PHP test suite passing
- Number of real-world applications that run
- Performance benchmarks vs PHP
- Code coverage

## Open Questions

1. **Priority of niche features** - Should we implement rarely used functions?
2. **PHP version compatibility** - Target PHP 8.1, 8.2, 8.3, or 8.4?
3. **Performance vs compatibility** - Trade-offs in implementation?
4. **Extension ecosystem** - How to handle PECL extensions?

## Related Documentation

- [AGENTS.md](../../AGENTS.md) - Overall project instructions
- [README.md](../../README.md) - Project overview
- [roadmap.md](../../docs/roadmap.md) - Current roadmap
- [features.md](../../docs/features.md) - Feature documentation

## Plan Files

See individual plan files for detailed implementation guidance:
- [PCRE Regular Expressions](./pcre-regex.md)
- [Extended Array Functions](./array-functions-extended.md)
- [Extended File System Functions](./filesystem-functions.md)
- [DateTime Support](./datetime-support.md)
- [Serialization Support](./serialization-support.md)
- [Generator Execution](./generators-execution.md)
- [Constants Support](./constants-support.md)
- [goto Statement](./goto-statement.md)
- [Code Organization](./code-organization.md)

## Progress Tracking

- [ ] PCRE Regular Expressions
- [ ] Extended Array Functions
- [ ] Extended File System Functions
- [ ] DateTime Support
- [ ] Serialization Support
- [ ] Generator Execution
- [ ] Constants Support
- [ ] goto Statement

### Code Organization

- [ ] Code Organization: Keep files under 300-500 lines

**Total: 0/9 Tier 1-2 features completed**

## Next Steps

1. Review and approve all plan documents
2. Assign priorities based on real-world needs
3. Begin implementation with Tier 1 features
4. Update progress as features are completed
5. Move completed plans to `implemented/` directory
