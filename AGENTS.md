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
        ├── string.rs    # String functions (23)
        ├── math.rs      # Math functions (9)
        ├── array.rs     # Array functions (15)
        ├── types.rs     # Type checking/conversion functions (14)
        ├── output.rs    # Output functions (4)
        └── reflection.rs # Reflection functions (8)

tests/                   # Test suite organized by feature (356 tests)
├── arrays/              # Array tests (18)
├── attributes/          # Attribute syntax and reflection tests (29)
├── builtins/            # Built-in function tests (26)
├── classes/             # Class and object tests (55 including anonymous classes)
├── comments/            # Comment syntax tests (4)
├── control_flow/        # Control flow tests (29)
├── echo/                # Echo statement tests (6)
├── enums/               # Enum tests (16)
├── errors/              # Error handling tests (8)
├── expressions/         # Expression evaluation tests (17)
├── functions/           # User-defined function tests (42 including arrow functions and first-class callables)
├── html/                # HTML passthrough tests (5)
├── interfaces/          # Interface tests (7)
├── numbers/             # Numeric literal tests (5)
├── operators/           # Operator tests (37)
├── strings/             # String literal and escape sequence tests (8)
├── tags/                # PHP tag tests (4)
├── traits/              # Trait tests (9)
└── variables/           # Variable assignment and scope tests (8)

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
- [x] Pipe operator: `|>` (PHP 8.5)

### Control Flow
- [x] `if`/`elseif`/`else` statements
- [x] `else if` (two tokens) syntax
- [x] Single-statement blocks (no braces required)
- [x] `while` loops
- [x] `do`...`while` loops
- [x] `for` loops with optional init/condition/update
- [x] `foreach` loops with arrays
- [x] `switch`/`case`/`default` with fall-through
- [x] `break` statement
- [x] `continue` statement
- [x] Nested loop support with proper break/continue scoping

### Arrays
- [x] Array literals (`[1, 2, 3]`)
- [x] Associative arrays (`["key" => "value"]`)
- [x] Array access (`$arr[0]`, `$arr["key"]`)
- [x] Array modification (`$arr[0] = value`)
- [x] Array append (`$arr[] = value`)
- [x] Nested arrays
- [x] `foreach` with value only (`foreach ($arr as $val)`)
- [x] `foreach` with key-value (`foreach ($arr as $key => $val)`)

### Functions
- [x] Function declarations with `function` keyword
- [x] Function calls (user-defined and built-in)
- [x] Return statements with optional values
- [x] Parameters (by value and by reference with `&`)
- [x] Default parameter values
- [x] Recursive functions
- [x] Case-insensitive function names (PHP-compatible)
- [x] Local scope (function variables don't leak to global)
- [x] Variadic functions (`...$args`)
- [x] Argument unpacking (`func(...$array)`)

### Built-in Functions (73)
- [x] **String** (23): `strlen`, `substr`, `strtoupper`, `strtolower`, `trim`, `ltrim`, `rtrim`, `str_repeat`, `str_replace`, `strpos`, `strrev`, `ucfirst`, `lcfirst`, `ucwords`, `str_starts_with`, `str_ends_with`, `str_contains`, `str_pad`, `explode`, `implode`/`join`, `sprintf`, `chr`, `ord`
- [x] **Math** (9): `abs`, `ceil`, `floor`, `round`, `max`, `min`, `pow`, `sqrt`, `rand`/`mt_rand`
- [x] **Array** (15): `count`/`sizeof`, `array_push`, `array_pop`, `array_shift`, `array_unshift`, `array_keys`, `array_values`, `in_array`, `array_search`, `array_reverse`, `array_merge`, `array_key_exists`, `range`, `array_first`, `array_last`
- [x] **Type** (14): `intval`, `floatval`/`doubleval`, `strval`, `boolval`, `gettype`, `is_null`, `is_bool`, `is_int`/`is_integer`/`is_long`, `is_float`/`is_double`/`is_real`, `is_string`, `is_array`, `is_numeric`, `isset`, `empty`
- [x] **Output** (4): `print`, `var_dump`, `print_r`, `printf`
- [x] **Reflection** (8): `get_class_attributes`, `get_method_attributes`, `get_property_attributes`, `get_function_attributes`, `get_parameter_attributes`, `get_method_parameter_attributes`, `get_interface_attributes`, `get_trait_attributes`

### Type Coercion (PHP-compatible)
- [x] Loose equality (`==`) with type coercion
- [x] Strict equality (`===`) without type coercion
- [x] PHP truthiness rules for boolean context
- [x] Automatic type conversion for arithmetic operations

### Classes & Objects
- [x] Class declarations with `class` keyword
- [x] Properties with visibility modifiers (`public`, `private`, `protected`)
- [x] Methods with `$this` reference
- [x] Constructors (`__construct`)
- [x] Object instantiation with `new`
- [x] Property access and modification (`$obj->property`)
- [x] Method calls (`$obj->method()`)
- [x] Static method calls (`ClassName::method()`)
- [x] Default property values
- [x] Multiple objects from same class with independent state
- [x] Case-insensitive class and method names (PHP-compatible)
- [x] Inheritance with `extends` keyword
- [x] Parent method calls with `parent::method()`
- [x] Interfaces with method signatures and constants
- [x] Interface inheritance (`extends Interface1, Interface2`)
- [x] Class implementation of interfaces (`implements Interface1, Interface2`)
- [x] Traits with properties and methods
- [x] Trait composition in classes (`use Trait1, Trait2`)
- [x] Trait conflict resolution (`insteadof`, `as`)
- [x] Traits using other traits
- [x] Constructor Property Promotion (PHP 8.0)
- [x] Readonly Properties (PHP 8.1)
- [x] Readonly Classes (PHP 8.2)
- [x] Object cloning with `clone` keyword (PHP 5.0)
- [x] Clone with property modification syntax (PHP 8.4)
- [x] Abstract classes and methods
- [x] Final classes and methods
- [x] Anonymous classes (PHP 7.0)

### Match Expressions (PHP 8.0)
- [x] Basic match syntax: `match($expr) { value => result }`
- [x] Multiple conditions per arm: `1, 2, 3 => result`
- [x] Default arm: `default => result`
- [x] Strict (===) comparison semantics
- [x] Match as expression (returns value)
- [x] Unhandled match error when no arm matches and no default

### Attributes (PHP 8.0)
- [x] Basic attribute syntax: `#[AttributeName]`
- [x] Attributes with positional arguments: `#[Route("/path")]`
- [x] Attributes with named arguments: `#[Route(path: "/path")]`
- [x] Multiple attributes: `#[Attr1] #[Attr2]` or `#[Attr1, Attr2]`
- [x] Attributes on classes, interfaces, traits
- [x] Attributes on methods, properties, functions
- [x] Attributes on parameters (including constructor promotion)
- [x] Attributes on interface methods and constants
- [x] Attributes parsing and storage in AST
- [x] Attribute reflection API (retrieving attributes at runtime)

### Enums (PHP 8.1)
- [x] Pure enums (cases without values)
- [x] Backed enums (int and string backing types)
- [x] Enum case access (`EnumName::CASE` syntax)
- [x] Case properties (`->name`, `->value`)
- [x] Built-in methods: `cases()`, `from()`, `tryFrom()`
- [x] Case-sensitive case names
- [x] Validation and error handling

### Pipe Operator (PHP 8.5)
- [x] Basic pipe syntax: `$value |> function(...)`
- [x] Function chaining: `$x |> f(...) |> g(...) |> h(...)`
- [x] Additional arguments: `$text |> substr(..., 0, 5)`
- [x] Left-to-right evaluation
- [x] Low precedence (higher than assignment, lower than ternary)
- [x] Works with built-in functions
- [x] Works with user-defined functions
- [x] Piped value inserted as first argument
- [x] Multi-line pipe chains

**Example:**
```php
<?php
$text = "  hello world  ";
$result = $text
    |> trim(...)
    |> strtoupper(...)
    |> substr(..., 0, 5);
echo $result; // "HELLO"
```

### Arrow Functions (PHP 7.4)
- [x] Basic arrow function syntax: `fn($param) => expression`
- [x] Automatic variable capture by value from outer scope
- [x] Single expression body (not statement block)
- [x] Implicit return of expression result
- [x] Support for default parameters
- [x] Support for variadic parameters (`...$args`)
- [x] Nested arrow functions
- [x] Variable function calls: `$func()` syntax
- [x] Callable type (closure values)

**Example:**
```php
<?php
// Basic arrow function
$double = fn($n) => $n * 2;
echo $double(5); // 10

// Auto-captures from outer scope by value
$multiplier = 3;
$multiply = fn($n) => $n * $multiplier;
echo $multiply(4); // 12

// Nested arrow functions
$outer = 5;
$f = fn($x) => fn($y) => $x + $y + $outer;
$g = $f(10);
echo $g(3); // 18
```

### First-Class Callables (PHP 8.1)
- [x] Basic syntax: `functionName(...)` creates closure
- [x] Works with built-in functions
- [x] Works with user-defined functions
- [x] Closures can be stored in variables
- [x] Closures can be passed as arguments
- [x] Integration with pipe operator: `$x |> trim(...) |> strtoupper(...)`
- [ ] Method callables: `$obj->method(...)` (parsing only, not yet callable)
- [ ] Static method callables: `Class::method(...)` (parsing only, not yet callable)

**Example:**
```php
<?php
// Create closure from function
$len = strlen(...);
echo $len("hello"); // 5

// Use with pipe operator
$result = "  hello  "
    |> trim(...)
    |> strtoupper(...);
echo $result; // HELLO

// Pass as argument
function apply($value, $func) {
    return $func($value);
}
echo apply("hello", strtoupper(...)); // HELLO
```

### Anonymous Classes (PHP 7.0)
- [x] Basic syntax: `new class { ... }`
- [x] Constructor arguments: `new class($arg) { ... }`
- [x] Extending classes: `new class extends Base { ... }`
- [x] Implementing interfaces: `new class implements Interface { ... }`
- [x] Unique internal class names (`class@anonymous$N`)
- [x] Full property and method support
- [x] Implicitly final (cannot be extended)

**Example:**
```php
<?php
// Basic anonymous class
$obj = new class {
    public function greet() {
        return "Hello!";
    }
};
echo $obj->greet(); // Hello!

// With constructor
$greeter = new class("World") {
    private $name;
    public function __construct($name) {
        $this->name = $name;
    }
    public function greet() {
        return "Hello, " . $this->name;
    }
};
echo $greeter->greet(); // Hello, World

// Extending a class
class Base {
    protected $value;
    public function __construct($val) {
        $this->value = $val;
    }
}

$obj = new class(42) extends Base {
    public function getValue() {
        return $this->value;
    }
};
echo $obj->getValue(); // 42
```

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

### Phase 4: Arrays ✅ Complete
- [x] Array literals (`[]`)
- [x] Associative arrays (`["key" => "value"]`)
- [x] Array access (`$arr[0]`, `$arr['key']`)
- [x] Array modification and append
- [x] `foreach` with arrays (value only and key-value)
- [x] Built-in array functions (`count`, `array_push`, `array_pop`, `in_array`, `array_keys`, `array_values`, `array_merge`, `array_reverse`, `array_search`, `array_key_exists`, `range`, etc.)

### Phase 5: Classes & Objects ✅ Complete
- [x] Class declarations with `class` keyword
- [x] Properties with visibility modifiers (`public`, `private`, `protected`)
- [x] Methods with `$this` reference
- [x] Constructors (`__construct`)
- [x] Object instantiation with `new`
- [x] Property access and method calls
- [x] Static method calls (`ClassName::method()`)
- [x] Default property values
- [x] Multiple objects from same class with independent state
- [x] Case-insensitive class and method names (PHP-compatible)
- [x] Inheritance with `extends` keyword
- [x] Parent method calls with `parent::method()`
- [x] Interfaces with method signatures and constants
- [x] Interface inheritance (`extends Interface1, Interface2`)
- [x] Class implementation of interfaces (`implements Interface1, Interface2`)
- [x] Traits with properties and methods
- [x] Trait composition in classes (`use Trait1, Trait2`)
- [x] Trait conflict resolution (`insteadof`, `as`)
- [x] Traits using other traits
- [x] Constructor Property Promotion (PHP 8.0)
- [x] Readonly Properties (PHP 8.1)
- [x] Readonly Classes (PHP 8.2)
- [x] Object cloning with `clone` keyword (PHP 5.0)
- [x] Clone with property modification syntax (PHP 8.4)

### Phase 6: Modern PHP 8.x Features ✅
- [x] Match Expressions (PHP 8.0)
- [x] Named Arguments (PHP 8.0)
- [x] Attributes (PHP 8.0) - Full support including reflection API
- [x] Enums (PHP 8.1) - Pure and backed enums with built-in methods
- [x] Pipe Operator (PHP 8.5) - Functional-style function chaining
- [x] Fibers (PHP 8.1)
- [x] Arrow Functions (PHP 7.4) - Short closures with automatic variable capture
- [x] First-Class Callables (PHP 8.1) - `strlen(...)` syntax for function closures
- [x] Anonymous Classes (PHP 7.0) - Inline class definitions

### Phase 7: PHP Core Language Compatibility (Planned)
Essential PHP features for compatibility with standard PHP code.

**Exception Handling:**
- [ ] try/catch/finally statements
- [ ] throw keyword and expressions (PHP 8.0)
- [ ] Exception class and multiple catch blocks
- [ ] Multi-catch (PHP 7.1) - `catch (TypeA | TypeB $e)`

**Type System:**
- [ ] Type declarations (int, string, float, bool, array, callable, object)
- [ ] Nullable types (PHP 7.1) - `?int`
- [ ] Union types (PHP 8.0) - `int|string`
- [ ] Intersection types (PHP 8.1) - `Iterator&Countable`
- [ ] DNF types (PHP 8.2) - `(A&B)|C`
- [ ] mixed, void, never, static return types

**Namespaces:**
- [ ] namespace declaration
- [ ] use statements and aliases
- [ ] Group use declarations (PHP 7.0)

**Generators:**
- [ ] yield keyword and yield from (PHP 7.0)
- [ ] Generator return values (PHP 7.0)
- [ ] Iterator interfaces

**Abstract & Final:**
- [x] abstract classes and methods
- [x] final classes and methods
- [ ] final constants (PHP 8.1)

**Magic Methods:**
- [ ] __toString(), __invoke(), __get()/__set()
- [ ] __isset()/__unset(), __call()/__callStatic()
- [ ] __clone(), __debugInfo()
- [ ] __serialize()/__unserialize() (PHP 7.4)

**Additional OOP:**
- [ ] Anonymous classes (PHP 7.0)
- [ ] Property hooks (PHP 8.4)
- [ ] Asymmetric visibility (PHP 8.4)
- [ ] Static properties and late static binding
- [ ] #[\Override] attribute (PHP 8.3)

**Control Flow:**
- [ ] Alternative syntax (`if:`, `endif;`, etc.)
- [ ] goto statement
- [ ] declare directive (`strict_types`)

**Functions:**
- [x] Arrow functions (PHP 7.4) - `fn($x) => $x * 2`
- [x] Variadic functions and argument unpacking
- [x] First-class callables (PHP 8.1) - `strlen(...)`

### Phase 8: PHP 8.5 Features (Planned)
- [ ] URI Extension - `Uri\Rfc3986\Uri` class
- [ ] Clone with syntax - `clone($obj, ['prop' => 'value'])`
- [ ] #[\NoDiscard] attribute
- [ ] Closures in constant expressions
- [ ] First-class callables in constants
- [x] array_first() / array_last()
- [ ] #[\DelayedTargetValidation]
- [ ] Final property promotion
- [ ] Attributes on constants
- [ ] Error backtraces for fatal errors

### Phase 9: Standard Library Expansion (Planned)
- [ ] PCRE regex (preg_match, preg_replace, etc.)
- [ ] Array functions (array_map, array_filter, array_reduce, sorting)
- [ ] Math functions (trigonometry, logarithms)
- [ ] JSON functions (json_encode, json_decode)
- [ ] DateTime classes
- [ ] File system functions

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

1. **AGENTS.md** - This file (project instructions for AI assistants)
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

4. **docs/plans/** - Feature implementation plans
   - `docs/plans/planned/` - Contains detailed implementation plans for features not yet implemented
   - `docs/plans/implemented/` - Contains plans for features that have been completed
   - **IMPORTANT**: After implementing a feature, move its plan from `planned/` to `implemented/`
   - This helps track what has been completed and serves as historical reference

### When to Update Documentation

- After adding new built-in functions
- After completing a roadmap phase
- After refactoring file structure
- After adding new language features (operators, statements, etc.)
- After adding new tests that cover new functionality
- **After implementing a planned feature** - Move the plan file from `docs/plans/planned/` to `docs/plans/implemented/`
