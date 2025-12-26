<h1 align="center">
  <br>
  <img src="https://raw.githubusercontent.com/leocavalcante/vhp/main/docs/assets/logo.png" alt="VHP Logo" width="200">
  <br>
  VHP
  <br>
</h1>

<h4 align="center">Vibe-coded Hypertext Preprocessor</h4>

<p align="center">
  <em>A PHP superset built entirely in Rust through AI-assisted development</em>
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#usage">Usage</a> •
  <a href="#examples">Examples</a> •
  <a href="#roadmap">Roadmap</a> •
  <a href="#architecture">Architecture</a>
</p>

---

## What is VHP?

**VHP** is a modern PHP implementation written from scratch in Rust. The name stands for "**V**ibe-coded **H**ypertext **P**reprocessor" — reflecting that it's being built entirely through prompts to AI agents ("vibe coding").

### Goals

- **Fast** — Native performance via Rust compilation
- **Secure** — Memory safety guaranteed by Rust's ownership model
- **Zero Dependencies** — Built using only Rust's standard library
- **PHP 8.x Compatible** — Run existing PHP code with zero modifications
- **Progressive** — New features added incrementally with comprehensive tests

## Features

### Basic Syntax
- PHP tags: `<?php`, `?>`, `<?=` (short echo)
- `echo` statement with comma-separated expressions
- String literals (single/double quoted) with escape sequences
- Integer, float, boolean, and null literals
- Comments: `//`, `/* */`, `#`
- HTML passthrough (mixed PHP/HTML)

### Variables & Assignment
```php
<?php
$name = "VHP";
$count = 42;
$count += 8;  // Compound assignment
echo "$name: $count";  // Output: VHP: 50
```

### Operators
```php
<?php
// Arithmetic
echo 2 + 3 * 4;      // 14 (correct precedence!)
echo 2 ** 10;        // 1024 (power operator)

// Comparison
echo 1 == "1" ? "loose" : "strict";   // loose
echo 1 === "1" ? "loose" : "strict";  // strict

// Null coalescing
$user = $name ?? "Anonymous";

// Ternary
echo $age >= 18 ? "adult" : "minor";

// Increment/decrement
$i = 0;
echo ++$i;  // 1 (pre-increment)
echo $i++;  // 1 (post-increment, $i is now 2)
```

### Control Flow
```php
<?php
// If-elseif-else
$score = 85;
if ($score >= 90) {
    echo "A";
} elseif ($score >= 80) {
    echo "B";
} else {
    echo "C";
}

// While loop
$i = 0;
while ($i < 5) {
    echo $i++;
}

// For loop
for ($i = 0; $i < 5; $i++) {
    echo $i;
}

// Do-while loop
$i = 0;
do {
    echo $i++;
} while ($i < 3);

// Switch statement
$day = 1;
switch ($day) {
    case 1:
        echo "Monday";
        break;
    case 2:
        echo "Tuesday";
        break;
    default:
        echo "Other day";
}

// Break and continue
for ($i = 0; $i < 10; $i++) {
    if ($i == 3) continue;  // Skip 3
    if ($i == 7) break;     // Stop at 7
    echo $i;
}
```

### PHP-Compatible Type Coercion
```php
<?php
// Loose equality with type juggling
echo 0 == "0" ? "yes" : "no";     // yes
echo 0 == "" ? "yes" : "no";      // yes
echo 0 == false ? "yes" : "no";   // yes

// Strict equality (no coercion)
echo 0 === "0" ? "yes" : "no";    // no
echo 0 === false ? "yes" : "no";  // no
```

### Functions
```php
<?php
// User-defined functions
function greet($name) {
    return "Hello, " . $name . "!";
}
echo greet("World");  // Hello, World!

// Default parameters
function power($base, $exp = 2) {
    return $base ** $exp;
}
echo power(3);     // 9
echo power(2, 10); // 1024

// Recursive functions
function factorial($n) {
    if ($n <= 1) return 1;
    return $n * factorial($n - 1);
}
echo factorial(5); // 120

// Built-in functions (50+)
echo strlen("Hello");              // 5
echo strtoupper("hello");          // HELLO
echo substr("Hello World", 0, 5);  // Hello
echo str_repeat("ab", 3);          // ababab
echo abs(-42);                     // 42
echo round(3.7);                   // 4
echo max(1, 5, 3);                 // 5
echo sprintf("Name: %s, Age: %d", "John", 25);
```

## Installation

### Build from source

```bash
git clone https://github.com/leocavalcante/vhp.git
cd vhp
cargo build --release
```

The binary will be at `./target/release/vhp`

### Run directly with Cargo

```bash
cargo run --release -- script.php
cargo run --release -- -r 'echo "Hello!";'
```

## Usage

```bash
# Run a PHP file
vhp script.php

# Run inline code
vhp -r 'echo "Hello, World!";'

# Run tests
vhp test           # Compact output
vhp test -v        # Verbose output
vhp test mydir     # Custom test directory

# Help
vhp --help
```

## Examples

### Hello World

```php
<?php
echo "Hello, VHP!\n";
```

### Variables and Math

```php
<?php
$a = 10;
$b = 5;
$c = ($a + $b) * 2 - $a / $b;
echo $c;  // Output: 28
```

### Mixed HTML/PHP

```php
<!DOCTYPE html>
<html>
<body>
    <h1><?= "Welcome to VHP!" ?></h1>
    <?php
    $items = 3;
    echo "<p>You have $items items.</p>";
    ?>
</body>
</html>
```

### Null Safety

```php
<?php
$config = null;
$timeout = $config ?? 30;
echo "Timeout: $timeout";  // Output: Timeout: 30
```

## Roadmap

| Phase | Status | Features |
|-------|--------|----------|
| **1. Variables & Operators** | ✅ Complete | Variables, assignment, arithmetic, comparison, logical, ternary, null coalescing |
| **2. Control Flow** | ✅ Complete | `if`/`else`, `while`, `for`, `do-while`, `switch`, `break`/`continue` |
| **3. Functions** | ✅ Complete | Declarations, calls, returns, parameters, 50+ built-ins |
| **4. Arrays** | 🚧 Next | Literals, access, modification, `foreach` iteration |
| **5. Classes & Objects** | 📋 Planned | Classes, properties, methods, inheritance, interfaces |
| **6. VHP Extensions** | 💡 Future | Type inference, pattern matching, async/await |

## Architecture

```
┌─────────────┐    ┌─────────┐    ┌────────┐    ┌─────────────┐
│ Source Code │───▶│  Lexer  │───▶│ Parser │───▶│ Interpreter │───▶ Output
└─────────────┘    └─────────┘    └────────┘    └─────────────┘
                       │              │               │
                   Tokens          AST           Execute
```

### Project Structure

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

## Testing

VHP uses `.vhpt` files (inspired by PHP's `.phpt` format):

```
--TEST--
Addition operator
--FILE--
<?php
echo 2 + 3;
--EXPECT--
5
```

For error testing:
```
--TEST--
Division by zero
--FILE--
<?php
echo 10 / 0;
--EXPECT_ERROR--
Division by zero
```

Run the test suite:
```bash
vhp test -v

# Output:
# Running 120 tests...
#   PASS Addition operator
#   PASS Basic if statement
#   PASS For loop with break
#   PASS User-defined function
#   PASS Built-in strlen
#   ...
# Tests: 120 total, 119 passed, 0 failed, 0 errors, 1 skipped
```

## Why "Vibe Coding"?

VHP is an experiment in AI-assisted software development. Every line of code has been written through conversations with AI agents (Claude). The goal is to demonstrate that complex systems like programming language interpreters can be built entirely through natural language prompts.

## Contributing

Contributions are welcome! Feel free to:
- Open issues for bugs or feature requests
- Submit pull requests
- Improve documentation
- Add more tests

## License

BSD 3-Clause License - see [LICENSE](LICENSE) for details.

---

<p align="center">
  Built with Rust and AI
</p>
