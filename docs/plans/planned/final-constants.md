# Plan: Final Constants (PHP 8.1)

## Overview

PHP 8.1 introduced the `final` modifier for class constants, preventing them from being overridden in child classes. This complements the existing `final` keyword for methods and classes.

**PHP Example:**
```php
<?php
// Before PHP 8.1: Constants could always be overridden
class Base {
    const VERSION = '1.0';
}

class Child extends Base {
    const VERSION = '2.0'; // Always allowed
}

// PHP 8.1+: Final constants cannot be overridden
class Config {
    final const API_VERSION = '1.0';
    const APP_VERSION = '2.0'; // Not final
}

class ExtendedConfig extends Config {
    // Error: Cannot override final constant
    const API_VERSION = '2.0';

    // OK: Not final
    const APP_VERSION = '3.0';
}
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/ast/stmt.rs` | Add `is_final` field to `Constant` struct (if not already present) |
| `src/parser/stmt.rs` | Parse `final const` syntax |
| `src/interpreter/stmt_exec/definitions.rs` | Validate final constants on class inheritance |
| `tests/classes/final_const_*.vhpt` | Test files |

## Implementation Steps

### Step 1: Update Constant AST (`src/ast/stmt.rs`)

Ensure the `Constant` struct has an `is_final` field:

```rust
/// Class/Interface/Trait constant
#[derive(Debug, Clone)]
pub struct Constant {
    pub name: String,
    pub value: Expr,
    pub visibility: Visibility,     // PHP 7.1+
    pub is_final: bool,             // PHP 8.1+
    pub attributes: Vec<Attribute>, // PHP 8.0+
}
```

If this already exists (from trait constants plan), no changes needed.

### Step 2: Parse Final Constants (`src/parser/stmt.rs`)

Update constant parsing to handle the `final` keyword. Find where constants are parsed in classes:

```rust
fn parse_class_member(
    &mut self,
    visibility: Visibility,
    attributes: Vec<Attribute>,
) -> Result<ClassMember, String> {
    // Check for 'final' keyword
    let is_final = if self.check(&TokenKind::Final) {
        self.advance();
        true
    } else {
        false
    };

    if self.check(&TokenKind::Const) {
        // Parse constant with final modifier
        return Ok(ClassMember::Constant(
            self.parse_constant_with_modifiers(visibility, is_final, attributes)?
        ));
    }

    if is_final {
        // 'final' can only be used with const, methods, or classes
        if self.check(&TokenKind::Function) {
            // Final method - existing logic
            // ...
        } else {
            return Err("'final' can only be used with constants, methods, or classes".to_string());
        }
    }

    // ... rest of member parsing ...
}
```

Ensure constant parsing includes the `is_final` flag:

```rust
fn parse_constant_with_modifiers(
    &mut self,
    visibility: Visibility,
    is_final: bool,
    attributes: Vec<Attribute>,
) -> Result<Constant, String> {
    self.expect(&TokenKind::Const)?;

    let name = self.expect_identifier()?;

    self.expect(&TokenKind::Assign)?;

    let value = self.parse_expression()?;

    self.expect(&TokenKind::Semicolon)?;

    Ok(Constant {
        name,
        value,
        visibility,
        is_final,
        attributes,
    })
}
```

### Step 3: Validate Final Constants on Inheritance (`src/interpreter/stmt_exec/definitions.rs`)

When defining a class, check if it overrides any final constants from parent:

```rust
fn execute_class_declaration(&mut self, class: &ClassDecl) -> Result<(), String> {
    // ... existing class definition logic ...

    // Validate that we don't override final constants
    if let Some(parent_name) = &class.parent {
        self.validate_no_final_override(class, parent_name)?;
    }

    // ... store class definition ...

    Ok(())
}
```

Add the validation method:

```rust
impl<W: Write> Interpreter<W> {
    /// Check that child class doesn't override parent's final constants
    fn validate_no_final_override(
        &self,
        child_class: &ClassDecl,
        parent_name: &str,
    ) -> Result<(), String> {
        // Get parent class definition
        let parent_key = parent_name.to_lowercase();
        let parent = self.classes.get(&parent_key)
            .ok_or_else(|| format!("Parent class '{}' not found", parent_name))?;

        // Check each constant in child class
        for child_const in &child_class.constants {
            // Look for same constant in parent (including ancestor chain)
            if let Some(parent_const) = self.find_constant_in_hierarchy(
                parent_name,
                &child_const.name,
            )? {
                // Check if parent constant is final
                if parent_const.is_final {
                    return Err(format!(
                        "{}::{} cannot override final constant {}::{}",
                        child_class.name,
                        child_const.name,
                        parent_name,
                        parent_const.name
                    ));
                }
            }
        }

        Ok(())
    }

    /// Find a constant in class hierarchy (current class and ancestors)
    fn find_constant_in_hierarchy(
        &self,
        class_name: &str,
        constant_name: &str,
    ) -> Result<Option<Constant>, String> {
        let class_key = class_name.to_lowercase();

        let class_def = self.classes.get(&class_key)
            .ok_or_else(|| format!("Class '{}' not found", class_name))?;

        // Check in current class
        for constant in &class_def.constants {
            if constant.name.eq_ignore_ascii_case(constant_name) {
                return Ok(Some(constant.clone()));
            }
        }

        // Check in parent class recursively
        if let Some(parent_name) = &class_def.parent {
            return self.find_constant_in_hierarchy(parent_name, constant_name);
        }

        Ok(None)
    }
}
```

### Step 4: Handle Final Constants in Traits

If trait constants are implemented, ensure final trait constants also cannot be overridden:

```rust
fn validate_trait_final_constants(
    &self,
    class: &ClassDecl,
) -> Result<(), String> {
    // Check all traits used by this class
    for trait_name in &class.traits {
        let trait_key = trait_name.to_lowercase();

        if let Some(trait_def) = self.traits.get(&trait_key) {
            // Check each trait constant
            for trait_const in &trait_def.constants {
                if trait_const.is_final {
                    // Check if class tries to override it
                    for class_const in &class.constants {
                        if class_const.name.eq_ignore_ascii_case(&trait_const.name) {
                            return Err(format!(
                                "{}::{} cannot override final constant from trait {}",
                                class.name,
                                class_const.name,
                                trait_name
                            ));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
```

### Step 5: Add Tests

**tests/classes/final_const_basic.vhpt**
```
--TEST--
Final constant cannot be overridden
--FILE--
<?php
class Base {
    final const VERSION = '1.0';
}

class Child extends Base {
    const VERSION = '2.0';
}
--EXPECT_ERROR--
cannot override final constant
```

**tests/classes/final_const_allowed.vhpt**
```
--TEST--
Non-final constant can be overridden
--FILE--
<?php
class Base {
    const VERSION = '1.0';
}

class Child extends Base {
    const VERSION = '2.0';
}

echo Child::VERSION;
--EXPECT--
2.0
```

**tests/classes/final_const_with_visibility.vhpt**
```
--TEST--
Final constant with visibility modifier
--FILE--
<?php
class Config {
    public final const PUBLIC_KEY = 'abc';
    protected final const PROTECTED_KEY = 'def';
    private final const PRIVATE_KEY = 'ghi';
}

echo Config::PUBLIC_KEY;
--EXPECT--
abc
```

**tests/classes/final_const_grandparent.vhpt**
```
--TEST--
Cannot override final constant from grandparent
--FILE--
<?php
class GrandParent {
    final const VALUE = 1;
}

class Parent extends GrandParent {
}

class Child extends Parent {
    const VALUE = 2;
}
--EXPECT_ERROR--
cannot override final constant
```

**tests/classes/final_const_interface.vhpt**
```
--TEST--
Interface constants are implicitly final (cannot be overridden)
--FILE--
<?php
interface Config {
    const VERSION = '1.0';
}

class App implements Config {
    const VERSION = '2.0';
}
--EXPECT_ERROR--
cannot override final constant
```

**tests/classes/final_const_multiple.vhpt**
```
--TEST--
Multiple final and non-final constants
--FILE--
<?php
class Base {
    final const FINAL_ONE = 'a';
    const NORMAL_ONE = 'b';
    final const FINAL_TWO = 'c';
    const NORMAL_TWO = 'd';
}

class Child extends Base {
    const NORMAL_ONE = 'B';  // OK
    const NORMAL_TWO = 'D';  // OK
}

echo Child::NORMAL_ONE . Child::NORMAL_TWO;
--EXPECT--
BD
```

**tests/classes/final_const_access.vhpt**
```
--TEST--
Final constant can be accessed normally
--FILE--
<?php
class Settings {
    final public const MAX_USERS = 100;
    final protected const MAX_RETRIES = 3;

    public static function getMaxRetries() {
        return self::MAX_RETRIES;
    }
}

echo Settings::MAX_USERS . "\n";
echo Settings::getMaxRetries();
--EXPECT--
100
3
```

**tests/classes/final_const_trait.vhpt**
```
--TEST--
Cannot override final constant from trait
--FILE--
<?php
trait Config {
    final const VERSION = '1.0';
}

class App {
    use Config;

    const VERSION = '2.0';
}
--EXPECT_ERROR--
cannot override final constant from trait
```

**tests/classes/final_const_case_insensitive.vhpt**
```
--TEST--
Final constant check is case-insensitive
--FILE--
<?php
class Base {
    final const MyConst = 'value';
}

class Child extends Base {
    const MYCONST = 'other';
}
--EXPECT_ERROR--
cannot override final constant
```

**tests/classes/final_const_no_override.vhpt**
```
--TEST--
Child can define new final constant (not an override)
--FILE--
<?php
class Base {
    final const BASE_CONST = 1;
}

class Child extends Base {
    final const CHILD_CONST = 2;
}

echo Base::BASE_CONST . "\n";
echo Child::CHILD_CONST;
--EXPECT--
1
2
```

**tests/classes/final_const_redeclare_same_value.vhpt**
```
--TEST--
Cannot redeclare final constant even with same value
--FILE--
<?php
class Base {
    final const VALUE = 42;
}

class Child extends Base {
    const VALUE = 42; // Same value, still error
}
--EXPECT_ERROR--
cannot override final constant
```

## PHP Compatibility Notes

| Feature | PHP Version |
|---------|-------------|
| Final constants | 8.1 |
| Syntax: `final const NAME` | 8.1 |
| Syntax: `public final const NAME` | 8.1 |
| Interface constants are implicitly final | All versions |

## Key Considerations

1. **Syntax**: `final const NAME = value` or `visibility final const NAME = value`
2. **Order**: Visibility comes before `final`: `public final const X = 1`
3. **Interface constants**: All interface constants are implicitly final (even before PHP 8.1)
4. **Inheritance chain**: Must check entire parent chain, not just immediate parent
5. **Trait constants**: Final trait constants cannot be overridden by class
6. **Case insensitivity**: Constant name matching is case-sensitive (unlike method names)
7. **Validation timing**: Check at class definition time, not constant access time

## Validation Rules

A constant is considered overridden if:
1. Parent class has a constant with the same name (case-insensitive)
2. Child class defines a constant with that name

A final constant cannot be overridden:
1. If parent defines `final const X`, child cannot define `const X`
2. If trait defines `final const X`, class using trait cannot define `const X`
3. Interface constants are always final (implicit)

## Error Messages

- `<Child>::<constant> cannot override final constant <Parent>::<constant>`
- `<Class>::<constant> cannot override final constant from trait <Trait>`
- `'final' can only be used with constants, methods, or classes`

## Implementation Order

1. Ensure `is_final` field exists on Constant struct
2. Parse `final` keyword before `const`
3. Parse order: `visibility final const NAME = value`
4. Add validation on class definition
5. Check parent chain for final constant conflicts
6. Check trait constants for final conflicts
7. Add comprehensive tests

## Edge Cases

1. **Private final constant**: Can be final, but already not inheritable due to visibility
2. **Final in interface**: All interface constants are implicitly final
3. **Same constant in parent and trait**: Need to check both
4. **Multiple traits with final constant**: Conflict resolution still applies
5. **Redefining with same value**: Still an error (override is override)
6. **Child defines new constant**: OK if different name
7. **Grandparent final constant**: Must check entire hierarchy

## Interface Constant Behavior

Note: Interface constants are implicitly final:

```rust
// When defining a class that implements an interface
fn validate_interface_constants(&self, class: &ClassDecl) -> Result<(), String> {
    for interface_name in &class.interfaces {
        let interface_key = interface_name.to_lowercase();

        if let Some(interface_def) = self.interfaces.get(&interface_key) {
            for interface_const in &interface_def.constants {
                // Check if class tries to override interface constant
                for class_const in &class.constants {
                    if class_const.name.eq_ignore_ascii_case(&interface_const.name) {
                        return Err(format!(
                            "{}::{} cannot override final constant from interface {}",
                            class.name,
                            class_const.name,
                            interface_name
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}
```

## Reference Implementations

- Final methods: Already implemented
- Final classes: Already implemented
- Class constants: Already implemented
- Constant visibility: Already implemented (PHP 7.1+)
- Trait constants: See trait-constants.md plan

## Additional Validation

When class is defined:
1. Check parent class constants (recursive)
2. Check interface constants (all implemented interfaces)
3. Check trait constants (all used traits)
4. For each child constant, ensure parent/interface/trait version is not final

## Performance Considerations

- Validation happens once at class definition time
- Recursive checks through inheritance chain (typically shallow)
- No runtime overhead after validation

## Note on Constant Behavior

Unlike methods, constants:
- Use case-sensitive names (PHP 8.0+, case-insensitive before)
- Cannot be defined at runtime (must be in class definition)
- Are copied, not inherited (child gets copy of parent constants)
- Cannot use late static binding (always `self::`, never `static::`)
