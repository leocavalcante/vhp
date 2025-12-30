# VHP Development Learnings

This document captures lessons learned during VHP development. All agents should consult this before starting work and update it when discovering new patterns or pitfalls.

## Purpose

- **Prevent repeated mistakes**: Document issues so they don't recur
- **Share knowledge**: Patterns discovered by one agent help all agents
- **Improve quality**: Build institutional knowledge over time

## How to Use This Document

### For Architects
Before creating a plan, check:
- Relevant feature patterns that worked well
- Known pitfalls for similar features
- PHP compatibility gotchas

### For Coders
Before implementing, review:
- Common implementation patterns
- Rust/PHP interoperability lessons
- Testing patterns that work

### For QA
When analyzing failures, check:
- If this is a known pattern
- Add new learnings when discovering novel issues

### For All Agents
- **Read before starting work**
- **Update after discovering new patterns**
- **Reference specific learnings in your reports**

---

## Learnings by Category

### PHP Compatibility

#### PHP: Case Sensitivity Rules

**Date**: Initial
**Feature**: General PHP compatibility
**Issue**: PHP has inconsistent case sensitivity rules
**Details**:
- Function names: Case-insensitive (`strlen` == `STRLEN`)
- Variable names: Case-sensitive (`$foo` != `$Foo`)
- Class names: Case-insensitive for instantiation
- Constants: Case-sensitive by default
- Keywords: Case-insensitive (`TRUE` == `true`)
**Prevention**: Always check PHP docs for case sensitivity of the specific feature

#### PHP: Type Coercion in Comparisons

**Date**: Initial
**Feature**: Comparison operators
**Issue**: PHP's loose comparison (`==`) has complex coercion rules
**Details**:
- String "0" is falsy
- Empty array is falsy
- String-to-number comparison converts string to number
- `"10" == "1e1"` is true (both become 10.0)
**Prevention**: Test both `==` and `===` behavior for new features

#### PHP: Null Coalescing Behavior

**Date**: Initial
**Feature**: Null coalescing operator (`??`)
**Issue**: Only checks for null, not falsy values
**Details**:
- `$a ?? 'default'` returns `$a` if not null (even if false, 0, "")
- Different from `$a ?: 'default'` which checks truthiness
**Prevention**: Ensure interpreter checks `is_null()` not `is_falsy()`

---

### Rust Patterns

#### Rust: Ownership in Interpreter Loops

**Date**: Initial
**Feature**: Foreach loops, iterators
**Issue**: Iterating over values while potentially modifying them
**Details**:
- Can't borrow mutably while iterating
- Need to clone or collect before modifying
- Use indices when mutation needed during iteration
**Prevention**: Plan ownership strategy before implementing iteration constructs

#### Rust: String vs &str in AST

**Date**: Initial
**Feature**: AST design
**Issue**: Choosing between owned and borrowed strings
**Details**:
- AST nodes should own their data (use `String`)
- Parsing can use `&str` temporarily
- Interpreter receives owned AST, can move values
**Prevention**: Prefer `String` in AST nodes for simplicity

#### Rust: Error Handling Patterns

**Date**: Initial
**Feature**: Parser/Interpreter errors
**Issue**: Consistent error handling across codebase
**Details**:
- Use `Result<T, String>` for simple error messages
- Include line/column in error messages
- Return early with `?` operator
- Don't panic in library code
**Prevention**: Follow existing error patterns in codebase

---

### Parser Patterns

#### Parser: Operator Precedence

**Date**: Initial
**Feature**: Expression parsing
**Issue**: Getting operator precedence wrong
**Details**:
- Use Pratt parsing for expressions
- Define precedence table explicitly
- Test precedence with complex expressions like `1 + 2 * 3`
- Remember: assignment is right-associative
**Prevention**: Always add precedence tests when adding operators

#### Parser: Statement vs Expression Context

**Date**: Initial
**Feature**: Expression statements
**Issue**: Some constructs are valid in both contexts
**Details**:
- `match` is an expression in PHP 8.0+
- Arrow functions are expressions
- Assignments are expressions that return a value
**Prevention**: Check PHP docs for whether construct is statement or expression

#### Parser: Error Recovery

**Date**: Initial
**Feature**: Syntax error handling
**Issue**: Parser should continue after errors when possible
**Details**:
- Synchronize at statement boundaries
- Don't cascade errors from one bad token
- Provide helpful error messages
**Prevention**: Test parser with intentionally malformed input

---

### Interpreter Patterns

#### Interpreter: Variable Scope

**Date**: Initial
**Feature**: Functions, closures
**Issue**: PHP has function-level scope, not block-level
**Details**:
- Variables in if/while blocks are visible outside
- Functions have their own scope
- `global` keyword imports from global scope
- Closures capture with `use` keyword
**Prevention**: Test scope visibility for new control flow constructs

#### Interpreter: Reference vs Value Semantics

**Date**: Initial
**Feature**: Arrays, objects, parameters
**Issue**: PHP passes arrays by value, objects by reference
**Details**:
- Arrays are copy-on-write (but we copy eagerly)
- Objects are always references
- `&$param` for pass-by-reference parameters
**Prevention**: Document value/reference semantics for each type

#### Interpreter: Return Value Handling

**Date**: Initial
**Feature**: Functions, methods
**Issue**: Functions without explicit return should return null
**Details**:
- Missing return statement = return null
- Early return exits function immediately
- Return in constructor returns the object (usually)
**Prevention**: Always handle the "no explicit return" case

---

### Testing Patterns

#### Testing: Test File Format

**Date**: Initial
**Feature**: .vhpt test files
**Issue**: Incorrect test file format causes confusing failures
**Details**:
- Must have `--TEST--`, `--FILE--`, and `--EXPECT--` or `--EXPECT_ERROR--`
- No blank lines between section headers and content
- `--EXPECT_ERROR--` matches substring of error message
**Prevention**: Validate test file format in test runner

#### Testing: Expected Output Exactness

**Date**: Initial
**Feature**: Test assertions
**Issue**: Whitespace and newline differences
**Details**:
- Trailing newlines matter
- PHP `echo` doesn't add newlines, `print_r` does
- `var_dump` format must match exactly
**Prevention**: Be explicit about newlines in expected output

#### Testing: Edge Cases to Always Test

**Date**: Initial
**Feature**: All features
**Issue**: Missing edge case coverage
**Details**:
Always test:
- Empty input (empty string, empty array)
- Null values
- Boolean edge cases (0, "", false)
- Very large numbers
- Unicode strings
- Nested structures (if applicable)
**Prevention**: Include edge case tests in every Test Strategy

---

### Common Pitfalls

#### Pitfall: Forgetting to Handle All Match Arms

**Date**: Initial
**Feature**: AST matching in interpreter
**Issue**: Adding new AST node but not handling it everywhere
**Details**:
- Rust will warn about non-exhaustive matches
- Easy to miss when there are many match statements
- Use `cargo clippy` to catch these
**Prevention**: Search for all uses of the enum when adding variants

#### Pitfall: Mixing PHP and Rust Truthiness

**Date**: Initial
**Feature**: Conditionals, loops
**Issue**: PHP truthiness differs from Rust
**Details**:
- PHP: 0, "", "0", [], null, false are falsy
- Rust: only `false` is falsy
- Must implement PHP truthiness explicitly
**Prevention**: Use dedicated `is_truthy()` function, don't use Rust's bool conversion

#### Pitfall: Token Position Tracking

**Date**: Initial
**Feature**: Error messages
**Issue**: Losing track of source position
**Details**:
- Tokens should carry line/column information
- AST nodes should preserve position for error reporting
- Error messages without positions are frustrating
**Prevention**: Include position in Token struct, propagate to AST

### PHP Compatibility: Feature Incompatibility Validation

**Date**: 2025-12-30
**Feature**: Asymmetric Visibility (PHP 8.4)
**Issue**: Some PHP 8.4 features are mutually exclusive and need validation
**Details**:
- Asymmetric visibility and property hooks are incompatible (both control write access)
- Asymmetric visibility and readonly are incompatible (both restrict writes)
- These validations should happen at parse time, not interpretation time
- Error messages should clearly explain why features are incompatible
- Pattern: When adding new features that control similar aspects (e.g., write access), check for conflicts with existing features
**Prevention**:
- Before implementing a new feature, check if it overlaps with existing features
- Add validation logic immediately after parsing the conflicting features
- Test the error cases with dedicated test files (e.g., `asymmetric_hooks_error.vhpt`)
- Document incompatibilities in the feature plan

Example validation pattern:
```rust
// After parsing both features
if !prop.hooks.is_empty() && write_visibility.is_some() {
    return Err(format!(
        "Property hooks cannot be combined with asymmetric visibility at line {}, column {}",
        self.current().line,
        self.current().column
    ));
}
```

### Interpreter Patterns: Union Type Coercion Order

**Date**: 2025-12-30
**Feature**: Union types (PHP 8.0) with strict_types
**Issue**: Type coercion tried first match instead of exact match for union types
**Details**:
When coercing a value to a union type like `int|string`, the original implementation tried to coerce to each type in order and returned the first successful coercion. This caused `"hello"` to be coerced to `0` (int) instead of being accepted as a string.

The issue manifested when union types were used without `declare(strict_types=1)`. In coercive mode:
- Value: `"hello"`
- Type hint: `int|string`
- Wrong behavior: Coerce to int → `0`
- Correct behavior: Accept as string (exact match)

**Root Cause**: The coercion algorithm didn't prioritize exact type matches over coercions.

**Solution**: For union types in coercive mode:
1. First check if the value exactly matches any type in the union (no coercion)
2. Only if no exact match, then try coercing to each type
3. Return first successful coercion

```rust
TypeHint::Union(types) => {
    // First, check for exact matches without coercion
    for t in types {
        if value.matches_type(t) {
            return Ok(value.clone());
        }
    }
    // If no exact match, try coercion
    for t in types {
        if let Ok(coerced) = self.coerce_to_type(value, t) {
            return Ok(coerced);
        }
    }
    Err("Cannot coerce to any type in union".to_string())
}
```

**Prevention**:
- For union types, always prefer exact matches over coercions
- Test union types in both strict mode (with `declare(strict_types=1)`) and coercive mode (default)
- Ensure type tests cover both error cases (wrong type) and success cases (multiple accepted types)

### Testing Patterns: Type Tests Need declare(strict_types=1)

**Date**: 2025-12-30
**Feature**: Type hints with error expectations
**Issue**: Tests expecting type errors must include `declare(strict_types=1)`
**Details**:
PHP defaults to coercive type mode, which attempts to convert values to match type hints. Type errors only occur when conversion fails or in strict mode.

Test failures seen:
- Test expects: `Error: must be of type int, string given`
- Actual output: `0` (string coerced to int)

**Root Cause**: Tests were written assuming strict type checking as default, but PHP (and VHP) default to coercive mode.

**Solution**: Any test expecting a type error message must include `declare(strict_types=1);` immediately after the `<?php` tag:

```php
--TEST--
Type error on wrong parameter type
--FILE--
<?php
declare(strict_types=1);

function requireInt(int $n) {
    echo $n;
}
requireInt("not an int");
--EXPECT_ERROR--
must be of type int, string given
```

**Prevention**:
- All type error tests must use `declare(strict_types=1)`
- Tests without `declare(strict_types=1)` should expect coercive behavior (conversions, not errors)
- When adding type-related tests, explicitly decide: strict mode or coercive mode?
- Document in test which mode is being tested

---

## Adding New Learnings

When you discover a new pattern or pitfall, add it using this template:

```markdown
### [Category]: [Brief Title]

**Date**: [Current date]
**Feature**: [Related feature]
**Issue**: [What went wrong or what pattern was discovered]
**Details**:
[Detailed explanation with examples]
**Prevention**: [How to avoid or apply this in future]
```

Categories:
- PHP Compatibility
- Rust Patterns
- Parser Patterns
- Interpreter Patterns
- Testing Patterns
- Common Pitfalls

---

## Revision History

| Date | Change | Author |
|------|--------|--------|
| Initial | Created with foundational learnings | architect |
| 2025-12-30 | Added PHP Compatibility: Feature Incompatibility Validation pattern from asymmetric-visibility implementation | qa |
| 2025-12-30 | Added Interpreter Patterns: Union Type Coercion Order and Testing Patterns: Type Tests Need declare(strict_types=1) | qa |
