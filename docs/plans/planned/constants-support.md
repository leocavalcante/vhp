# Constants Support

## Status: Planned

## Overview

Implement support for constants including define(), defined(), class constants, const keyword, and constants in traits.

## Current Status

Basic constants can be used, but advanced features like define(), defined(), and trait constants are not fully implemented.

## Background

Constants are fundamental to PHP for:
- Configuration values
- Version numbers
- API endpoints
- Class-level immutable data
- Mathematical constants

## Requirements

### Procedural Functions

1. **define**
   ```php
   define($name, $value, $case_insensitive = false): bool
   ```
   - Define a named constant
   - Support global constants
   - Support case-insensitive option (deprecated in PHP 8.2)

2. **defined**
   ```php
   defined($name): bool
   ```
   - Check whether a named constant exists

3. **constant**
   ```php
   constant($name): mixed
   ```
   - Return value of a constant

### Class Constants

1. **const keyword**
   ```php
   class MyClass {
       const MAX_SIZE = 100;
       const API_URL = 'https://api.example.com';
   }
   ```

2. **Access constants**
   ```php
   echo MyClass::MAX_SIZE;
   echo self::MAX_SIZE;  // Inside class
   echo parent::MAX_SIZE; // Parent constant
   ```

3. **Constant visibility** (PHP 7.1)
   ```php
   class MyClass {
       public const PUBLIC = 'public';
       protected const PROTECTED = 'protected';
       private const PRIVATE = 'private';
   }
   ```

4. **Final constants** (PHP 8.1)
   ```php
   class MyClass {
       final const FINAL_CONST = 'cannot override';
   }
   ```

5. **Interface constants**
   ```php
   interface MyInterface {
       const INTERFACE_CONST = 'value';
   }
   ```

### Trait Constants (PHP 8.2)

```php
trait MyTrait {
    const TRAIT_CONST = 'value';
}

class MyClass {
    use MyTrait;
}
```

### Magic Constants

Already implemented:
- `__LINE__`
- `__FILE__`
- `__DIR__`
- `__FUNCTION__`
- `__CLASS__`
- `__METHOD__`
- `__TRAIT__` (if in trait)
- `__NAMESPACE__`
- `__COMPILER_HALT_OFFSET__`

### Constant Expressions

Support constant expressions for:
- Default parameter values
- Default property values
- const initializers
- Static variable initializers

Allowed in constant expressions:
- Scalars (int, float, string, bool)
- Arrays (PHP 5.6+)
- Constant arrays (PHP 7.0+)
- Access to class/interface constants
- Some operators: `+`, `-`, `*`, `/`, `%`, `**`, `<<`, `>>`, `.`, `|`, `&`, `^`, `~`, `!`, `&&`, `||`, `??`
- Ternary operator
- Parentheses

### compile-time Constants

PHP 8.3+:
- Class constants can be evaluated at compile time in some cases

## Implementation Plan

### Phase 1: Procedural Constants

**File:** `runtime/constants.rs` (new or extend existing)

**Tasks:**
- [ ] Implement define() with global constant storage
- [ ] Implement defined() to check existence
- [ ] Implement constant() to retrieve value
- [ ] Handle case-insensitive option
- [ ] Add error handling for redefinition
- [ ] Add tests

### Phase 2: Class const Parsing

**File:** `ast/stmt.rs` (existing)

```rust
pub enum Stmt {
    // ... existing
    ClassConstant {
        name: String,
        value: Expr,
        visibility: Visibility,
        final: bool,
    },
}
```

**File:** `parser/stmt.rs` (existing)

**Tasks:**
- [ ] Parse const keyword in class body
- [ ] Parse visibility modifiers (public, protected, private)
- [ ] Parse final modifier
- [ ] Parse constant value expressions
- [ ] Add multiple constants per statement
- [ ] Add tests

### Phase 3: Class const Compilation

**File:** `vm/compiler/definitions.rs` (existing)

**Tasks:**
- [ ] Compile class constants
- [ ] Store constants in CompiledClass
- [ ] Validate constant expressions at compile time
- [ ] Add error handling for invalid constant expressions
- [ ] Add tests

### Phase 4: Class const Execution

**File:** `vm/class.rs` (existing)

```rust
pub struct CompiledClass {
    // ... existing
    pub constants: HashMap<String, Value>,
    pub constant_visibility: HashMap<String, Visibility>,
    pub constant_final: HashSet<String>,
}
```

**File:** `runtime/value.rs` or `runtime/object.rs` (existing)

**Tasks:**
- [ ] Implement Class::constant() method
- [ ] Handle ClassName::CONST_NAME syntax
- [ ] Handle self::CONST_NAME
- [ ] Handle parent::CONST_NAME
- [ ] Check visibility for constants
- [ ] Handle final constants
- [ ] Add tests

### Phase 5: Interface Constants

**Tasks:**
- [ ] Parse const in interface body
- [ ] Compile interface constants
- [ ] Implement interface constant access
- [ ] Handle constant inheritance through interfaces
- [ ] Add tests

### Phase 6: Trait Constants (PHP 8.2)

**File:** `ast/stmt.rs` (extend)
**File:** `parser/stmt.rs` (extend)

**Tasks:**
- [ ] Parse const in trait body
- [ ] Compile trait constants
- [ ] Implement trait constant access via use
- [ ] Handle constant conflicts in traits
- [ ] Add tests

### Phase 7: Constant Expression Validation

**File:** `vm/compiler/compiler_types.rs` (existing)

**Tasks:**
- [ ] Create constant expression validator
- [ ] Validate allowed operations in const expressions
- [ ] Detect non-constant expressions at compile time
- [ ] Support array expressions in constants
- [ ] Support class/interface constant access
- [ ] Add tests

### Phase 8: compile-time Evaluation (PHP 8.3)

**Tasks:**
- [ ] Evaluate constant expressions at compile time where possible
- [ ] Handle arithmetic operations in constants
- [ ] Handle string concatenation in constants
- [ ] Handle array operations in constants
- [ ] Add tests

### Phase 9: Visibility Checking

**Tasks:**
- [ ] Enforce public constant visibility
- [ ] Enforce protected constant visibility (inside class hierarchy)
- [ ] Enforce private constant visibility (inside class only)
- [ ] Handle visibility in final constants
- [ ] Add tests

### Phase 10: Final Constant Enforcement

**Tasks:**
- [ ] Mark final constants in class definition
- [ ] Prevent overriding final constants in child classes
- [ ] Add error messages for override attempts
- [ ] Add tests

### Phase 11: Global Constant Storage

**File:** `runtime/mod.rs` or `runtime/constants.rs`

```rust
use std::collections::HashMap;

pub struct GlobalConstants {
    constants: HashMap<String, ConstantInfo>,
}

pub struct ConstantInfo {
    value: Value,
    case_insensitive: bool,
    defined_at: String, // File:line for error messages
}
```

**Tasks:**
- [ ] Create global constant storage
- [ ] Initialize with PHP predefined constants
- [ ] Support define() and defined() access
- [ ] Handle case-insensitive lookup
- [ ] Add tests

### Phase 12: Predefined Constants

**Tasks:**
- [ ] Define common PHP constants
  - PHP_VERSION, PHP_MAJOR_VERSION, etc.
  - PHP_OS, PHP_EOL
  - TRUE, FALSE, NULL
  - E_ERROR, E_WARNING, E_NOTICE, etc.
  - PHP_INT_MAX, PHP_INT_SIZE
  - __DIR__, __FILE__, etc.
- [ ] Add tests

### Phase 13: Tests

**File:** `tests/constants/` (new directory)

Test coverage:
- define() and defined()
- constant()
- Class constants
- Class constant visibility (public, protected, private)
- Final constants
- Interface constants
- Trait constants
- Constant expressions
- compile-time constant evaluation
- Predefined constants
- Edge cases and errors

## Implementation Details

### Constant Expression Validator

```rust
fn is_constant_expression(expr: &Expr, compiler: &Compiler) -> bool {
    match expr {
        Expr::Int(_) | Expr::Float(_) | Expr::String(_) | Expr::Bool(_) | Expr::Null => true,
        Expr::Array { .. } => true, // PHP 5.6+
        Expr::BinaryOp { op, left, right } => {
            is_constant_binary_op(op) &&
            is_constant_expression(left, compiler) &&
            is_constant_expression(right, compiler)
        },
        Expr::Ternary { condition, then_branch, else_branch } => {
            is_constant_expression(condition, compiler) &&
            is_constant_expression(then_branch, compiler) &&
            is_constant_expression(else_branch, compiler)
        },
        Expr::ConstAccess { class_name, const_name } => {
            // Check if class constant exists
            compiler.class_exists(class_name) &&
                compiler.constant_exists(class_name, const_name)
        },
        _ => false,
    }
}
```

### Class Constant Access

```rust
impl Instance {
    pub fn get_class_constant(&self, class_name: &str, const_name: &str) -> Result<Value, String> {
        let class = self.get_class(class_name)?;
        let visibility = class.get_constant_visibility(const_name)?;

        // Check visibility based on calling context
        match visibility {
            Visibility::Public => Ok(class.get_constant_value(const_name)?),
            Visibility::Protected => {
                if self.is_class_hierarchy(class_name) {
                    Ok(class.get_constant_value(const_name)?)
                } else {
                    Err("Cannot access protected constant".to_string())
                }
            },
            Visibility::Private => {
                if self.get_current_class_name() == class_name {
                    Ok(class.get_constant_value(const_name)?)
                } else {
                    Err("Cannot access private constant".to_string())
                }
            },
        }
    }
}
```

### Trait Constants

```rust
pub struct CompiledTrait {
    // ... existing
    pub constants: HashMap<String, Value>,
}

// When trait is used in class, merge constants
fn merge_trait_constants(class: &mut CompiledClass, trait_: &CompiledTrait) -> Result<(), String> {
    for (name, value) in &trait_.constants {
        if class.constants.contains_key(name) {
            return Err(format!("Trait constant {} conflicts with class constant", name));
        }
        class.constants.insert(name.clone(), value.clone());
    }
    Ok(())
}
```

## Dependencies

- Existing class system
- Existing AST
- Existing compiler infrastructure

## Testing Strategy

1. **Unit Tests**: Each function and feature
2. **Integration Tests**: Combined constant usage
3. **Visibility Tests**: Access control enforcement
4. **Expression Tests**: Constant expression validation
5. **Compatibility Tests**: Match PHP 8.x behavior

## Success Criteria

- define() and defined() work correctly
- Class constants with all visibility levels work
- Final constants are enforced
- Trait constants work (PHP 8.2)
- Constant expressions are validated
- All tests pass

## Performance Considerations

- Cache constant values after compilation
- Fast constant lookup (HashMap)
- compile-time evaluation where possible
- Minimize runtime checks for public constants

## Open Questions

- Should we support case-insensitive constants (deprecated in PHP 8.2)?
- How to handle dynamic constant names with constant() function?

## References

- PHP Constants documentation: https://www.php.net/manual/en/language.constants.php
- PHP Class Constants: https://www.php.net/manual/en/language.oop5.constants.php
- PHP 8.2 Trait Constants RFC: https://wiki.php.net/rfc/class_const_trait_compat
- PHP 8.3 Readonly Classes: https://wiki.php.net/rfc/readonly_classes

## Related Plans

- Late static binding (already implemented)
- Class inheritance (already implemented)
