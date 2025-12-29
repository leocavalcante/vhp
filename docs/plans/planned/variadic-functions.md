# Plan: Variadic Functions and Argument Unpacking

## Overview

Variadic functions accept a variable number of arguments using the `...` (splat) operator. The same operator can be used to unpack arrays or traversables when calling functions.

**PHP Example:**
```php
<?php
// Variadic function - collects remaining args into array
function sum(int ...$numbers): int {
    $total = 0;
    foreach ($numbers as $n) {
        $total += $n;
    }
    return $total;
}

echo sum(1, 2, 3);      // 6
echo sum(1, 2, 3, 4, 5); // 15

// Mixed regular and variadic parameters
function greet(string $greeting, string ...$names): string {
    return $greeting . " " . implode(", ", $names);
}

echo greet("Hello", "Alice", "Bob"); // Hello Alice, Bob

// Argument unpacking - spreads array into arguments
$nums = [1, 2, 3];
echo sum(...$nums);  // 6

// Works with named arguments too (PHP 8.0)
function create($name, $age, $city) {
    return "$name, $age, from $city";
}
$data = ['name' => 'Alice', 'age' => 30, 'city' => 'NYC'];
echo create(...$data);
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/token.rs` | Add `Ellipsis` token (...) |
| `src/ast/stmt.rs` | Add `is_variadic` to FunctionParam |
| `src/ast/expr.rs` | Add `Spread` expression |
| `src/parser/stmt/mod.rs` | Parse variadic parameters |
| `src/parser/expr/mod.rs` | Parse argument unpacking |
| `src/interpreter/functions/mod.rs` | Handle variadic args |
| `tests/functions/variadic_*.vhpt` | Test files |

## Implementation Steps

### Step 1: Add Token (`src/token.rs`)

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ... existing tokens ...
    
    /// ... (ellipsis/splat operator)
    Ellipsis,
    
    // ... rest of tokens ...
}
```

### Step 2: Update Lexer (`src/lexer/mod.rs`)

```rust
fn tokenize(&mut self) -> Result<Vec<Token>, String> {
    match c {
        '.' => {
            // Check for ... (ellipsis)
            if self.peek() == '.' && self.peek_next() == '.' {
                self.advance(); // consume second .
                self.advance(); // consume third .
                tokens.push(Token::new(TokenKind::Ellipsis, pos));
            } else {
                // Regular dot (concatenation)
                self.advance();
                tokens.push(Token::new(TokenKind::Dot, pos));
            }
        }
        // ...
    }
}
```

### Step 3: Update FunctionParam (`src/ast/stmt.rs`)

```rust
#[derive(Debug, Clone)]
pub struct FunctionParam {
    pub name: String,
    pub type_hint: Option<TypeHint>,
    pub default: Option<Expr>,
    pub by_ref: bool,
    pub is_variadic: bool,          // NEW: true for ...$param
    pub visibility: Option<Visibility>,
    pub readonly: bool,
    pub attributes: Vec<Attribute>,
}
```

### Step 4: Add Spread Expression (`src/ast/expr.rs`)

```rust
#[derive(Debug, Clone)]
pub enum Expr {
    // ... existing variants ...
    
    /// Spread/unpack expression: ...$array
    Spread(Box<Expr>),
}
```

### Step 5: Parse Variadic Parameters (`src/parser/stmt/mod.rs`)

```rust
fn parse_function_param(&mut self) -> Result<FunctionParam, String> {
    let attributes = self.parse_attributes()?;
    let visibility = self.parse_optional_visibility()?;
    
    let readonly = if self.check(&TokenKind::Readonly) {
        self.advance();
        true
    } else {
        false
    };
    
    // Parse optional type hint
    let type_hint = if self.is_type_start() {
        Some(self.parse_type_hint()?)
    } else {
        None
    };
    
    // Check for variadic: ...
    let is_variadic = if self.check(&TokenKind::Ellipsis) {
        self.advance();
        true
    } else {
        false
    };
    
    // Check for by-reference: &
    let by_ref = if self.check(&TokenKind::Ampersand) {
        self.advance();
        true
    } else {
        false
    };
    
    // Parse variable name
    let name = if let TokenKind::Variable(var_name) = &self.current_token().kind {
        let n = var_name.clone();
        self.advance();
        n
    } else {
        return Err("Expected parameter name".to_string());
    };
    
    // Variadic params cannot have default values
    let default = if self.check(&TokenKind::Assign) {
        if is_variadic {
            return Err("Variadic parameter cannot have a default value".to_string());
        }
        self.advance();
        Some(self.parse_expression()?)
    } else {
        None
    };
    
    Ok(FunctionParam {
        name,
        type_hint,
        default,
        by_ref,
        is_variadic,
        visibility,
        readonly,
        attributes,
    })
}
```

### Step 6: Validate Variadic Position

Variadic parameter must be last:

```rust
fn validate_function_params(params: &[FunctionParam]) -> Result<(), String> {
    let mut found_variadic = false;
    
    for (i, param) in params.iter().enumerate() {
        if found_variadic {
            return Err("Only the last parameter can be variadic".to_string());
        }
        
        if param.is_variadic {
            found_variadic = true;
            
            // Check no required params after variadic
            for later_param in &params[i+1..] {
                if later_param.default.is_none() && !later_param.is_variadic {
                    return Err("Required parameter cannot follow variadic parameter".to_string());
                }
            }
        }
    }
    
    Ok(())
}
```

### Step 7: Parse Argument Unpacking (`src/parser/expr/mod.rs`)

In argument list parsing:

```rust
fn parse_argument_list(&mut self) -> Result<Vec<Expr>, String> {
    let mut args = vec![];
    
    if self.check(&TokenKind::RightParen) {
        return Ok(args);
    }
    
    loop {
        // Check for spread: ...expr
        let arg = if self.check(&TokenKind::Ellipsis) {
            self.advance();
            let expr = self.parse_expression()?;
            Expr::Spread(Box::new(expr))
        } else {
            self.parse_expression()?
        };
        
        args.push(arg);
        
        if !self.check(&TokenKind::Comma) {
            break;
        }
        self.advance();
    }
    
    Ok(args)
}
```

### Step 8: Handle Variadic in Function Calls (`src/interpreter/functions/mod.rs`)

```rust
fn call_function(
    &mut self,
    func: &FunctionDef,
    args: &[Expr],
) -> Result<Value, String> {
    // Evaluate and flatten arguments
    let mut evaluated_args = vec![];
    
    for arg in args {
        match arg {
            Expr::Spread(inner) => {
                // Unpack array into multiple arguments
                let value = self.evaluate(inner)?;
                match value {
                    Value::Array(arr) => {
                        for (_, v) in arr {
                            evaluated_args.push(v);
                        }
                    }
                    _ => return Err("Cannot unpack non-array".to_string()),
                }
            }
            _ => {
                evaluated_args.push(self.evaluate(arg)?);
            }
        }
    }
    
    // Bind parameters
    self.push_scope();
    
    let mut arg_index = 0;
    
    for param in &func.params {
        if param.is_variadic {
            // Collect remaining args into array
            let remaining: Vec<Value> = evaluated_args[arg_index..].to_vec();
            let arr = remaining
                .into_iter()
                .enumerate()
                .map(|(i, v)| (Value::Integer(i as i64), v))
                .collect();
            self.set_variable(&param.name, Value::Array(arr));
            break;
        }
        
        let value = if arg_index < evaluated_args.len() {
            evaluated_args[arg_index].clone()
        } else if let Some(ref default) = param.default {
            self.evaluate(default)?
        } else {
            return Err(format!(
                "Missing required argument ${}",
                param.name
            ));
        };
        
        // Type check if needed
        if let Some(ref type_hint) = param.type_hint {
            self.validate_type(&value, type_hint, &func.name, arg_index)?;
        }
        
        self.set_variable(&param.name, value);
        arg_index += 1;
    }
    
    // Execute function body
    let result = self.execute_statements(&func.body)?;
    
    self.pop_scope();
    
    match result {
        ControlFlow::Return(v) => Ok(v),
        _ => Ok(Value::Null),
    }
}
```

### Step 9: Add Tests

**tests/functions/variadic_basic.vhpt**
```
--TEST--
Basic variadic function
--FILE--
<?php
function sum(...$numbers) {
    $total = 0;
    foreach ($numbers as $n) {
        $total += $n;
    }
    return $total;
}

echo sum(1, 2, 3);
--EXPECT--
6
```

**tests/functions/variadic_empty.vhpt**
```
--TEST--
Variadic function with no arguments
--FILE--
<?php
function collect(...$items) {
    return count($items);
}

echo collect();
--EXPECT--
0
```

**tests/functions/variadic_mixed.vhpt**
```
--TEST--
Mixed regular and variadic parameters
--FILE--
<?php
function greet($greeting, ...$names) {
    return $greeting . " " . implode(", ", $names);
}

echo greet("Hello", "Alice", "Bob", "Carol");
--EXPECT--
Hello Alice, Bob, Carol
```

**tests/functions/variadic_typed.vhpt**
```
--TEST--
Variadic with type hint
--FILE--
<?php
function sumInts(int ...$numbers): int {
    $total = 0;
    foreach ($numbers as $n) {
        $total += $n;
    }
    return $total;
}

echo sumInts(10, 20, 30);
--EXPECT--
60
```

**tests/functions/spread_basic.vhpt**
```
--TEST--
Spread operator to unpack array
--FILE--
<?php
function add($a, $b, $c) {
    return $a + $b + $c;
}

$nums = [1, 2, 3];
echo add(...$nums);
--EXPECT--
6
```

**tests/functions/spread_with_regular.vhpt**
```
--TEST--
Spread mixed with regular arguments
--FILE--
<?php
function concat($a, $b, $c, $d) {
    return $a . $b . $c . $d;
}

$arr = ["c", "d"];
echo concat("a", "b", ...$arr);
--EXPECT--
abcd
```

**tests/functions/spread_multiple.vhpt**
```
--TEST--
Multiple spread operators
--FILE--
<?php
function sum(...$nums) {
    $t = 0;
    foreach ($nums as $n) $t += $n;
    return $t;
}

$a = [1, 2];
$b = [3, 4];
echo sum(...$a, ...$b);
--EXPECT--
10
```

**tests/functions/variadic_position_error.vhpt**
```
--TEST--
Variadic parameter must be last
--FILE--
<?php
function bad(...$a, $b) {}
--EXPECT_ERROR--
Only the last parameter can be variadic
```

**tests/functions/variadic_no_default.vhpt**
```
--TEST--
Variadic cannot have default
--FILE--
<?php
function bad(...$a = []) {}
--EXPECT_ERROR--
Variadic parameter cannot have a default value
```

**tests/functions/spread_assoc_array.vhpt**
```
--TEST--
Spread with associative array (named args)
--FILE--
<?php
function create($name, $age) {
    return "$name is $age";
}

$data = ["name" => "Alice", "age" => 30];
echo create(...$data);
--EXPECT--
Alice is 30
```

**tests/functions/variadic_by_ref.vhpt**
```
--TEST--
Variadic with by-reference
--FILE--
<?php
function doubleAll(&...$nums) {
    foreach ($nums as &$n) {
        $n *= 2;
    }
}

$a = 1;
$b = 2;
$c = 3;
doubleAll($a, $b, $c);
echo "$a $b $c";
--EXPECT--
2 4 6
```

## PHP Compatibility Notes

| Feature | PHP Version |
|---------|-------------|
| Variadic parameters | 5.6 |
| Argument unpacking | 5.6 |
| Type hints on variadic | 7.0 |
| Named argument unpacking | 8.0 |

## Key Rules

1. **Position**: Variadic parameter must be last
2. **No default**: Variadic cannot have default value
3. **Type applies**: Type hint applies to each collected argument
4. **Empty is valid**: No arguments = empty array
5. **Unpack order**: Spread args are flattened left-to-right
6. **Assoc unpack**: Associative arrays map to named parameters (PHP 8.0)

## Implementation Order

1. Token (Ellipsis)
2. Parse variadic in parameters
3. Basic variadic function calls
4. Parse spread in arguments
5. Argument unpacking
6. Type hints on variadic
7. By-reference variadic
8. Named argument unpacking

## Error Messages

- `Only the last parameter can be variadic`
- `Variadic parameter cannot have a default value`
- `Cannot unpack non-array`
- `Cannot use positional argument after named argument unpacking`
