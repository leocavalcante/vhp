# VHP: Vibe-coded Hypertext Preprocessor

## Project Overview

VHP is a PHP superset built entirely in Rust with minimal external dependencies. The goal is to create a fast, secure, PHP 8.x-compatible language implementation that can be progressively enhanced with new features.

**Philosophy**: Built entirely through AI-assisted development ("vibe coding"). Every feature should be implemented incrementally with corresponding tests.

## Quick Reference

```bash
# Build (using Make)
make build              # Debug build
make release            # Release build

# Build (using Cargo directly)
cargo build --release

# Run PHP file
./target/release/vhp script.php

# Run inline code
./target/release/vhp -r 'echo "Hello";'

# Run tests
make test               # Build and run tests (compact output)
make test-verbose       # Build and run tests (verbose output)
./target/release/vhp test        # Compact output
./target/release/vhp test -v     # Verbose output

# Lint
make lint               # Run clippy with warnings as errors
```

## Architecture

```
src/
├── main.rs              # CLI entry point, argument parsing
├── token.rs             # Token type definitions (TokenKind, Token)
├── lexer.rs             # Lexical analysis (source code → tokens)
├── test_runner.rs       # .vhpt test framework
├── ast/                 # Abstract Syntax Tree (modularized)
│   ├── mod.rs           # Module exports
│   ├── expr.rs          # Expression AST nodes
│   ├── stmt.rs          # Statement AST nodes
│   └── ops.rs           # Operator definitions
├── parser/              # Recursive descent parser (modularized)
│   ├── mod.rs           # Module exports
│   ├── expr.rs          # Expression parsing
│   ├── stmt.rs          # Statement parsing
│   └── precedence.rs    # Operator precedence (Pratt parsing)
└── interpreter/         # Tree-walking interpreter (modularized)
    ├── mod.rs           # Main interpreter logic
    ├── value.rs         # Value type and coercion
    └── builtins/        # Built-in function modules
        ├── mod.rs       # Module exports
        ├── string.rs    # String functions (24)
        ├── math.rs      # Math functions (9)
        ├── types.rs     # Type checking/conversion functions (13)
        └── output.rs    # Output functions (4)

tests/                   # Test suite organized by feature (120 tests)
├── builtins/            # Built-in function tests (21)
├── comments/            # Comment syntax tests (4)
├── control_flow/        # Control flow tests (25)
├── echo/                # Echo statement tests (6)
├── errors/              # Error handling tests (3)
├── expressions/         # Expression evaluation tests (6)
├── functions/           # User-defined function tests (10)
├── html/                # HTML passthrough tests (3)
├── numbers/             # Numeric literal tests (5)
├── operators/           # Operator tests (23)
├── strings/             # String literal and escape sequence tests (6)
├── tags/                # PHP tag tests (3)
└── variables/           # Variable assignment and scope tests (5)

Makefile                 # Build automation (build, lint, test targets)
```

## Implementation Pipeline

```
Source Code → Lexer → Tokens → Parser → AST → Interpreter → Output
```

1. **Lexer** (`lexer.rs`): Converts source text into tokens, handles PHP/HTML mode switching
2. **Parser** (`parser/`): Builds AST from tokens using recursive descent with Pratt parsing for operator precedence
3. **Interpreter** (`interpreter/`): Tree-walking interpreter with variable storage and PHP-compatible type coercion

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
- [x] **String** (24): `strlen`, `substr`, `strtoupper`, `strtolower`, `trim`, `ltrim`, `rtrim`, `str_repeat`, `str_replace`, `strpos`, `strrev`, `ucfirst`, `lcfirst`, `ucwords`, `str_starts_with`, `str_ends_with`, `str_contains`, `str_pad`, `explode`, `implode`/`join`, `sprintf`, `chr`, `ord`
- [x] **Math** (9): `abs`, `ceil`, `floor`, `round`, `max`, `min`, `pow`, `sqrt`, `rand`/`mt_rand`
- [x] **Type** (13): `intval`, `floatval`/`doubleval`, `strval`, `boolval`, `gettype`, `is_null`, `is_bool`, `is_int`/`is_integer`/`is_long`, `is_float`/`is_double`/`is_real`, `is_string`, `is_numeric`, `isset`, `empty`
- [x] **Output** (4): `print`, `var_dump`, `print_r`, `printf`

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

### 3. Update AST (`ast/`)

Add new expression or statement types:

```rust
// In ast/expr.rs
pub enum Expr {
    String(String),
    Integer(i64),
    Variable(String),  // New expression type
    // ...
}

// In ast/stmt.rs
pub enum Stmt {
    Echo(Vec<Expr>),
    If { condition: Expr, then_branch: Vec<Stmt>, else_branch: Option<Vec<Stmt>> },
    // ...
}
```

### 4. Update Parser (`parser/`)

Add parsing methods:

```rust
// In parser/stmt.rs
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

### 5. Update Interpreter (`interpreter/`)

Add execution logic:

```rust
// In interpreter/mod.rs
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

// For built-in functions, add to the appropriate file in interpreter/builtins/
// String functions → builtins/string.rs
// Math functions → builtins/math.rs
// Type functions → builtins/types.rs
// Output functions → builtins/output.rs
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
- [x] Built-in functions (`strlen`, `substr`, `strtoupper`, `abs`, `ceil`, `floor`, `round`, `max`, `min`, `pow`, `sqrt`, `rand`/`mt_rand`, `intval`, `floatval`/`doubleval`, `strval`, `gettype`, `is_*`, `isset`, `empty`, `var_dump`, `print`, `print_r`, `printf`, `sprintf`, `chr`, `ord`, `explode`, `implode`/`join`, etc.)

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

## Documentation Workflow

**IMPORTANT**: After making any significant changes to the codebase, update the following documentation files to reflect the current state:

1. **CLAUDE.md** - This file (project instructions for AI assistants)
   - Update Architecture section if file structure changes
   - Update Current Features if new features are added
   - Update Built-in Functions list if new functions are added
   - Update Roadmap section if phases are completed

2. **README.md** - Public-facing documentation
   - Update Features section
   - Update Built-in Functions lists
   - Update Roadmap table
   - Update Architecture/Project Structure if changed

3. **docs/** - GitHub Pages documentation site
   - `docs/architecture.md` - Update if file structure changes
   - `docs/features.md` - Update if new features or built-in functions are added
   - `docs/roadmap.md` - Update if phases are completed or new phases added
   - `docs/index.md` - Update Quick Start or Goals if needed

### When to Update Documentation

- After adding new built-in functions
- After completing a roadmap phase
- After refactoring file structure
- After adding new language features (operators, statements, etc.)
- After adding new tests that cover new functionality
