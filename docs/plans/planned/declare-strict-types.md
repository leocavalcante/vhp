# Plan: Declare Directive (strict_types)

## Overview

The `declare` directive controls certain execution aspects of a script. The most important use is `declare(strict_types=1)` which enables strict type checking for function calls in that file.

**PHP Example:**
```php
<?php
declare(strict_types=1);

function add(int $a, int $b): int {
    return $a + $b;
}

echo add(1, 2);      // OK: 3
echo add(1.5, 2);    // Error: must be of type int, float given
echo add("1", 2);    // Error: must be of type int, string given

// Without strict_types (default behavior):
// add(1.5, 2) would work, converting 1.5 to 1
// add("1", 2) would work, converting "1" to 1
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/token.rs` | Add `Declare` token |
| `src/ast/stmt.rs` | Add `Declare` statement |
| `src/parser/stmt/mod.rs` | Parse declare directive |
| `src/interpreter/mod.rs` | Track strict_types mode, modify type coercion |
| `tests/types/strict_*.vhpt` | Test files |

## Implementation Steps

### Step 1: Add Token (`src/token.rs`)

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ... existing tokens ...
    
    Declare,
    
    // ... rest of tokens ...
}
```

### Step 2: Update Lexer (`src/lexer/mod.rs`)

```rust
fn tokenize_identifier(&mut self) -> TokenKind {
    match ident.to_lowercase().as_str() {
        // ... existing keywords ...
        "declare" => TokenKind::Declare,
        // ...
    }
}
```

### Step 3: Add Declare Statement (`src/ast/stmt.rs`)

```rust
/// Declare directive type
#[derive(Debug, Clone)]
pub enum DeclareDirective {
    /// strict_types=0 or strict_types=1
    StrictTypes(bool),
    /// encoding='UTF-8'
    Encoding(String),
    /// ticks=N
    Ticks(i64),
}

// Add to Stmt enum
pub enum Stmt {
    // ... existing variants ...
    
    /// declare(directive) or declare(directive) { ... }
    Declare {
        directives: Vec<DeclareDirective>,
        body: Option<Vec<Stmt>>,  // None for file-scope
    },
}
```

### Step 4: Parse Declare Statement (`src/parser/stmt/mod.rs`)

```rust
fn parse_declare(&mut self) -> Result<Stmt, String> {
    self.expect(&TokenKind::Declare)?;
    self.expect(&TokenKind::LeftParen)?;
    
    let mut directives = vec![];
    
    loop {
        let name = self.expect_identifier()?;
        self.expect(&TokenKind::Assign)?;
        
        let directive = match name.to_lowercase().as_str() {
            "strict_types" => {
                let value = self.parse_expression()?;
                match value {
                    Expr::Integer(0) => DeclareDirective::StrictTypes(false),
                    Expr::Integer(1) => DeclareDirective::StrictTypes(true),
                    _ => return Err("strict_types value must be 0 or 1".to_string()),
                }
            }
            "encoding" => {
                let value = self.parse_expression()?;
                match value {
                    Expr::String(s) => DeclareDirective::Encoding(s),
                    _ => return Err("encoding value must be a string".to_string()),
                }
            }
            "ticks" => {
                let value = self.parse_expression()?;
                match value {
                    Expr::Integer(n) => DeclareDirective::Ticks(n),
                    _ => return Err("ticks value must be an integer".to_string()),
                }
            }
            _ => return Err(format!("Unknown declare directive: {}", name)),
        };
        
        directives.push(directive);
        
        if !self.check(&TokenKind::Comma) {
            break;
        }
        self.advance();
    }
    
    self.expect(&TokenKind::RightParen)?;
    
    // Check for block syntax: declare(...) { ... }
    let body = if self.check(&TokenKind::LeftBrace) {
        self.advance();
        let stmts = self.parse_block()?;
        self.expect(&TokenKind::RightBrace)?;
        Some(stmts)
    } else if self.check(&TokenKind::Colon) {
        // Alternative syntax: declare(...): ... enddeclare;
        self.advance();
        let stmts = self.parse_statements_until(&TokenKind::Identifier("enddeclare".to_string()))?;
        self.expect_identifier_eq("enddeclare")?;
        self.expect(&TokenKind::Semicolon)?;
        Some(stmts)
    } else {
        // File-scope: declare(...);
        self.expect(&TokenKind::Semicolon)?;
        None
    };
    
    Ok(Stmt::Declare { directives, body })
}
```

### Step 5: Track Strict Types Mode (`src/interpreter/mod.rs`)

```rust
pub struct Interpreter {
    // ... existing fields ...
    
    /// strict_types mode for current file/scope
    strict_types: bool,
    
    /// Stack of strict_types states for nested scopes
    strict_types_stack: Vec<bool>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            // ...
            strict_types: false,
            strict_types_stack: vec![],
        }
    }
    
    fn execute_declare(&mut self, directives: &[DeclareDirective], body: &Option<Vec<Stmt>>) -> Result<ControlFlow, String> {
        // Process directives
        for directive in directives {
            match directive {
                DeclareDirective::StrictTypes(enabled) => {
                    if body.is_some() {
                        // Block-scope strict_types
                        self.strict_types_stack.push(self.strict_types);
                        self.strict_types = *enabled;
                    } else {
                        // File-scope strict_types (must be first statement)
                        self.strict_types = *enabled;
                    }
                }
                DeclareDirective::Encoding(_) => {
                    // Encoding is mostly ignored in modern PHP
                }
                DeclareDirective::Ticks(_) => {
                    // Ticks for register_tick_function (advanced)
                }
            }
        }
        
        // Execute body if present
        if let Some(stmts) = body {
            let result = self.execute_statements(stmts)?;
            
            // Restore strict_types after block
            if !self.strict_types_stack.is_empty() {
                self.strict_types = self.strict_types_stack.pop().unwrap();
            }
            
            return Ok(result);
        }
        
        Ok(ControlFlow::Normal)
    }
}
```

### Step 6: Modify Type Coercion Based on Strict Mode

Update argument validation:

```rust
fn validate_argument(
    &self,
    param: &FunctionParam,
    value: &Value,
    func_name: &str,
    arg_position: usize,
) -> Result<Value, String> {
    let Some(ref type_hint) = param.type_hint else {
        return Ok(value.clone());
    };
    
    if self.strict_types {
        // Strict mode: exact type match required (with limited exceptions)
        if !value.matches_type_strict(type_hint) {
            return Err(format!(
                "{}(): Argument #{} must be of type {}, {} given",
                func_name,
                arg_position + 1,
                self.format_type_hint(type_hint),
                value.type_name()
            ));
        }
        Ok(value.clone())
    } else {
        // Coercive mode: attempt to coerce value
        match self.coerce_to_type(value, type_hint) {
            Ok(coerced) => Ok(coerced),
            Err(_) => Err(format!(
                "{}(): Argument #{} must be of type {}, {} given",
                func_name,
                arg_position + 1,
                self.format_type_hint(type_hint),
                value.type_name()
            )),
        }
    }
}
```

### Step 7: Implement Type Coercion (`src/interpreter/value.rs`)

```rust
impl Value {
    /// Strict type matching (no coercion)
    pub fn matches_type_strict(&self, type_hint: &TypeHint) -> bool {
        match type_hint {
            TypeHint::Simple(name) => match (name.as_str(), self) {
                ("int", Value::Integer(_)) => true,
                ("float", Value::Float(_)) => true,
                ("float", Value::Integer(_)) => true, // int is subtype of float
                ("string", Value::String(_)) => true,
                ("bool", Value::Bool(_)) => true,
                ("array", Value::Array(_)) => true,
                ("null", Value::Null) => true,
                ("mixed", _) => true,
                _ => false,
            },
            TypeHint::Nullable(inner) => {
                matches!(self, Value::Null) || self.matches_type_strict(inner)
            }
            TypeHint::Union(types) => {
                types.iter().any(|t| self.matches_type_strict(t))
            }
            _ => self.matches_type(type_hint), // Fall back to loose matching
        }
    }
}

impl Interpreter {
    /// Attempt to coerce value to type (for non-strict mode)
    fn coerce_to_type(&self, value: &Value, type_hint: &TypeHint) -> Result<Value, String> {
        match type_hint {
            TypeHint::Simple(name) => match name.as_str() {
                "int" => match value {
                    Value::Integer(_) => Ok(value.clone()),
                    Value::Float(f) => Ok(Value::Integer(*f as i64)),
                    Value::String(s) => s.parse::<i64>()
                        .map(Value::Integer)
                        .map_err(|_| "Cannot coerce to int".to_string()),
                    Value::Bool(true) => Ok(Value::Integer(1)),
                    Value::Bool(false) => Ok(Value::Integer(0)),
                    _ => Err("Cannot coerce to int".to_string()),
                },
                "float" => match value {
                    Value::Float(_) => Ok(value.clone()),
                    Value::Integer(i) => Ok(Value::Float(*i as f64)),
                    Value::String(s) => s.parse::<f64>()
                        .map(Value::Float)
                        .map_err(|_| "Cannot coerce to float".to_string()),
                    _ => Err("Cannot coerce to float".to_string()),
                },
                "string" => match value {
                    Value::String(_) => Ok(value.clone()),
                    Value::Integer(i) => Ok(Value::String(i.to_string())),
                    Value::Float(f) => Ok(Value::String(f.to_string())),
                    Value::Bool(true) => Ok(Value::String("1".to_string())),
                    Value::Bool(false) => Ok(Value::String("".to_string())),
                    _ => Err("Cannot coerce to string".to_string()),
                },
                "bool" => Ok(Value::Bool(value.to_bool())),
                _ => Err(format!("Cannot coerce to {}", name)),
            },
            TypeHint::Nullable(inner) => {
                if matches!(value, Value::Null) {
                    Ok(Value::Null)
                } else {
                    self.coerce_to_type(value, inner)
                }
            }
            _ => Err("Cannot coerce to this type".to_string()),
        }
    }
}
```

### Step 8: Validate Declare Position

`declare(strict_types=1)` must be the very first statement:

```rust
fn validate_declare_position(&self, program: &Program) -> Result<(), String> {
    let mut found_non_declare = false;
    
    for stmt in &program.statements {
        match stmt {
            Stmt::Declare { directives, body: None } => {
                // Check if strict_types and not first statement
                for dir in directives {
                    if matches!(dir, DeclareDirective::StrictTypes(_)) && found_non_declare {
                        return Err(
                            "strict_types declaration must be the very first statement".to_string()
                        );
                    }
                }
            }
            Stmt::Namespace { .. } => {
                // Namespace can come before strict_types
            }
            _ => {
                found_non_declare = true;
            }
        }
    }
    
    Ok(())
}
```

### Step 9: Add Tests

**tests/types/strict_types_enabled.vhpt**
```
--TEST--
strict_types=1 requires exact types
--FILE--
<?php
declare(strict_types=1);

function add(int $a, int $b): int {
    return $a + $b;
}

echo add(1, 2);
--EXPECT--
3
```

**tests/types/strict_types_error.vhpt**
```
--TEST--
strict_types=1 rejects float for int
--FILE--
<?php
declare(strict_types=1);

function requireInt(int $n): void {
    echo $n;
}

requireInt(1.5);
--EXPECT_ERROR--
must be of type int, float given
```

**tests/types/strict_types_string_error.vhpt**
```
--TEST--
strict_types=1 rejects string for int
--FILE--
<?php
declare(strict_types=1);

function requireInt(int $n): void {
    echo $n;
}

requireInt("42");
--EXPECT_ERROR--
must be of type int, string given
```

**tests/types/coercive_mode.vhpt**
```
--TEST--
Without strict_types, values are coerced
--FILE--
<?php
function add(int $a, int $b): int {
    return $a + $b;
}

echo add("1", "2") . "\n";
echo add(1.9, 2.1);
--EXPECT--
3
3
```

**tests/types/strict_int_to_float.vhpt**
```
--TEST--
strict_types allows int for float (widening)
--FILE--
<?php
declare(strict_types=1);

function half(float $n): float {
    return $n / 2;
}

echo half(10);
--EXPECT--
5
```

**tests/types/strict_return_type.vhpt**
```
--TEST--
strict_types affects return type too
--FILE--
<?php
declare(strict_types=1);

function getString(): string {
    return "hello";
}

echo getString();
--EXPECT--
hello
```

**tests/types/declare_block_scope.vhpt**
```
--TEST--
declare with block scope
--FILE--
<?php
function outer(int $n) {
    return $n;
}

declare(strict_types=1) {
    function inner(int $n) {
        return $n;
    }
    // This would error: inner("1");
}

// Outside block, coercion works
echo outer("42");
--EXPECT--
42
```

**tests/types/strict_types_position_error.vhpt**
```
--TEST--
strict_types must be first statement
--FILE--
<?php
echo "hello";
declare(strict_types=1);
--EXPECT_ERROR--
strict_types declaration must be the very first statement
```

## PHP Compatibility Notes

| Feature | PHP Version |
|---------|-------------|
| `declare(strict_types=1)` | 7.0 |
| Block-scoped declare | 7.0 |
| Alternative syntax `declare(): enddeclare;` | 7.0 |

## Key Rules

1. **File scope**: `declare(strict_types=1);` affects entire file
2. **Caller determines**: Strict mode is determined by the calling file, not the function definition file
3. **Position**: Must be first statement in file (before anything except comments)
4. **Widening allowed**: `int` can be passed to `float` even in strict mode
5. **No narrowing**: `float` cannot be passed to `int` in strict mode

## Implementation Order

1. Token and parsing
2. Declare statement execution
3. Strict mode tracking
4. Modify type validation
5. Type coercion for non-strict mode
6. Position validation
7. Block scope support

## Error Messages

- `strict_types declaration must be the very first statement in the script`
- `Argument #X must be of type X, Y given`
- `Return value must be of type X, Y returned`
