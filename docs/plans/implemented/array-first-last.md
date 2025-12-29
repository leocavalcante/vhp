# Plan: array_first() and array_last() Functions

## Overview

PHP 8.5 introduces `array_first()` and `array_last()` functions to get the first and last elements of an array without modifying the array's internal pointer.

**PHP Example:**
```php
<?php
$arr = [10, 20, 30, 40];

echo array_first($arr);  // 10
echo array_last($arr);   // 40

// Works with associative arrays
$assoc = ['a' => 1, 'b' => 2, 'c' => 3];
echo array_first($assoc);  // 1
echo array_last($assoc);   // 3

// Empty array returns null
echo array_first([]);  // null

// With callback for filtering
echo array_first([1, 2, 3, 4], fn($v) => $v > 2);  // 3
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/interpreter/builtins/array.rs` | Add array_first and array_last |
| `tests/builtins/array_first.vhpt` | Tests |
| `tests/builtins/array_last.vhpt` | Tests |

## Implementation Steps

### Step 1: Add array_first Function (`src/interpreter/builtins/array.rs`)

```rust
/// array_first(array $array, ?callable $callback = null): mixed
/// Get the first element of an array, optionally filtered by callback
pub fn array_first(&mut self, args: &[Expr]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("array_first() expects at least 1 parameter, 0 given".to_string());
    }
    
    let arr = self.evaluate(&args[0])?;
    
    let array = match arr {
        Value::Array(a) => a,
        _ => return Err("array_first() expects parameter 1 to be array".to_string()),
    };
    
    if array.is_empty() {
        return Ok(Value::Null);
    }
    
    // Check for callback
    if args.len() > 1 {
        let callback = self.evaluate(&args[1])?;
        
        // Find first element matching callback
        for (_, value) in &array {
            let matches = self.call_callback(&callback, vec![value.clone()])?;
            if matches.to_bool() {
                return Ok(value.clone());
            }
        }
        
        // No match found
        return Ok(Value::Null);
    }
    
    // Return first element (arrays maintain insertion order)
    if let Some((_, value)) = array.iter().next() {
        Ok(value.clone())
    } else {
        Ok(Value::Null)
    }
}
```

### Step 2: Add array_last Function

```rust
/// array_last(array $array, ?callable $callback = null): mixed
/// Get the last element of an array, optionally filtered by callback
pub fn array_last(&mut self, args: &[Expr]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("array_last() expects at least 1 parameter, 0 given".to_string());
    }
    
    let arr = self.evaluate(&args[0])?;
    
    let array = match arr {
        Value::Array(a) => a,
        _ => return Err("array_last() expects parameter 1 to be array".to_string()),
    };
    
    if array.is_empty() {
        return Ok(Value::Null);
    }
    
    // Check for callback
    if args.len() > 1 {
        let callback = self.evaluate(&args[1])?;
        
        // Find last element matching callback (iterate in reverse)
        for (_, value) in array.iter().rev() {
            let matches = self.call_callback(&callback, vec![value.clone()])?;
            if matches.to_bool() {
                return Ok(value.clone());
            }
        }
        
        // No match found
        return Ok(Value::Null);
    }
    
    // Return last element
    if let Some((_, value)) = array.iter().last() {
        Ok(value.clone())
    } else {
        Ok(Value::Null)
    }
}
```

### Step 3: Register Functions

In the builtin function registry:

```rust
fn call_builtin(&mut self, name: &str, args: &[Expr]) -> Result<Option<Value>, String> {
    match name.to_lowercase().as_str() {
        // ... existing functions ...
        "array_first" => Ok(Some(self.array_first(args)?)),
        "array_last" => Ok(Some(self.array_last(args)?)),
        // ...
    }
}
```

### Step 4: Add Tests

**tests/builtins/array_first.vhpt**
```
--TEST--
array_first basic usage
--FILE--
<?php
$arr = [10, 20, 30];
echo array_first($arr);
--EXPECT--
10
```

**tests/builtins/array_first_empty.vhpt**
```
--TEST--
array_first with empty array
--FILE--
<?php
$result = array_first([]);
var_dump($result);
--EXPECT--
NULL
```

**tests/builtins/array_first_assoc.vhpt**
```
--TEST--
array_first with associative array
--FILE--
<?php
$arr = ['x' => 100, 'y' => 200, 'z' => 300];
echo array_first($arr);
--EXPECT--
100
```

**tests/builtins/array_first_callback.vhpt**
```
--TEST--
array_first with callback filter
--FILE--
<?php
$arr = [1, 2, 3, 4, 5];
echo array_first($arr, fn($v) => $v > 3);
--EXPECT--
4
```

**tests/builtins/array_first_callback_no_match.vhpt**
```
--TEST--
array_first callback with no match
--FILE--
<?php
$arr = [1, 2, 3];
$result = array_first($arr, fn($v) => $v > 10);
var_dump($result);
--EXPECT--
NULL
```

**tests/builtins/array_last.vhpt**
```
--TEST--
array_last basic usage
--FILE--
<?php
$arr = [10, 20, 30];
echo array_last($arr);
--EXPECT--
30
```

**tests/builtins/array_last_empty.vhpt**
```
--TEST--
array_last with empty array
--FILE--
<?php
$result = array_last([]);
var_dump($result);
--EXPECT--
NULL
```

**tests/builtins/array_last_assoc.vhpt**
```
--TEST--
array_last with associative array
--FILE--
<?php
$arr = ['x' => 100, 'y' => 200, 'z' => 300];
echo array_last($arr);
--EXPECT--
300
```

**tests/builtins/array_last_callback.vhpt**
```
--TEST--
array_last with callback filter
--FILE--
<?php
$arr = [1, 2, 3, 4, 5];
echo array_last($arr, fn($v) => $v < 4);
--EXPECT--
3
```

**tests/builtins/array_first_single.vhpt**
```
--TEST--
array_first with single element
--FILE--
<?php
echo array_first([42]);
--EXPECT--
42
```

**tests/builtins/array_last_single.vhpt**
```
--TEST--
array_last with single element
--FILE--
<?php
echo array_last([42]);
--EXPECT--
42
```

## PHP 8.5 Compatibility Notes

These functions are new in PHP 8.5 (RFC accepted). They differ from existing approaches:

| Approach | Modifies Pointer | Returns |
|----------|------------------|---------|
| `reset($arr)` | Yes | First element |
| `end($arr)` | Yes | Last element |
| `$arr[array_key_first($arr)]` | No | First element |
| `$arr[array_key_last($arr)]` | No | Last element |
| `array_first($arr)` | No | First element |
| `array_last($arr)` | No | Last element |

## Key Benefits

1. **Cleaner syntax**: Single function call vs multiple
2. **No pointer modification**: Safer for iteration
3. **Callback support**: Built-in filtering
4. **Null-safe**: Returns null for empty arrays

## Implementation Order

1. Basic array_first without callback
2. Basic array_last without callback
3. Callback support (requires arrow functions or closures)
4. Tests

## Dependencies

- Arrow functions (for callback syntax) - optional, can use string function names
