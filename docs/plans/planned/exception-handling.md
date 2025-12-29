# Plan: Exception Handling (try/catch/finally/throw)

## Overview

Exception handling is fundamental to PHP and required for proper error management. This feature adds:
- `try`/`catch`/`finally` statements
- `throw` keyword (both statement and expression in PHP 8.0+)
- Base `Exception` class
- Multiple catch blocks
- Multi-catch syntax (PHP 7.1+): `catch (TypeA | TypeB $e)`

**PHP Example:**
```php
<?php
// Basic try/catch
try {
    throw new Exception("Something went wrong");
} catch (Exception $e) {
    echo $e->getMessage();
}

// With finally
try {
    echo "trying\n";
} catch (Exception $e) {
    echo "caught\n";
} finally {
    echo "finally\n";
}

// Multi-catch (PHP 7.1+)
try {
    // ...
} catch (InvalidArgumentException | RuntimeException $e) {
    echo $e->getMessage();
}

// Throw expression (PHP 8.0+)
$value = $nullableValue ?? throw new Exception("Value required");
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/token.rs` | Add `Try`, `Catch`, `Finally`, `Throw` tokens |
| `src/lexer/mod.rs` | Recognize new keywords |
| `src/ast/stmt.rs` | Add `TryCatch` statement, `Throw` statement |
| `src/ast/expr.rs` | Add `Throw` expression (PHP 8.0) |
| `src/parser/stmt/mod.rs` | Parse try/catch/finally statements |
| `src/parser/expr/mod.rs` | Parse throw as expression |
| `src/interpreter/mod.rs` | Add `Exception` type to runtime |
| `src/interpreter/value.rs` | Add `Exception` value variant |
| `src/interpreter/stmt_exec/mod.rs` | Execute try/catch/finally |
| `tests/exceptions/*.vhpt` | Test files |

## Implementation Steps

### Step 1: Add Tokens (`src/token.rs`)

Add these tokens to the `TokenKind` enum after the existing keywords section (around line 30):

```rust
    // Exception Keywords
    Try,      // try
    Catch,    // catch
    Finally,  // finally
    Throw,    // throw
```

### Step 2: Update Lexer (`src/lexer/mod.rs`)

Find the keyword matching section (where identifiers are converted to keywords) and add:

```rust
"try" => TokenKind::Try,
"catch" => TokenKind::Catch,
"finally" => TokenKind::Finally,
"throw" => TokenKind::Throw,
```

### Step 3: Extend AST (`src/ast/stmt.rs`)

Add a new struct for catch clauses and update the `Stmt` enum:

```rust
/// Catch clause for try statement
#[derive(Debug, Clone)]
pub struct CatchClause {
    /// Exception types to catch (supports multi-catch with |)
    pub exception_types: Vec<String>,
    /// Variable name to bind exception (e.g., $e)
    pub variable: String,
    /// Body of catch block
    pub body: Vec<Stmt>,
}

// Add to Stmt enum:
    /// Try/Catch/Finally statement
    TryCatch {
        try_body: Vec<Stmt>,
        catch_clauses: Vec<CatchClause>,
        finally_body: Option<Vec<Stmt>>,
    },
    
    /// Throw statement
    Throw(Expr),
```

### Step 4: Extend AST (`src/ast/expr.rs`)

Add throw as an expression (PHP 8.0+) to the `Expr` enum:

```rust
    /// Throw expression (PHP 8.0+)
    /// Used in: $x ?? throw new Exception("..."), fn() => throw new Exception()
    ThrowExpr(Box<Expr>),
```

### Step 5: Update Value Type (`src/interpreter/value.rs`)

Add an Exception variant to the `Value` enum:

```rust
/// Exception value
#[derive(Debug, Clone)]
pub struct ExceptionValue {
    pub class_name: String,
    pub message: String,
    pub code: i64,
    pub file: String,
    pub line: usize,
    pub previous: Option<Box<ExceptionValue>>,
}

// Add to Value enum:
    Exception(ExceptionValue),
```

### Step 6: Update Parser (`src/parser/stmt/mod.rs`)

Add parsing for try/catch/finally. Create a new function:

```rust
/// Parse try/catch/finally statement
/// try { ... } catch (ExceptionType $e) { ... } finally { ... }
fn parse_try(&mut self) -> Result<Stmt, String> {
    self.advance(); // consume 'try'
    
    // Parse try block
    self.expect(TokenKind::LeftBrace)?;
    let try_body = self.parse_block()?;
    
    let mut catch_clauses = Vec::new();
    let mut finally_body = None;
    
    // Parse catch clauses
    while self.check(&TokenKind::Catch) {
        self.advance(); // consume 'catch'
        self.expect(TokenKind::LeftParen)?;
        
        // Parse exception types (supports Type1 | Type2)
        let mut exception_types = Vec::new();
        loop {
            if let TokenKind::Identifier(name) = &self.current_token().kind {
                exception_types.push(name.clone());
                self.advance();
            } else {
                return Err("Expected exception type".to_string());
            }
            
            // Check for multi-catch separator |
            if self.check(&TokenKind::Or) || self.check_char('|') {
                self.advance();
            } else {
                break;
            }
        }
        
        // Parse variable name
        let variable = if let TokenKind::Variable(name) = &self.current_token().kind {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err("Expected exception variable".to_string());
        };
        
        self.expect(TokenKind::RightParen)?;
        self.expect(TokenKind::LeftBrace)?;
        let catch_body = self.parse_block()?;
        
        catch_clauses.push(CatchClause {
            exception_types,
            variable,
            body: catch_body,
        });
    }
    
    // Parse optional finally
    if self.check(&TokenKind::Finally) {
        self.advance(); // consume 'finally'
        self.expect(TokenKind::LeftBrace)?;
        finally_body = Some(self.parse_block()?);
    }
    
    // Must have at least one catch or finally
    if catch_clauses.is_empty() && finally_body.is_none() {
        return Err("Try must have at least one catch or finally block".to_string());
    }
    
    Ok(Stmt::TryCatch {
        try_body,
        catch_clauses,
        finally_body,
    })
}
```

In `parse_statement()`, add the case:

```rust
TokenKind::Try => self.parse_try(),
TokenKind::Throw => self.parse_throw_statement(),
```

Add throw statement parsing:

```rust
fn parse_throw_statement(&mut self) -> Result<Stmt, String> {
    self.advance(); // consume 'throw'
    let expr = self.parse_expression()?;
    self.expect_semicolon()?;
    Ok(Stmt::Throw(expr))
}
```

### Step 7: Update Parser for Throw Expression (`src/parser/expr/mod.rs`)

Allow `throw` as an expression in places like null coalesce and ternary. In the expression parser, handle `throw` as a primary expression that can appear in expression context:

```rust
// In parse_primary or similar
TokenKind::Throw => {
    self.advance(); // consume 'throw'
    let expr = self.parse_expression()?;
    Ok(Expr::ThrowExpr(Box::new(expr)))
}
```

### Step 8: Create Exception Handling in Interpreter

In `src/interpreter/mod.rs`, add a custom result type for exception propagation:

```rust
/// Result type that can propagate exceptions
pub enum ExecutionResult {
    Ok,
    Return(Value),
    Break,
    Continue,
    Exception(ExceptionValue),
}
```

### Step 9: Update Statement Execution (`src/interpreter/stmt_exec/mod.rs`)

Add execution logic for try/catch/finally:

```rust
Stmt::TryCatch { try_body, catch_clauses, finally_body } => {
    // Execute try block
    let try_result = self.execute_block(try_body);
    
    let mut caught = false;
    let mut result = ExecutionResult::Ok;
    
    // Check if exception was thrown
    if let ExecutionResult::Exception(ref exception) = try_result {
        // Find matching catch clause
        for catch_clause in catch_clauses {
            let matches = catch_clause.exception_types.iter()
                .any(|t| t == &exception.class_name || t == "Exception");
            
            if matches {
                // Bind exception to variable
                self.set_variable(&catch_clause.variable, Value::Exception(exception.clone()));
                result = self.execute_block(&catch_clause.body);
                caught = true;
                break;
            }
        }
        
        if !caught {
            result = try_result; // Re-propagate uncaught exception
        }
    } else {
        result = try_result;
    }
    
    // Always execute finally
    if let Some(finally_stmts) = finally_body {
        let finally_result = self.execute_block(finally_stmts);
        // finally result can override return/exception
        if !matches!(finally_result, ExecutionResult::Ok) {
            result = finally_result;
        }
    }
    
    result
}

Stmt::Throw(expr) => {
    let value = self.evaluate(expr)?;
    if let Value::Object(obj) = value {
        // Create exception from object
        let message = obj.get_property("message")
            .and_then(|v| v.to_string_value())
            .unwrap_or_default();
        ExecutionResult::Exception(ExceptionValue {
            class_name: obj.class_name.clone(),
            message,
            code: 0,
            file: String::new(),
            line: 0,
            previous: None,
        })
    } else {
        Err("Can only throw objects".to_string())
    }
}
```

### Step 10: Add Built-in Exception Class

In `src/interpreter/objects/mod.rs` or a new file, define the base Exception class:

```rust
/// Create built-in Exception class definition
pub fn create_exception_class() -> ClassDef {
    ClassDef {
        name: "Exception".to_string(),
        parent: None,
        interfaces: vec![],
        trait_uses: vec![],
        properties: vec![
            Property {
                name: "message".to_string(),
                visibility: Visibility::Protected,
                default: Some(Expr::String(String::new())),
                readonly: false,
                attributes: vec![],
            },
            Property {
                name: "code".to_string(),
                visibility: Visibility::Protected,
                default: Some(Expr::Integer(0)),
                readonly: false,
                attributes: vec![],
            },
        ],
        methods: vec![
            // __construct($message = "", $code = 0, $previous = null)
            // getMessage(), getCode(), getFile(), getLine(), getPrevious()
        ],
        readonly: false,
        attributes: vec![],
    }
}
```

### Step 11: Add Tests (`tests/exceptions/`)

Create test files:

**tests/exceptions/basic_try_catch.vhpt**
```
--TEST--
Basic try/catch exception handling
--FILE--
<?php
try {
    throw new Exception("Test error");
} catch (Exception $e) {
    echo $e->getMessage();
}
--EXPECT--
Test error
```

**tests/exceptions/try_catch_finally.vhpt**
```
--TEST--
Try/catch/finally execution order
--FILE--
<?php
try {
    echo "try\n";
    throw new Exception("error");
} catch (Exception $e) {
    echo "catch\n";
} finally {
    echo "finally\n";
}
--EXPECT--
try
catch
finally
```

**tests/exceptions/finally_without_catch.vhpt**
```
--TEST--
Try/finally without catch
--FILE--
<?php
function test() {
    try {
        echo "try\n";
        return "returned";
    } finally {
        echo "finally\n";
    }
}
echo test();
--EXPECT--
try
finally
returned
```

**tests/exceptions/multi_catch.vhpt**
```
--TEST--
Multi-catch with pipe operator (PHP 7.1+)
--FILE--
<?php
class CustomException extends Exception {}

try {
    throw new CustomException("custom error");
} catch (InvalidArgumentException | CustomException $e) {
    echo "Caught: " . $e->getMessage();
}
--EXPECT--
Caught: custom error
```

**tests/exceptions/throw_expression.vhpt**
```
--TEST--
Throw as expression (PHP 8.0+)
--FILE--
<?php
function getValue($val) {
    return $val ?? throw new Exception("Value required");
}

try {
    $result = getValue(null);
} catch (Exception $e) {
    echo $e->getMessage();
}
--EXPECT--
Value required
```

**tests/exceptions/nested_try_catch.vhpt**
```
--TEST--
Nested try/catch blocks
--FILE--
<?php
try {
    try {
        throw new Exception("inner");
    } catch (Exception $e) {
        echo "caught inner: " . $e->getMessage() . "\n";
        throw new Exception("rethrown");
    }
} catch (Exception $e) {
    echo "caught outer: " . $e->getMessage();
}
--EXPECT--
caught inner: inner
caught outer: rethrown
```

**tests/exceptions/uncaught_exception.vhpt**
```
--TEST--
Uncaught exception error
--FILE--
<?php
throw new Exception("Uncaught!");
--EXPECT_ERROR--
Uncaught Exception: Uncaught!
```

### Step 12: Update Documentation

Update these files:
- `AGENTS.md`: Mark exception handling as implemented in Phase 7
- `docs/features.md`: Add exception handling section
- `docs/roadmap.md`: Update Phase 7 status

## Key Considerations

1. **Exception inheritance**: `Exception` is the base class. Custom exceptions should extend it.
2. **Finally always runs**: Even with return statements in try/catch.
3. **Re-throwing**: Caught exceptions can be thrown again.
4. **Multi-catch precedence**: First matching catch clause wins.
5. **Throw expression context**: Valid in null coalesce, ternary, arrow functions.

## PHP Compatibility Notes

- PHP 7.1+: Multi-catch with `|` separator
- PHP 8.0+: `throw` can be used as expression
- Exception `$previous` parameter for exception chaining
- `getMessage()`, `getCode()`, `getFile()`, `getLine()`, `getTrace()` methods

## Reference Implementation

Similar patterns exist in:
- `Stmt::If` for multi-branch control flow
- `Stmt::Switch` for case matching
- `Stmt::Match` for expression-based matching
