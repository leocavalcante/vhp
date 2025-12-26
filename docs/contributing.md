---
layout: default
title: Contributing
nav_order: 9
---

# Contributing

Contributions to VHP are welcome! Here's how you can help.

## Ways to Contribute

- **Report bugs** - Open an issue describing the problem
- **Request features** - Suggest new functionality
- **Submit pull requests** - Fix bugs or implement features
- **Improve documentation** - Help make the docs clearer
- **Add tests** - Increase test coverage

## Development Setup

1. Fork and clone the repository:
   ```bash
   git clone https://github.com/YOUR_USERNAME/vhp.git
   cd vhp
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run tests:
   ```bash
   cargo build --release && ./target/release/vhp test -v
   ```

## Adding New Features

### 1. Update Token Types (`src/token.rs`)

Add new token variants:

```rust
pub enum TokenKind {
    // Add new tokens here
    NewKeyword,
}
```

### 2. Update Lexer (`src/lexer.rs`)

Add recognition logic:

```rust
match ident.to_lowercase().as_str() {
    "newkeyword" => TokenKind::NewKeyword,
    // ...
}
```

### 3. Update AST (`src/ast.rs`)

Add new expression or statement types:

```rust
pub enum Stmt {
    NewStatement { /* fields */ },
    // ...
}
```

### 4. Update Parser (`src/parser.rs`)

Add parsing methods:

```rust
fn parse_new_statement(&mut self) -> Result<Stmt, String> {
    // Parse the new statement
}
```

### 5. Update Interpreter (`src/interpreter.rs`)

Add execution logic:

```rust
match stmt {
    Stmt::NewStatement { /* fields */ } => {
        // Execute the new statement
    }
    // ...
}
```

### 6. Add Tests

Create `.vhpt` test files in the appropriate `tests/` subdirectory.

## Code Style

- **No external dependencies** unless absolutely necessary
- **Comprehensive tests** for every feature
- **Clear error messages** with line/column information
- **PHP compatibility** - existing PHP 8.x code should work

## Pull Request Guidelines

1. Create a feature branch from `main`
2. Make your changes with clear commit messages
3. Add tests for new functionality
4. Ensure all tests pass
5. Update documentation if needed
6. Submit a pull request with a clear description

## License

By contributing, you agree that your contributions will be licensed under the BSD 3-Clause License.
