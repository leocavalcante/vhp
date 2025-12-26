# VHP: Vibe-coded Hypertext Preprocessor

## Project Overview

VHP is a PHP superset built entirely in Rust with minimal external dependencies. The goal is to create a fast, secure, PHP 8.x-compatible language implementation that can be progressively enhanced with new features.

**Philosophy**: Built entirely through AI-assisted development ("vibe coding"). Every feature should be implemented incrementally with corresponding tests.

## Quick Reference

```bash
# Build
cargo build --release

# Run PHP file
./target/release/vhp script.php

# Run inline code
./target/release/vhp -r 'echo "Hello";'

# Run tests
./target/release/vhp test        # Compact output
./target/release/vhp test -v     # Verbose output
```

## Architecture

```
src/
├── main.rs         # CLI entry point, argument parsing
├── token.rs        # Token type definitions (TokenKind, Token)
├── lexer.rs        # Lexical analysis (source code → tokens)
├── ast.rs          # AST node definitions (Expr, Stmt, Program)
├── parser.rs       # Recursive descent parser (tokens → AST)
├── interpreter.rs  # Tree-walking interpreter (AST → output)
└── test_runner.rs  # .vhpt test framework

tests/              # Test suite organized by feature (120 tests)
├── builtins/       # Built-in function tests
├── comments/       # Comment syntax tests
├── control_flow/   # Control flow tests (if, while, for, switch, break, continue)
├── echo/           # Echo statement tests
├── errors/         # Error handling tests
├── expressions/    # Expression evaluation tests
├── functions/      # User-defined function tests
├── html/           # HTML passthrough tests
├── numbers/        # Numeric literal tests
├── operators/      # Operator tests (arithmetic, comparison, logical)
├── strings/        # String literal and escape sequence tests
├── tags/           # PHP tag tests (<?php, <?=, ?>)
└── variables/      # Variable assignment and scope tests
```

## Implementation Pipeline

```
Source Code → Lexer → Tokens → Parser → AST → Interpreter → Output
```

1. **Lexer** (`lexer.rs`): Converts source text into tokens, handles PHP/HTML mode switching
2. **Parser** (`parser.rs`): Builds AST from tokens using recursive descent with Pratt parsing for operator precedence
3. **Interpreter** (`interpreter.rs`): Tree-walking interpreter with variable storage and PHP-compatible type coercion

## Current Features (v0.1.0)

### Basic Syntax
- [x] PHP tags: `<?php`, `?>`, `<?=` (short echo)
- [x] `echo` statement with comma-separated expressions
- [x] String literals (single/double quoted)
- [x] Escape sequences: `\n`, `\t`, `\r`, `\\`, `\'`, `\"`, `\$`
- [x] Integer and float literals
- [x] Boolean literals (`true`, `false`)
- [x] Null literal (`null`)
- [x] Comments: `//`, `/* */`, `#`
- [x] HTML passthrough (mixed PHP/HTML)
- [x] Case-insensitive keywords (`echo`, `ECHO`, `Echo`, `TRUE`, `NULL`, etc.)

### Variables & Assignment
- [x] Variables (`$name`)
- [x] Assignment (`$x = value`)
- [x] Compound assignment (`+=`, `-=`, `*=`, `/=`, `%=`, `.=`)
- [x] Undefined variables return `null`

### Operators
- [x] Arithmetic: `+`, `-`, `*`, `/`, `%`, `**` (power)
- [x] String concatenation: `.`
- [x] Comparison: `==`, `===`, `!=`, `!==`, `<`, `>`, `<=`, `>=`, `<=>` (spaceship)
- [x] Logical: `&&`, `||`, `!`, `and`, `or`, `xor`
- [x] Null coalescing: `??`
- [x] Ternary: `? :`
- [x] Increment/decrement: `++$x`, `$x++`, `--$x`, `$x--`
- [x] Unary negation: `-$x`

### Control Flow
- [x] `if`/`elseif`/`else` statements
- [x] `else if` (two tokens) syntax
- [x] Single-statement blocks (no braces required)
- [x] `while` loops
- [x] `do`...`while` loops
- [x] `for` loops with optional init/condition/update
- [x] `switch`/`case`/`default` with fall-through
- [x] `break` statement
- [x] `continue` statement
- [x] Nested loop support with proper break/continue scoping

### Functions
- [x] Function declarations with `function` keyword
- [x] Function calls (user-defined and built-in)
- [x] Return statements with optional values
- [x] Parameters (by value and by reference with `&`)
- [x] Default parameter values
- [x] Recursive functions
- [x] Case-insensitive function names (PHP-compatible)
- [x] Local scope (function variables don't leak to global)

### Built-in Functions (50+)
- [x] **String**: `strlen`, `substr`, `strtoupper`, `strtolower`, `trim`, `ltrim`, `rtrim`, `str_repeat`, `str_replace`, `strpos`, `stripos`, `strrev`, `ucfirst`, `lcfirst`, `ucwords`, `str_starts_with`, `str_ends_with`, `str_contains`, `str_pad`, `sprintf`, `chr`, `ord`
- [x] **Math**: `abs`, `ceil`, `floor`, `round`, `max`, `min`, `pow`, `sqrt`, `rand`
- [x] **Type**: `intval`, `floatval`, `strval`, `boolval`, `gettype`, `is_null`, `is_bool`, `is_int`, `is_integer`, `is_long`, `is_float`, `is_double`, `is_real`, `is_string`, `is_numeric`
- [x] **Variable**: `isset`, `empty`, `var_dump`, `print_r`, `print`

### Type Coercion (PHP-compatible)
- [x] Loose equality (`==`) with type coercion
- [x] Strict equality (`===`) without type coercion
- [x] PHP truthiness rules for boolean context
- [x] Automatic type conversion for arithmetic operations

## Adding New Features

### 1. Update Token Types (`token.rs`)

Add new token variants to `TokenKind`:

```rust
pub enum TokenKind {
    // Add new tokens here
    If,
    Else,
    Variable(String),  // $name
    // ...
}
```

### 2. Update Lexer (`lexer.rs`)

Add recognition logic in `tokenize()`:

```rust
// For keywords, add to the identifier match:
match ident.to_lowercase().as_str() {
    "echo" => TokenKind::Echo,
    "if" => TokenKind::If,      // New keyword
    // ...
}

// For new character sequences, add new match arms
```

### 3. Update AST (`ast.rs`)

Add new expression or statement types:

```rust
pub enum Expr {
    String(String),
    Integer(i64),
    Variable(String),  // New expression type
    // ...
}

pub enum Stmt {
    Echo(Vec<Expr>),
    If { condition: Expr, then_branch: Vec<Stmt>, else_branch: Option<Vec<Stmt>> },
    // ...
}
```

### 4. Update Parser (`parser.rs`)

Add parsing methods:

```rust
fn parse_if(&mut self) -> Result<Stmt, String> {
    // Parse if statement
}

fn parse_statement(&mut self) -> Result<Option<Stmt>, String> {
    match token.kind {
        TokenKind::If => Ok(Some(self.parse_if()?)),
        // ...
    }
}
```

### 5. Update Interpreter (`interpreter.rs`)

Add execution logic:

```rust
pub fn execute(&mut self, program: &Program) -> io::Result<()> {
    for stmt in &program.statements {
        match stmt {
            Stmt::If { condition, then_branch, else_branch } => {
                // Execute if statement
            }
            // ...
        }
    }
}
```

### 6. Add Tests

Create `.vhpt` test files in appropriate `tests/` subdirectory:

```
--TEST--
Descriptive test name
--FILE--
<?php
// PHP code to test
--EXPECT--
expected output
```

For error tests:
```
--TEST--
Error case description
--FILE--
<?php
// Code that should error
--EXPECT_ERROR--
partial error message to match
```

## Test Format (.vhpt)

| Section | Required | Description |
|---------|----------|-------------|
| `--TEST--` | Yes | Test name |
| `--FILE--` | Yes | PHP code to execute |
| `--EXPECT--` | Yes* | Expected output |
| `--EXPECT_ERROR--` | Yes* | Expected error substring |
| `--DESCRIPTION--` | No | Detailed description |
| `--SKIPIF--` | No | Reason to skip (for unimplemented features) |

*One of `--EXPECT--` or `--EXPECT_ERROR--` required.

## Roadmap

### Phase 1: Variables & Operators ✅ Complete
- [x] Variables (`$name`)
- [x] Assignment (`=`) and compound assignment (`+=`, `-=`, etc.)
- [x] Arithmetic operators (`+`, `-`, `*`, `/`, `%`, `**`)
- [x] String concatenation (`.`)
- [x] Comparison operators (`==`, `===`, `!=`, `!==`, `<`, `>`, `<=`, `>=`, `<=>`)
- [x] Logical operators (`&&`, `||`, `!`, `and`, `or`, `xor`)
- [x] Null coalescing (`??`)
- [x] Ternary operator (`? :`)
- [x] Increment/decrement (`++`, `--`)

### Phase 2: Control Flow ✅ Complete
- [x] `if`/`elseif`/`else`
- [x] `while` loops
- [x] `do`...`while` loops
- [x] `for` loops
- [x] `foreach` loops (syntax parsing - requires arrays for full support)
- [x] `switch`/`case`/`default`
- [x] `break`/`continue`

### Phase 3: Functions ✅ Complete
- [x] Function declarations
- [x] Function calls
- [x] Return statements
- [x] Parameters (by value, by reference)
- [x] Default parameter values
- [x] Built-in functions (`strlen`, `substr`, `strtoupper`, `abs`, `ceil`, `floor`, `round`, `max`, `min`, `pow`, `sqrt`, `rand`, `intval`, `floatval`, `strval`, `gettype`, `is_*`, `isset`, `empty`, `var_dump`, `print`, `sprintf`, `chr`, `ord`, etc.)

### Phase 4: Arrays (Next)
- [ ] Array literals (`[]`, `array()`)
- [ ] Array access (`$arr[0]`, `$arr['key']`)
- [ ] Array modification
- [ ] `foreach` with arrays

### Phase 5: Classes & Objects
- [ ] Class declarations
- [ ] Properties and methods
- [ ] Constructors
- [ ] Visibility (public, private, protected)
- [ ] Inheritance
- [ ] Interfaces and traits

### Phase 6: VHP Extensions (Beyond PHP)
- [ ] Type inference
- [ ] Pattern matching
- [ ] Null coalescing improvements
- [ ] Async/await
- [ ] Better error messages

## Code Style Guidelines

- **No external dependencies** unless absolutely necessary
- **Comprehensive tests** for every feature
- **Clear error messages** with line/column information
- **PHP compatibility** - existing PHP 8.x code should work
- **Incremental development** - small, focused changes

## Common Patterns

### Adding a Binary Operator

1. Add token: `Plus`, `Minus`, etc.
2. Lexer: recognize the character
3. AST: `BinaryOp { left: Expr, op: Operator, right: Expr }`
4. Parser: implement operator precedence (Pratt parsing recommended)
5. Interpreter: evaluate both sides, apply operation

### Adding a Keyword Statement

1. Add token: `If`, `While`, `For`, etc.
2. Lexer: add to keyword matching
3. AST: add statement variant
4. Parser: add `parse_<keyword>()` method
5. Interpreter: add execution logic

## Debugging Tips

- Use `cargo build` for faster iteration (debug build)
- Add `println!` in lexer/parser to trace token stream
- The `--EXPECT_ERROR--` test section is useful for testing error paths
- Run single test file: create a temp `.vhpt` and run `vhp test path/to/file.vhpt`

## Dependencies

Currently zero external crates. Rust std library only.

If adding dependencies becomes necessary, prefer:
- Well-maintained, minimal crates
- No transitive dependency bloat
- Security-audited when possible
