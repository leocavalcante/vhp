# Plan: Arrow Functions (Short Closures)

## Overview

Arrow functions provide a shorter syntax for anonymous functions that automatically capture variables from the enclosing scope by value. Introduced in PHP 7.4.

**PHP Example:**
```php
<?php
// Traditional closure
$double = function($n) {
    return $n * 2;
};

// Arrow function (same behavior)
$double = fn($n) => $n * 2;

// Auto-captures from outer scope
$multiplier = 3;
$multiply = fn($n) => $n * $multiplier;
echo $multiply(4); // 12

// Can be used inline
$numbers = [1, 2, 3, 4];
$doubled = array_map(fn($n) => $n * 2, $numbers);

// With type hints
$toInt = fn(string $s): int => intval($s);
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/token.rs` | Add `Fn` token |
| `src/ast/expr.rs` | Add `ArrowFunction` expression |
| `src/parser/expr/mod.rs` | Parse arrow functions |
| `src/interpreter/expr_eval/mod.rs` | Evaluate arrow functions |
| `src/interpreter/value.rs` | Handle arrow function as callable |
| `tests/functions/arrow_*.vhpt` | Test files |

## Implementation Steps

### Step 1: Add Token (`src/token.rs`)

Add the `fn` keyword token:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ... existing tokens ...
    
    // Keywords
    Fn,  // fn (arrow function)
    
    // ... rest of tokens ...
}
```

### Step 2: Update Lexer (`src/lexer/mod.rs`)

Add `fn` to keyword recognition:

```rust
fn tokenize_identifier(&mut self) -> TokenKind {
    // ... collect identifier characters ...
    
    match ident.to_lowercase().as_str() {
        // ... existing keywords ...
        "fn" => TokenKind::Fn,
        // ...
    }
}
```

### Step 3: Add Arrow Function to AST (`src/ast/expr.rs`)

Add the `ArrowFunction` variant:

```rust
#[derive(Debug, Clone)]
pub enum Expr {
    // ... existing variants ...
    
    /// Arrow function: fn($params) => expr
    ArrowFunction {
        params: Vec<FunctionParam>,
        return_type: Option<TypeHint>,
        body: Box<Expr>,  // Single expression (not statement block)
    },
    
    // ... rest of variants ...
}
```

### Step 4: Parse Arrow Functions (`src/parser/expr/mod.rs`)

Add parsing for arrow functions:

```rust
/// Parse arrow function: fn(params) => expression
fn parse_arrow_function(&mut self) -> Result<Expr, String> {
    // Already consumed 'fn' token
    
    // Parse parameter list
    self.expect(&TokenKind::LeftParen)?;
    let params = self.parse_function_params()?;
    self.expect(&TokenKind::RightParen)?;
    
    // Parse optional return type
    let return_type = if self.check(&TokenKind::Colon) {
        self.advance();
        Some(self.parse_type_hint()?)
    } else {
        None
    };
    
    // Expect => (fat arrow)
    self.expect(&TokenKind::DoubleArrow)?;
    
    // Parse the expression body (NOT a statement block)
    let body = self.parse_expression()?;
    
    Ok(Expr::ArrowFunction {
        params,
        return_type,
        body: Box::new(body),
    })
}
```

Integrate into primary expression parsing:

```rust
fn parse_primary(&mut self) -> Result<Expr, String> {
    match &self.current_token().kind {
        // ... existing cases ...
        
        TokenKind::Fn => {
            self.advance(); // consume 'fn'
            self.parse_arrow_function()
        }
        
        // ... rest of cases ...
    }
}
```

### Step 5: Add Callable Value Type (`src/interpreter/value.rs`)

If not already present, add a way to represent callable values:

```rust
/// Represents a closure/callable
#[derive(Debug, Clone)]
pub struct Closure {
    pub params: Vec<FunctionParam>,
    pub return_type: Option<TypeHint>,
    pub body: ClosureBody,
    pub captured_vars: HashMap<String, Value>,  // Auto-captured from scope
}

#[derive(Debug, Clone)]
pub enum ClosureBody {
    Block(Vec<Stmt>),
    Expression(Expr),  // For arrow functions
}
```

Update `Value` enum:

```rust
#[derive(Debug, Clone)]
pub enum Value {
    // ... existing variants ...
    Closure(Rc<Closure>),  // or Box<Closure>
}
```

### Step 6: Evaluate Arrow Functions (`src/interpreter/expr_eval/mod.rs`)

When evaluating an arrow function, capture variables from current scope:

```rust
fn eval_arrow_function(
    &mut self,
    params: &[FunctionParam],
    return_type: &Option<TypeHint>,
    body: &Expr,
) -> Result<Value, String> {
    // Capture ALL variables from current scope by value
    // This is the key difference from regular closures
    let captured_vars = self.capture_scope_variables();
    
    let closure = Closure {
        params: params.to_vec(),
        return_type: return_type.clone(),
        body: ClosureBody::Expression(body.clone()),
        captured_vars,
    };
    
    Ok(Value::Closure(Rc::new(closure)))
}

/// Capture all variables from current scope by value
fn capture_scope_variables(&self) -> HashMap<String, Value> {
    let mut captured = HashMap::new();
    
    // Iterate through all variable scopes from innermost to outermost
    for scope in self.scopes.iter().rev() {
        for (name, value) in scope {
            if !captured.contains_key(name) {
                captured.insert(name.clone(), value.clone());
            }
        }
    }
    
    captured
}
```

### Step 7: Call Arrow Functions

When calling a closure, set up the captured environment:

```rust
fn call_closure(
    &mut self,
    closure: &Closure,
    args: Vec<Value>,
) -> Result<Value, String> {
    // Create new scope
    self.push_scope();
    
    // Add captured variables to scope
    for (name, value) in &closure.captured_vars {
        self.set_variable(name, value.clone());
    }
    
    // Bind parameters
    for (i, param) in closure.params.iter().enumerate() {
        let value = if i < args.len() {
            args[i].clone()
        } else if let Some(ref default) = param.default {
            self.evaluate(default)?
        } else {
            return Err(format!("Missing argument {}", i + 1));
        };
        
        // Type check if type hint present
        if let Some(ref type_hint) = param.type_hint {
            if !value.matches_type(type_hint) {
                return Err(format!(
                    "Argument {} must be of type {}, {} given",
                    i + 1,
                    self.format_type_hint(type_hint),
                    value.type_name()
                ));
            }
        }
        
        self.set_variable(&param.name, value);
    }
    
    // Execute body
    let result = match &closure.body {
        ClosureBody::Expression(expr) => self.evaluate(expr)?,
        ClosureBody::Block(stmts) => {
            for stmt in stmts {
                if let ControlFlow::Return(val) = self.execute_stmt(stmt)? {
                    return Ok(val);
                }
            }
            Value::Null
        }
    };
    
    // Validate return type
    if let Some(ref return_type) = closure.return_type {
        if !result.matches_type(return_type) {
            return Err(format!(
                "Return value must be of type {}, {} returned",
                self.format_type_hint(return_type),
                result.type_name()
            ));
        }
    }
    
    self.pop_scope();
    Ok(result)
}
```

### Step 8: Support Callable in Function Calls

Arrow functions can be stored in variables and called:

```rust
fn call_expression(&mut self, callee: &Expr, args: &[Expr]) -> Result<Value, String> {
    let callee_value = self.evaluate(callee)?;
    
    match callee_value {
        Value::Closure(closure) => {
            let arg_values = args.iter()
                .map(|a| self.evaluate(a))
                .collect::<Result<Vec<_>, _>>()?;
            self.call_closure(&closure, arg_values)
        }
        Value::String(func_name) => {
            // Call by function name
            self.call_function(&func_name, args)
        }
        _ => Err(format!("Value is not callable: {:?}", callee_value)),
    }
}
```

Also update function call syntax to handle variable function calls:

```rust
// Handle $func() syntax
Expr::Variable(name) => {
    let value = self.get_variable(name)?;
    match value {
        Value::Closure(closure) => {
            // Call the closure
            let arg_values = args.iter()
                .map(|a| self.evaluate(a))
                .collect::<Result<Vec<_>, _>>()?;
            self.call_closure(&closure, arg_values)
        }
        Value::String(func_name) => {
            // Variable function call
            self.call_function(&func_name, args)
        }
        _ => Err(format!("{} is not callable", name)),
    }
}
```

### Step 9: Add Tests

**tests/functions/arrow_basic.vhpt**
```
--TEST--
Basic arrow function
--FILE--
<?php
$double = fn($n) => $n * 2;
echo $double(5);
--EXPECT--
10
```

**tests/functions/arrow_capture.vhpt**
```
--TEST--
Arrow function captures variables from outer scope
--FILE--
<?php
$multiplier = 3;
$multiply = fn($n) => $n * $multiplier;
echo $multiply(4);
--EXPECT--
12
```

**tests/functions/arrow_capture_by_value.vhpt**
```
--TEST--
Arrow function captures by value not reference
--FILE--
<?php
$x = 10;
$getX = fn() => $x;
$x = 20;
echo $getX();
--EXPECT--
10
```

**tests/functions/arrow_with_types.vhpt**
```
--TEST--
Arrow function with type hints
--FILE--
<?php
$toUpper = fn(string $s): string => strtoupper($s);
echo $toUpper("hello");
--EXPECT--
HELLO
```

**tests/functions/arrow_multiline_expr.vhpt**
```
--TEST--
Arrow function with complex expression
--FILE--
<?php
$process = fn($a, $b) => $a > $b 
    ? $a - $b 
    : $b - $a;
echo $process(3, 7) . "\n";
echo $process(7, 3);
--EXPECT--
4
4
```

**tests/functions/arrow_nested.vhpt**
```
--TEST--
Nested arrow functions
--FILE--
<?php
$outer = 5;
$f = fn($x) => fn($y) => $x + $y + $outer;
$g = $f(10);
echo $g(3);
--EXPECT--
18
```

**tests/functions/arrow_with_array_functions.vhpt**
```
--TEST--
Arrow function with array_map (when implemented)
--SKIPIF--
array_map not yet implemented
--FILE--
<?php
$numbers = [1, 2, 3];
$doubled = array_map(fn($n) => $n * 2, $numbers);
echo implode(", ", $doubled);
--EXPECT--
2, 4, 6
```

**tests/functions/arrow_no_params.vhpt**
```
--TEST--
Arrow function with no parameters
--FILE--
<?php
$now = fn() => 42;
echo $now();
--EXPECT--
42
```

**tests/functions/arrow_default_params.vhpt**
```
--TEST--
Arrow function with default parameter
--FILE--
<?php
$greet = fn($name = "World") => "Hello, $name";
echo $greet() . "\n";
echo $greet("PHP");
--EXPECT--
Hello, World
Hello, PHP
```

## Key Differences from Regular Closures

| Feature | Regular Closure | Arrow Function |
|---------|-----------------|----------------|
| Syntax | `function($x) { return $x; }` | `fn($x) => $x` |
| Capture | Explicit with `use` | Automatic by value |
| Body | Statement block | Single expression |
| Return | Explicit `return` | Implicit return |

## PHP Compatibility Notes

- Arrow functions were added in PHP 7.4
- They can only contain a single expression (no statements)
- Variables are captured by VALUE at definition time
- Cannot use `&` for by-reference capture (unlike `use (&$var)`)
- Type hints work the same as regular functions

## Implementation Order

1. Token and lexer first
2. AST structure
3. Basic parsing without types
4. Expression evaluation
5. Variable capture
6. Type hints support
7. Integration with callable type

## Testing Priority

1. Basic arrow functions
2. Variable capture
3. Nested arrow functions
4. Type hints
5. Edge cases (no params, defaults)
