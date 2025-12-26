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
2. **Parser** (`parser.rs`): Builds AST from tokens using recursive descent with Pratt parsing for operator precedence
3. **Interpreter** (`interpreter.rs`): Tree-walking interpreter with variable storage and PHP-compatible type coercion

## Project Structure

```
src/
├── main.rs         # CLI entry point
├── token.rs        # Token definitions
├── lexer.rs        # Lexical analysis (source → tokens)
├── ast.rs          # Abstract Syntax Tree definitions
├── parser.rs       # Pratt parser (tokens → AST)
├── interpreter.rs  # Tree-walking interpreter
└── test_runner.rs  # .vhpt test framework

tests/              # 120 tests organized by feature
├── builtins/       # Built-in function tests
├── comments/       # Comment syntax tests
├── control_flow/   # Control flow tests (if, while, for, switch)
├── echo/           # Echo statement tests
├── errors/         # Error handling tests
├── expressions/    # Expression evaluation tests
├── functions/      # User-defined function tests
├── html/           # HTML passthrough tests
├── numbers/        # Numeric literal tests
├── operators/      # Operator tests
├── strings/        # String literal tests
├── tags/           # PHP tag tests
└── variables/      # Variable tests
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

### AST (`ast.rs`)

Defines the abstract syntax tree structure:

```rust
pub enum Expr {
    String(String),
    Integer(i64),
    Variable(String),
    BinaryOp { left: Box<Expr>, op: Operator, right: Box<Expr> },
    FunctionCall { name: String, args: Vec<Expr> },
    // ... and more
}

pub enum Stmt {
    Echo(Vec<Expr>),
    If { condition: Expr, then_branch: Vec<Stmt>, else_branch: Option<Vec<Stmt>> },
    While { condition: Expr, body: Vec<Stmt> },
    Function { name: String, params: Vec<Param>, body: Vec<Stmt> },
    // ... and more
}
```

### Parser (`parser.rs`)

Builds AST from tokens using:

- **Recursive descent** for statements
- **Pratt parsing** for operator precedence in expressions
- Error recovery with meaningful messages

### Interpreter (`interpreter.rs`)

Executes the AST:

- Tree-walking evaluation
- Variable storage in hash maps
- PHP-compatible type coercion
- Built-in function implementations
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
