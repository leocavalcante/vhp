# Plan: Exception Handling (try/catch/finally/throw) ✅

**Status:** ✅ IMPLEMENTED  
**Completed:** December 2024  
**Tests:** 11 test files in `tests/exceptions/`

## Overview

Exception handling is fundamental to PHP and required for proper error management. This feature adds:
- `try`/`catch`/`finally` statements
- `throw` keyword (both statement and expression in PHP 8.0+)
- Base `Exception` class with `getMessage()` and `getCode()` methods
- Multiple catch blocks
- Multi-catch syntax (PHP 7.1+): `catch (TypeA | TypeB $e)`
- Exception inheritance support

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

## Implementation Summary

### Files Modified

| File | Changes |
|------|---------|
| `src/token.rs` | Added `Try`, `Catch`, `Finally`, `Throw` tokens |
| `src/lexer/mod.rs` | Recognition of exception keywords |
| `src/ast/stmt.rs` | Added `TryCatch` statement, `Throw` statement, `CatchClause` struct |
| `src/ast/expr.rs` | Added `ThrowExpr` for throw expressions (PHP 8.0) |
| `src/parser/stmt/mod.rs` | Parsing for try/catch/finally and throw statements |
| `src/parser/expr/mod.rs` | Parsing for throw as expression |
| `src/interpreter/mod.rs` | Exception propagation and handling logic |
| `src/interpreter/value.rs` | Added `Exception` value variant |
| `src/interpreter/objects/mod.rs` | Base `Exception` class implementation |
| `src/interpreter/stmt_exec/mod.rs` | Execution of try/catch/finally blocks |
| `tests/exceptions/*.vhpt` | 11 comprehensive test files |

### Test Coverage

All 11 test files pass successfully:

1. **basic_try_catch.vhpt** - Basic exception catching
2. **catch_child_via_parent.vhpt** - Exception inheritance
3. **exception_get_message.vhpt** - getMessage() method
4. **exception_with_message.vhpt** - Constructor with message
5. **finally_without_catch.vhpt** - Finally block without catch
6. **multi_catch.vhpt** - Multi-catch with pipe operator
7. **multiple_catch_clauses.vhpt** - Multiple sequential catch blocks
8. **nested_try_catch.vhpt** - Nested exception handling
9. **throw_expression.vhpt** - Throw as expression (PHP 8.0)
10. **try_catch_finally.vhpt** - Complete try/catch/finally flow
11. **uncaught_exception.vhpt** - Uncaught exception error handling

## Features Implemented

### Core Exception Handling

✅ **try/catch/finally statements** - Complete control flow for exception handling  
✅ **throw keyword** - Both statement and expression forms  
✅ **Base Exception class** - With `getMessage()` and `getCode()` methods  
✅ **Multiple catch blocks** - Sequential exception type checking  
✅ **Multi-catch (PHP 7.1)** - `catch (TypeA | TypeB $e)` syntax  
✅ **Exception inheritance** - Child exceptions caught by parent handlers  
✅ **Throw expressions (PHP 8.0)** - Throw in null coalesce, ternary, arrow functions  
✅ **Finally always executes** - Runs regardless of exception or return  

### Exception Class

The base `Exception` class provides:
- Constructor: `new Exception($message, $code)`
- `getMessage()` - Returns exception message string
- `getCode()` - Returns exception code integer
- Inheritance support for custom exception classes

### Multi-Catch Syntax

Supports PHP 7.1+ multi-catch with pipe separator:
```php
catch (TypeA | TypeB | TypeC $e) {
    // Handle any of these types
}
```

### Throw as Expression

PHP 8.0+ throw expressions work in:
- Null coalescing: `$x ?? throw new Exception()`
- Ternary: `$cond ? $val : throw new Exception()`
- Arrow functions: `fn() => throw new Exception()`

## Key Implementation Details

### Exception Propagation

Exceptions propagate up the call stack until caught or terminating execution. The interpreter uses a result type that can carry exception values through statement execution.

### Finally Block Semantics

The finally block:
- Always executes, even with return/break/continue in try/catch
- Can override return values if it contains its own return
- Executes before exception propagates if not caught

### Exception Matching

Catch blocks match exceptions by:
1. Exact class name match
2. Parent class match (inheritance)
3. First matching catch wins (order matters)

### Multi-Catch Implementation

Multiple exception types in a single catch clause are separated by `|` and checked against the thrown exception type. Any match triggers that catch block.

## PHP Compatibility

VHP's exception handling is fully compatible with:
- **PHP 5.0+**: Basic try/catch/finally
- **PHP 7.1+**: Multi-catch syntax
- **PHP 8.0+**: Throw expressions

## Usage Examples

### Basic Error Handling

```php
<?php
function divide($a, $b) {
    if ($b === 0) {
        throw new Exception("Division by zero");
    }
    return $a / $b;
}

try {
    echo divide(10, 0);
} catch (Exception $e) {
    echo "Error: " . $e->getMessage();
}
```

### Custom Exception Classes

```php
<?php
class ValidationException extends Exception {}
class DatabaseException extends Exception {}

try {
    throw new ValidationException("Invalid input");
} catch (ValidationException $e) {
    echo "Validation: " . $e->getMessage();
} catch (DatabaseException $e) {
    echo "Database: " . $e->getMessage();
}
```

### Resource Cleanup with Finally

```php
<?php
function processFile($filename) {
    try {
        echo "Opening $filename\n";
        if (!file_exists($filename)) {
            throw new Exception("File not found");
        }
        // Process file
    } finally {
        echo "Cleanup resources\n";
    }
}
```

## Documentation Updates

All documentation has been updated to reflect this implementation:

- ✅ **AGENTS.md** - Phase 7 marked as in progress, exception handling marked complete
- ✅ **README.md** - Features list includes exception handling
- ✅ **docs/features.md** - Comprehensive exception handling section with examples
- ✅ **docs/roadmap.md** - Phase 7 exception handling items checked
- ✅ **docs/examples.md** - Exception handling examples added

## Future Enhancements

Potential additions for complete PHP compatibility:

- [ ] `getFile()`, `getLine()` methods with source location tracking
- [ ] `getTrace()` and `getTraceAsString()` for stack traces
- [ ] `getPrevious()` for exception chaining
- [ ] SPL Exception subclasses (RuntimeException, LogicException, etc.)
- [ ] `Error` class hierarchy for fatal errors
- [ ] Exception handler customization with `set_exception_handler()`

## Conclusion

Exception handling is now fully implemented in VHP, providing robust error management capabilities that match PHP 7.1+ and PHP 8.0+ semantics. The implementation includes 11 comprehensive tests covering all major exception handling scenarios, from basic try/catch to advanced features like multi-catch and throw expressions.
