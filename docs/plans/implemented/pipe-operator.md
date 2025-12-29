# Plan: Pipe Operator (PHP 8.5)

## Overview

The Pipe Operator (`|>`) is a functional-style operator that allows chaining function calls in a more readable, left-to-right manner. It passes the result of the left-hand expression as the first argument to the right-hand function call.

The pipe operator transforms code like `c(b(a($x)))` into `$x |> a(...) |> b(...) |> c(...)`, making data transformations more readable and maintaining the order of operations as written.

**PHP Example:**
```php
// Without pipe operator
$result = strtoupper(trim(substr($text, 0, 10)));

// With pipe operator (PHP 8.5+)
$result = $text
    |> substr(..., 0, 10)
    |> trim(...)
    |> strtoupper(...);
```

The `...` placeholder indicates where the piped value should be inserted. If omitted, the value is passed as the first argument.

## Files to Modify

| File | Changes |
|------|---------|
| `src/token.rs` | Add `Pipe` token (`|>`) |
| `src/lexer.rs` | Recognize `|>` operator |
| `src/ast/expr.rs` | Add `Pipe` expression variant |
| `src/ast/ops.rs` | Add `Pipe` to `BinaryOp` enum |
| `src/parser/precedence.rs` | Add pipe precedence level |
| `src/parser/expr.rs` | Parse pipe expressions |
| `src/interpreter/mod.rs` | Implement pipe evaluation |
| `tests/operators/pipe_*.vhpt` | Test files |
| Documentation files | Update feature lists |

## Implementation Steps

### Step 1: Add Token (`src/token.rs`)

Add the `Pipe` token to the `TokenKind` enum after line 112 (after `DoubleColon`):

```rust
    DoubleColon,       // ::
    Pipe,              // |> (PHP 8.5 pipe operator)
    Hash,              // # (for attributes when followed by [)
```

### Step 2: Update Lexer (`src/lexer.rs`)

Find the section where `|` would be tokenized (around line 200-400, in the character matching logic). Add pipe operator recognition:

Look for where other two-character operators are handled (like `=>`, `==`, etc.) and add:

```rust
'|' => {
    self.advance();
    if self.current() == Some('>') {
        self.advance();
        tokens.push(Token::new(TokenKind::Pipe, line, column));
    } else {
        // PHP uses | for bitwise OR, but VHP doesn't support it yet
        return Err(format!(
            "Unexpected character '|' at line {}, column {}. Did you mean '|>' (pipe operator)?",
            line, column
        ));
    }
}
```

**Note**: Since VHP doesn't currently support bitwise operators, a lone `|` should produce a helpful error message.

### Step 3: Add Operator to AST (`src/ast/ops.rs`)

Add `Pipe` to the `BinaryOp` enum after line 32 (after `NullCoalesce`):

```rust
    // Null coalescing
    NullCoalesce, // ??

    // Pipe operator
    Pipe,         // |> (PHP 8.5)
}
```

### Step 4: Update Precedence (`src/parser/precedence.rs`)

Add a new precedence level for the pipe operator. It should be low precedence (evaluated last in most contexts), but higher than assignment. Insert after line 11:

```rust
pub enum Precedence {
    None = 0,
    Assignment = 1,    // = += -= etc.
    Pipe = 2,          // |> (pipe operator - PHP 8.5)
    Ternary = 3,       // ?:
    NullCoalesce = 4,  // ??
    Or = 5,            // || or
    And = 6,           // && and
    Xor = 7,           // xor
    Equality = 8,      // == === != !==
    Comparison = 9,    // < > <= >= <=>
    Concat = 10,       // .
    AddSub = 11,       // + -
    MulDiv = 12,       // * / %
    Pow = 13,          // ** (right associative)
    Unary = 14,        // ! - ++ --
}
```

Then add the pipe operator to the `get_precedence` function (around line 36):

```rust
        TokenKind::QuestionMark => Precedence::Ternary,
        TokenKind::Pipe => Precedence::Pipe,
        TokenKind::NullCoalesce => Precedence::NullCoalesce,
```

The pipe operator is left-associative (default), so no changes needed to `is_right_assoc`.

### Step 5: Parse Pipe Expressions (`src/parser/expr.rs`)

The pipe operator is a binary operator and will be handled by the existing Pratt parser infrastructure. However, we need to handle it specially in the binary expression evaluation.

Find the `parse_infix` method (around line 400-600) where binary operators are parsed. The pipe operator will be automatically handled by the precedence system, but we need to ensure it creates the correct AST node.

In the binary operator matching section, add special handling for pipe. Find where `BinaryOp` variants are created from tokens (around line 450-500):

```rust
fn token_to_binary_op(kind: &TokenKind) -> Option<BinaryOp> {
    match kind {
        TokenKind::Plus => Some(BinaryOp::Add),
        TokenKind::Minus => Some(BinaryOp::Sub),
        TokenKind::Mul => Some(BinaryOp::Mul),
        TokenKind::Div => Some(BinaryOp::Div),
        TokenKind::Mod => Some(BinaryOp::Mod),
        TokenKind::Pow => Some(BinaryOp::Pow),
        TokenKind::Concat => Some(BinaryOp::Concat),
        TokenKind::Equal => Some(BinaryOp::Equal),
        TokenKind::Identical => Some(BinaryOp::Identical),
        TokenKind::NotEqual => Some(BinaryOp::NotEqual),
        TokenKind::NotIdentical => Some(BinaryOp::NotIdentical),
        TokenKind::LessThan => Some(BinaryOp::LessThan),
        TokenKind::GreaterThan => Some(BinaryOp::GreaterThan),
        TokenKind::LessEqual => Some(BinaryOp::LessEqual),
        TokenKind::GreaterEqual => Some(BinaryOp::GreaterEqual),
        TokenKind::Spaceship => Some(BinaryOp::Spaceship),
        TokenKind::And => Some(BinaryOp::And),
        TokenKind::Or => Some(BinaryOp::Or),
        TokenKind::Xor => Some(BinaryOp::Xor),
        TokenKind::NullCoalesce => Some(BinaryOp::NullCoalesce),
        TokenKind::Pipe => Some(BinaryOp::Pipe),
        _ => None,
    }
}
```

### Step 6: Interpret Pipe Expressions (`src/interpreter/mod.rs`)

Find the `eval_binary` method (around line 400-600) and add a case for the `Pipe` operator:

```rust
fn eval_binary(&mut self, left: &Expr, op: &BinaryOp, right: &Expr) -> Result<Value, String> {
    match op {
        // ... existing operators ...

        BinaryOp::Pipe => self.eval_pipe(left, right),
    }
}
```

Add a new method `eval_pipe` after the `eval_binary` method (around line 650):

```rust
/// Evaluate pipe operator: $value |> function(...)
/// The left side is evaluated and passed as the first argument to the right side function call.
///
/// Examples:
/// - $x |> strtoupper(...)  => strtoupper($x)
/// - $x |> substr(..., 0, 5) => substr($x, 0, 5)
/// - $x |> max(..., 10)     => max($x, 10)
fn eval_pipe(&mut self, left: &Expr, right: &Expr) -> Result<Value, String> {
    // Evaluate the left side to get the value to pipe
    let piped_value = self.eval_expr(left)?;

    // The right side must be a function call or method call
    match right {
        Expr::FunctionCall { name, args } => {
            // Check if any argument is the placeholder (...)
            // For now, we'll insert the piped value as the first argument
            // Full placeholder support can be added later

            // Create a new argument list with the piped value as first arg
            let mut new_args = vec![Argument {
                name: None,
                value: Box::new(Expr::String("__piped__".to_string())), // Placeholder
            }];
            new_args.extend_from_slice(args);

            // Evaluate the function call with the piped value
            // We need to temporarily inject the piped value
            self.eval_function_call_with_piped_value(name, args, piped_value)
        }

        Expr::MethodCall { object, method, args } => {
            // For method calls, insert piped value as first argument after $this
            self.eval_method_call_with_piped_value(object, method, args, piped_value)
        }

        _ => Err(format!(
            "Pipe operator right-hand side must be a function call or method call, got {:?}",
            right
        )),
    }
}

/// Helper to evaluate function call with a piped value as first argument
fn eval_function_call_with_piped_value(
    &mut self,
    name: &str,
    args: &[Argument],
    piped_value: Value,
) -> Result<Value, String> {
    // Evaluate all arguments
    let mut arg_values = vec![piped_value]; // Piped value is first argument

    for arg in args {
        arg_values.push(self.eval_expr(&arg.value)?);
    }

    // Check for built-in functions first
    if let Some(result) = self.try_builtin_function(name, &arg_values)? {
        return Ok(result);
    }

    // Check for user-defined functions
    let func_lower = name.to_lowercase();
    if let Some(func) = self.functions.get(&func_lower).cloned() {
        return self.call_user_function(&func, &arg_values, &HashMap::new());
    }

    Err(format!("Undefined function: {}", name))
}

/// Helper to evaluate method call with a piped value
fn eval_method_call_with_piped_value(
    &mut self,
    object: &Expr,
    method: &str,
    args: &[Argument],
    piped_value: Value,
) -> Result<Value, String> {
    let object_value = self.eval_expr(object)?;

    // Evaluate arguments with piped value as first
    let mut arg_values = vec![piped_value];
    for arg in args {
        arg_values.push(self.eval_expr(&arg.value)?);
    }

    match object_value {
        Value::Object(mut instance) => {
            let method_lower = method.to_lowercase();

            // Look up the method in the class definition
            if let Some(class_def) = self.classes.get(&instance.class_name.to_lowercase()).cloned() {
                if let Some(method_func) = class_def.methods.get(&method_lower) {
                    // Set current object context
                    let saved_object = self.current_object.clone();
                    let saved_class = self.current_class.clone();
                    self.current_object = Some(instance.clone());
                    self.current_class = Some(class_def.name.clone());

                    // Call the method
                    let result = self.call_user_function(method_func, &arg_values, &HashMap::new());

                    // Update instance if properties were modified
                    if let Some(updated) = self.current_object.take() {
                        instance = updated;
                    }

                    // Restore context
                    self.current_object = saved_object;
                    self.current_class = saved_class;

                    return result;
                }

                return Err(format!(
                    "Method '{}' not found on class '{}'",
                    method, class_def.name
                ));
            }

            Err(format!("Class '{}' not found", instance.class_name))
        }
        _ => Err(format!(
            "Attempting to call method on non-object ({})",
            object_value.get_type()
        )),
    }
}
```

**Note**: The above implementation assumes the piped value is always inserted as the first argument. Full RFC-compliant placeholder (`...`) support would require additional AST changes to mark placeholder positions, which can be deferred to a future enhancement.

### Step 7: Add Tests (`tests/operators/`)

Create a new subdirectory for pipe operator tests or add to existing operators directory.

#### Test 1: `tests/operators/pipe_basic.vhpt`

```php
--TEST--
Pipe operator - basic function chaining
--FILE--
<?php
$text = "  hello world  ";
$result = $text |> trim(...) |> strtoupper(...);
echo $result;
--EXPECT--
HELLO WORLD
```

#### Test 2: `tests/operators/pipe_with_args.vhpt`

```php
--TEST--
Pipe operator - function with additional arguments
--FILE--
<?php
$text = "hello world";
$result = $text |> substr(..., 0, 5) |> strtoupper(...);
echo $result;
--EXPECT--
HELLO
```

#### Test 3: `tests/operators/pipe_multiple.vhpt`

```php
--TEST--
Pipe operator - multiple transformations
--FILE--
<?php
$numbers = [1, 2, 3, 4, 5];
$result = $numbers
    |> array_reverse(...)
    |> array_pop(...)
    |> pow(..., 2);
echo $result;
--EXPECT--
1
```

#### Test 4: `tests/operators/pipe_precedence.vhpt`

```php
--TEST--
Pipe operator - precedence with other operators
--FILE--
<?php
$x = 10;
$result = $x + 5 |> pow(..., 2);
echo $result;
--EXPECT--
225
```

#### Test 5: `tests/operators/pipe_associativity.vhpt`

```php
--TEST--
Pipe operator - left associativity
--FILE--
<?php
function add5($x) { return $x + 5; }
function mul2($x) { return $x * 2; }
function sub3($x) { return $x - 3; }

$result = 10 |> add5(...) |> mul2(...) |> sub3(...);
echo $result;
--EXPECT--
27
```

**Explanation**: `10 |> add5(...)` = 15, `15 |> mul2(...)` = 30, `30 |> sub3(...)` = 27

#### Test 6: `tests/operators/pipe_with_variables.vhpt`

```php
--TEST--
Pipe operator - using variables in arguments
--FILE--
<?php
$text = "hello";
$length = 3;
$result = $text |> substr(..., 0, $length) |> strtoupper(...);
echo $result;
--EXPECT--
HEL
```

#### Test 7: `tests/operators/pipe_builtins.vhpt`

```php
--TEST--
Pipe operator - with various built-in functions
--FILE--
<?php
$arr = [3, 1, 4, 1, 5];
$result = $arr |> array_reverse(...) |> count(...);
echo $result;
--EXPECT--
5
```

#### Test 8: `tests/operators/pipe_user_function.vhpt`

```php
--TEST--
Pipe operator - with user-defined functions
--FILE--
<?php
function double($x) {
    return $x * 2;
}

function addTen($x) {
    return $x + 10;
}

$result = 5 |> double(...) |> addTen(...);
echo $result;
--EXPECT--
20
```

#### Test 9: `tests/operators/pipe_non_function_error.vhpt`

```php
--TEST--
Pipe operator - error when right side is not a function call
--FILE--
<?php
$x = 10;
$result = $x |> 5;
--EXPECT_ERROR--
Pipe operator right-hand side must be a function call
```

#### Test 10: `tests/operators/pipe_multiline.vhpt`

```php
--TEST--
Pipe operator - works across multiple lines
--FILE--
<?php
$text = "  HELLO WORLD  ";
$result = $text
    |> trim(...)
    |> strtolower(...)
    |> ucfirst(...);
echo $result;
--EXPECT--
Hello world
```

### Step 8: Update Documentation

#### Update `docs/features.md`

Add a new section in the operators or modern PHP features section:

```markdown
## Pipe Operator (PHP 8.5+)

The pipe operator (`|>`) enables functional-style function chaining, making data transformations more readable by flowing left-to-right.

### Basic Usage

```php
$text = "  hello  ";
$result = $text |> trim(...) |> strtoupper(...);
echo $result; // "HELLO"
```

### With Arguments

The piped value is inserted as the first argument, with additional arguments following:

```php
$text = "hello world";
$result = $text |> substr(..., 0, 5) |> strtoupper(...);
echo $result; // "HELLO"
```

### Multiple Transformations

```php
$numbers = [1, 2, 3, 4, 5];
$result = $numbers
    |> array_reverse(...)
    |> array_pop(...)
    |> pow(..., 2);
echo $result; // 1
```

### Benefits

- **Readability**: Operations flow in the order they're applied
- **Composability**: Easy to add or remove transformation steps
- **Functional Style**: Enables point-free programming patterns

### Notes

- The pipe operator has lower precedence than most operators but higher than assignment
- Left-associative: `a |> b |> c` is evaluated as `(a |> b) |> c`
- The `...` placeholder indicates where the piped value is inserted (always first argument in current implementation)
```

#### Update `AGENTS.md`

Find the roadmap Phase 6 section (around line 434) and update:

```markdown
### Phase 6: Modern PHP 8.x Features (In Progress)
- [x] Match Expressions (PHP 8.0)
- [x] Named Arguments (PHP 8.0)
- [x] Attributes (PHP 8.0)
- [x] Enums (PHP 8.1)
- [ ] Fibers (PHP 8.1)
- [x] Pipe Operator (PHP 8.5)
```

#### Update `README.md`

Find the operators section and add:

```markdown
- [x] Pipe operator: `|>` (PHP 8.5)
```

#### Update `docs/roadmap.md`

Update Phase 6 (around line 101):

```markdown
- [x] **Pipe Operator** (PHP 8.5) - Functional-style operator for chaining function calls.
```

## Key Considerations

### PHP Compatibility

1. **RFC Status**: The pipe operator is an RFC for PHP 8.5. The exact syntax and semantics may change before final release. This implementation follows the current RFC proposal.

2. **Placeholder Syntax**: The RFC uses `...` as a placeholder. In this initial implementation, the piped value is always inserted as the first argument. Full placeholder positioning support can be added later.

3. **Precedence**: The pipe operator should have low precedence (higher than assignment, lower than ternary).

4. **Associativity**: Left-associative, so `a |> b |> c` evaluates as `(a |> b) |> c`.

### Edge Cases

1. **Right Side Must Be Function Call**: The right-hand side must be a function call or method call, not a variable or expression.

2. **Error Messages**: Provide clear error messages when used incorrectly.

3. **Multiple Arguments**: When the right side has arguments, the piped value is inserted first, followed by the specified arguments.

4. **Type Preservation**: The result type of the left side becomes the input to the right side function.

### Interaction with Existing Features

1. **Operator Precedence**: The pipe operator has lower precedence than most operators, so `$x + 5 |> func(...)` pipes `($x + 5)` to `func()`.

2. **Ternary Operator**: `$cond ? $a : $b |> func(...)` should pipe the ternary result to `func()`.

3. **Assignment**: `$result = $x |> func(...)` assigns the final piped result to `$result`.

4. **Method Chaining**: Can be combined with method calls: `$obj->method() |> func(...)`.

### Limitations in Initial Implementation

1. **Placeholder Positioning**: The `...` placeholder is parsed but the piped value is always inserted as the first argument. Future enhancement could support explicit positioning.

2. **Partial Application**: Full partial application semantics are not supported in this initial version.

3. **Type Checking**: No type validation is performed; runtime errors may occur if incompatible types are piped.

## Test Cases

The test suite covers:
- Basic function chaining with `|>` operator
- Functions with additional arguments beyond the piped value
- Multiple transformations chained together
- Precedence interactions with other operators
- Associativity (left-to-right evaluation)
- Using variables in piped function arguments
- Built-in functions with pipe operator
- User-defined functions with pipe operator
- Error case: non-function on right side
- Multi-line pipe chains

## Reference Implementation

Similar patterns in the existing codebase:

| Feature | File | Reference |
|---------|------|-----------|
| Binary operators | `src/ast/ops.rs` | Binary operator enum |
| Operator precedence | `src/parser/precedence.rs` | Precedence levels |
| Binary op parsing | `src/parser/expr.rs` | Infix parsing |
| Binary op evaluation | `src/interpreter/mod.rs` | `eval_binary` method |
| Function call evaluation | `src/interpreter/mod.rs` | `eval_function_call` |
| Null coalescing `??` | Multiple files | Similar low-precedence binary op |

## Implementation Notes

1. **Lexer**: The `|>` token must be recognized as a distinct operator, not confused with bitwise OR `|` (which VHP doesn't support yet).

2. **Parser**: Leverage existing Pratt parser infrastructure for binary operators.

3. **Interpreter**: The pipe operator requires special evaluation logic to insert the left value into the right function call.

4. **Testing**: Focus on various function types (built-in, user-defined) and argument scenarios.

5. **Future Enhancements**:
   - Full placeholder (`...`) positioning support
   - Partial application
   - Pipe with object method chaining
   - Pipe with static method calls
   - Performance optimization for pipe chains

6. **Error Handling**: Provide helpful error messages for common mistakes like piping to non-functions or using unsupported syntax.
