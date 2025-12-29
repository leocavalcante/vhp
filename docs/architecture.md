---
layout: default
title: Architecture
nav_order: 7
---

# Architecture

VHP follows a classic interpreter pipeline with three main stages.

## Pipeline Overview

```
┌─────────────┐    ┌─────────┐    ┌────────┐    ┌─────────────┐
│ Source Code │───▶│  Lexer  │───▶│ Parser │───▶│ Interpreter │───▶ Output
└─────────────┘    └─────────┘    └────────┘    └─────────────┘
                       │              │               │
                   Tokens          AST           Execute
```

1. **Lexer** (`lexer.rs`): Converts source text into tokens, handles PHP/HTML mode switching
2. **Parser** (`parser/`): Builds AST from tokens using recursive descent with Pratt parsing for operator precedence
3. **Interpreter** (`interpreter/`): Tree-walking interpreter with variable storage and PHP-compatible type coercion

## Project Structure

```
src/
├── main.rs              # CLI entry point
├── token.rs             # Token definitions
├── lexer.rs             # Lexical analysis (source → tokens)
├── test_runner.rs       # .vhpt test framework
├── ast/                 # Abstract Syntax Tree (modularized)
│   ├── mod.rs           # Module exports
│   ├── expr.rs          # Expression AST nodes
│   ├── stmt.rs          # Statement AST nodes
│   └── ops.rs           # Operator definitions
├── parser/              # Pratt parser (modularized)
│   ├── mod.rs           # Module exports
│   ├── expr.rs          # Expression parsing
│   ├── stmt.rs          # Statement parsing
│   └── precedence.rs    # Operator precedence
└── interpreter/         # Tree-walking interpreter (modularized)
    ├── mod.rs           # Main interpreter logic
    ├── value.rs         # Value type and coercion
    └── builtins/        # Built-in function modules
        ├── mod.rs       # Module exports
        ├── string.rs    # String functions (23)
        ├── math.rs      # Math functions (9)
        ├── array.rs     # Array functions (13)
        ├── types.rs     # Type functions (14)
        ├── output.rs    # Output functions (4)
        └── reflection.rs # Reflection functions (8)

tests/                   # 306 tests organized by feature
├── arrays/              # Array tests (18)
├── attributes/          # Attribute syntax and reflection tests (29)
├── builtins/            # Built-in function tests (26)
├── classes/             # Class and object tests (50)
├── comments/            # Comment syntax tests (4)
├── control_flow/        # Control flow tests (29)
├── echo/                # Echo statement tests (6)
├── enums/               # Enum tests (16)
├── errors/              # Error handling tests (8)
├── expressions/         # Expression evaluation tests (17)
├── functions/           # User-defined function tests (20)
├── html/                # HTML passthrough tests (5)
├── interfaces/          # Interface tests (7)
├── numbers/             # Numeric literal tests (5)
├── operators/           # Operator tests (37)
├── strings/             # String literal and escape sequence tests (8)
├── tags/                # PHP tag tests (4)
├── traits/              # Trait tests (9)
└── variables/           # Variable assignment and scope tests (8)
```

## Components

### Token (`token.rs`)

Defines all token types recognized by the lexer:

```rust
pub enum TokenKind {
    // Keywords
    Echo, If, Else, While, For, Function, Return,

    // Literals
    StringLiteral(String),
    Integer(i64),
    Float(f64),
    True, False, Null,

    // Operators
    Plus, Minus, Star, Slash,
    // ... and more
}
```

### Lexer (`lexer.rs`)

Converts source code into a stream of tokens:

- Handles PHP/HTML mode switching
- Recognizes keywords, literals, and operators
- Tracks line and column positions for error reporting
- Supports escape sequences in strings

### AST (`ast/`)

Defines the abstract syntax tree structure in separate modules:

```rust
// ast/expr.rs - Expression nodes
pub enum Expr {
    String(String),
    Integer(i64),
    Variable(String),
    Binary { left: Box<Expr>, op: BinaryOp, right: Box<Expr> },
    FunctionCall { name: String, args: Vec<Expr> },
    // ... and more
}

// ast/stmt.rs - Statement nodes
pub enum Stmt {
    Echo(Vec<Expr>),
    If { condition: Expr, then_branch: Vec<Stmt>, else_branch: Option<Vec<Stmt>> },
    While { condition: Expr, body: Vec<Stmt> },
    Function { name: String, params: Vec<FunctionParam>, body: Vec<Stmt> },
    // ... and more
}

// ast/ops.rs - Operator definitions
pub enum BinaryOp { Add, Sub, Mul, Div, ... }
pub enum UnaryOp { Neg, Not, PreInc, PreDec, ... }
pub enum AssignOp { Assign, AddAssign, SubAssign, ... }
```

### Parser (`parser/`)

Builds AST from tokens using:

- **Recursive descent** for statements (in `parser/stmt.rs`)
- **Pratt parsing** for operator precedence in expressions (in `parser/expr.rs` and `parser/precedence.rs`)
- Error recovery with meaningful messages

### Interpreter (`interpreter/`)

Executes the AST:

- Tree-walking evaluation (in `interpreter/mod.rs`)
- Variable storage in hash maps
- PHP-compatible type coercion (in `interpreter/value.rs`)
- Built-in function implementations organized by category:
  - `builtins/string.rs` - String manipulation functions
  - `builtins/math.rs` - Mathematical functions
  - `builtins/types.rs` - Type checking and conversion
  - `builtins/output.rs` - Output and debugging functions
- Control flow handling (break, continue, return)

## Design Principles

### Zero Dependencies

VHP uses only Rust's standard library. This ensures:
- Fast compilation
- Small binary size
- No supply chain risks
- Easy auditing

### PHP Compatibility

Every feature aims for PHP 8.x behavioral compatibility:
- Same type coercion rules
- Same operator precedence
- Same truthiness semantics
- Case-insensitive keywords and function names

### Incremental Development

Each feature is:
- Implemented in small, focused changes
- Covered by comprehensive tests
- Documented in this documentation
