# Plan: Final Classes, Methods, and Constants

## Overview

The `final` keyword prevents classes from being extended and methods from being overridden. PHP 8.1 added support for final class constants.

**PHP Example:**
```php
<?php
// Final class cannot be extended
final class Singleton {
    private static $instance = null;
    
    public static function getInstance() {
        if (self::$instance === null) {
            self::$instance = new self();
        }
        return self::$instance;
    }
    
    private function __construct() {}
}

// class ExtendedSingleton extends Singleton {} // Error!

// Final method cannot be overridden
class Base {
    final public function cannotOverride() {
        return "immutable";
    }
    
    public function canOverride() {
        return "base";
    }
}

class Child extends Base {
    // Cannot override cannotOverride()
    
    public function canOverride() {
        return "child";
    }
}

// Final constant (PHP 8.1+)
class Config {
    final public const VERSION = "1.0.0";
}

class Extended extends Config {
    // public const VERSION = "2.0.0"; // Error! Cannot override final constant
}
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/token.rs` | Add `Final` token |
| `src/ast/stmt.rs` | Add `is_final` to Class, Method, Constant |
| `src/parser/stmt/mod.rs` | Parse final modifier |
| `src/interpreter/objects/mod.rs` | Validate final rules |
| `tests/classes/final_*.vhpt` | Test files |

## Implementation Steps

### Step 1: Add Token (`src/token.rs`)

Add the `final` keyword token:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ... existing tokens ...
    
    Final,
    
    // ... rest of tokens ...
}
```

### Step 2: Update Lexer (`src/lexer/mod.rs`)

Add `final` to keyword recognition:

```rust
fn tokenize_identifier(&mut self) -> TokenKind {
    match ident.to_lowercase().as_str() {
        // ... existing keywords ...
        "final" => TokenKind::Final,
        // ...
    }
}
```

### Step 3: Update Class AST (`src/ast/stmt.rs`)

Add `is_final` flag to `Stmt::Class`:

```rust
Stmt::Class {
    name: String,
    is_abstract: bool,
    is_final: bool,           // NEW: true if final class
    is_readonly: bool,
    parent: Option<String>,
    interfaces: Vec<String>,
    traits: Vec<TraitUse>,
    properties: Vec<Property>,
    methods: Vec<Method>,
    attributes: Vec<Attribute>,
},
```

### Step 4: Update Method Structure (`src/ast/stmt.rs`)

Add `is_final` flag to `Method`:

```rust
#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_abstract: bool,
    pub is_final: bool,           // NEW: true if final method
    pub params: Vec<FunctionParam>,
    pub return_type: Option<TypeHint>,
    pub body: Vec<Stmt>,
    pub attributes: Vec<Attribute>,
}
```

### Step 5: Add Constant Structure with Final Support

Add class constants with final support:

```rust
/// Class constant
#[derive(Debug, Clone)]
pub struct ClassConstant {
    pub name: String,
    pub value: Expr,
    pub visibility: Visibility,
    pub is_final: bool,           // NEW: PHP 8.1+ final constants
    pub attributes: Vec<Attribute>,
}
```

Update `Stmt::Class` to include constants:

```rust
Stmt::Class {
    name: String,
    is_abstract: bool,
    is_final: bool,
    is_readonly: bool,
    parent: Option<String>,
    interfaces: Vec<String>,
    traits: Vec<TraitUse>,
    properties: Vec<Property>,
    methods: Vec<Method>,
    constants: Vec<ClassConstant>,    // NEW
    attributes: Vec<Attribute>,
},
```

### Step 6: Parse Final Modifier (`src/parser/stmt/mod.rs`)

The class parsing was covered in abstract-classes.md. Here's the key final-specific logic:

```rust
fn parse_class_declaration(&mut self) -> Result<Stmt, String> {
    // Parse modifiers
    let mut is_abstract = false;
    let mut is_final = false;
    let mut is_readonly = false;
    
    loop {
        match &self.current_token().kind {
            TokenKind::Abstract => {
                if is_final {
                    return Err("Cannot use 'abstract' with 'final'".to_string());
                }
                is_abstract = true;
                self.advance();
            }
            TokenKind::Final => {
                if is_abstract {
                    return Err("Cannot use 'final' with 'abstract'".to_string());
                }
                is_final = true;
                self.advance();
            }
            TokenKind::Readonly => {
                is_readonly = true;
                self.advance();
            }
            TokenKind::Class => break,
            _ => break,
        }
    }
    
    // ... rest of class parsing ...
}
```

### Step 7: Parse Final Methods and Constants

In class member parsing:

```rust
fn parse_class_member(&mut self) -> Result<ClassMember, String> {
    let attributes = self.parse_attributes()?;
    
    // Collect modifiers
    let mut visibility = None;
    let mut is_static = false;
    let mut is_abstract = false;
    let mut is_final = false;
    
    loop {
        match &self.current_token().kind {
            TokenKind::Public => { visibility = Some(Visibility::Public); self.advance(); }
            TokenKind::Protected => { visibility = Some(Visibility::Protected); self.advance(); }
            TokenKind::Private => { visibility = Some(Visibility::Private); self.advance(); }
            TokenKind::Static => { is_static = true; self.advance(); }
            TokenKind::Abstract => {
                if is_final {
                    return Err("Cannot use 'abstract' with 'final'".to_string());
                }
                is_abstract = true;
                self.advance();
            }
            TokenKind::Final => {
                if is_abstract {
                    return Err("Cannot use 'final' with 'abstract'".to_string());
                }
                is_final = true;
                self.advance();
            }
            _ => break,
        }
    }
    
    // Parse constant
    if self.check(&TokenKind::Const) {
        self.advance();
        let name = self.expect_identifier()?;
        self.expect(&TokenKind::Assign)?;
        let value = self.parse_expression()?;
        self.expect(&TokenKind::Semicolon)?;
        
        return Ok(ClassMember::Constant(ClassConstant {
            name,
            value,
            visibility: visibility.unwrap_or(Visibility::Public),
            is_final,
            attributes,
        }));
    }
    
    // Parse method
    if self.check(&TokenKind::Function) {
        // ... method parsing with is_final ...
    }
    
    // ... property parsing ...
}
```

### Step 8: Validate Final Class Inheritance (`src/interpreter/objects/mod.rs`)

When processing `extends`:

```rust
fn register_class(&mut self, class: &ClassStmt) -> Result<(), String> {
    // Check if parent is final
    if let Some(ref parent_name) = class.parent {
        if let Some(parent) = self.get_class(parent_name) {
            if parent.is_final {
                return Err(format!(
                    "Class {} cannot extend final class {}",
                    class.name,
                    parent_name
                ));
            }
        }
    }
    
    // ... register class ...
}
```

### Step 9: Validate Final Method Override

When a child class defines a method:

```rust
fn validate_method_override(
    &self,
    child_class: &str,
    method_name: &str,
    parent: &Class,
) -> Result<(), String> {
    // Check if parent has final method with same name
    for parent_method in &parent.methods {
        if parent_method.name.eq_ignore_ascii_case(method_name) {
            if parent_method.is_final {
                return Err(format!(
                    "Cannot override final method {}::{}",
                    parent.name,
                    method_name
                ));
            }
        }
    }
    Ok(())
}
```

### Step 10: Validate Final Constant Override

When checking constant inheritance:

```rust
fn validate_constant_override(
    &self,
    child_class: &str,
    const_name: &str,
    parent: &Class,
) -> Result<(), String> {
    for parent_const in &parent.constants {
        if parent_const.name == const_name {
            if parent_const.is_final {
                return Err(format!(
                    "Cannot override final constant {}::{}",
                    parent.name,
                    const_name
                ));
            }
        }
    }
    Ok(())
}
```

### Step 11: Add Tests

**tests/classes/final_class_basic.vhpt**
```
--TEST--
Final class cannot be extended
--FILE--
<?php
final class Immutable {
    public function getValue() {
        return 42;
    }
}

$obj = new Immutable();
echo $obj->getValue();
--EXPECT--
42
```

**tests/classes/final_class_extend_error.vhpt**
```
--TEST--
Cannot extend final class
--FILE--
<?php
final class Base {}
class Child extends Base {}
--EXPECT_ERROR--
cannot extend final class Base
```

**tests/classes/final_method_basic.vhpt**
```
--TEST--
Final method in non-final class
--FILE--
<?php
class Base {
    final public function locked() {
        return "locked";
    }
    
    public function unlocked() {
        return "base";
    }
}

class Child extends Base {
    public function unlocked() {
        return "child";
    }
}

$c = new Child();
echo $c->locked() . "\n";
echo $c->unlocked();
--EXPECT--
locked
child
```

**tests/classes/final_method_override_error.vhpt**
```
--TEST--
Cannot override final method
--FILE--
<?php
class Base {
    final public function noOverride() {}
}

class Child extends Base {
    public function noOverride() {}
}
--EXPECT_ERROR--
Cannot override final method
```

**tests/classes/final_constant.vhpt**
```
--TEST--
Final class constant
--FILE--
<?php
class Config {
    final public const VERSION = "1.0";
}

echo Config::VERSION;
--EXPECT--
1.0
```

**tests/classes/final_constant_override_error.vhpt**
```
--TEST--
Cannot override final constant
--FILE--
<?php
class Base {
    final public const LOCKED = 42;
}

class Child extends Base {
    public const LOCKED = 100;
}
--EXPECT_ERROR--
Cannot override final constant
```

**tests/classes/final_abstract_conflict.vhpt**
```
--TEST--
Cannot use final and abstract together
--FILE--
<?php
final abstract class Invalid {}
--EXPECT_ERROR--
Cannot use 'final' with 'abstract'
```

**tests/classes/final_static_method.vhpt**
```
--TEST--
Final static method
--FILE--
<?php
class Util {
    final public static function helper() {
        return "helped";
    }
}

echo Util::helper();
--EXPECT--
helped
```

**tests/classes/final_private_method.vhpt**
```
--TEST--
Final private method (allowed but redundant)
--FILE--
<?php
class Base {
    final private function internal() {
        return "internal";
    }
    
    public function callInternal() {
        return $this->internal();
    }
}

$b = new Base();
echo $b->callInternal();
--EXPECT--
internal
```

## Key Rules

1. **Final classes**: Cannot be extended by any class
2. **Final methods**: Cannot be overridden by child classes
3. **Final constants** (PHP 8.1+): Cannot be overridden by child classes
4. **Conflicts**: `final` and `abstract` cannot be used together
5. **Private methods**: Can technically be final but it's redundant
6. **Constructors**: Can be final (prevents child from changing construction)

## PHP Compatibility Notes

| Feature | PHP Version |
|---------|-------------|
| `final` class | 5.0 |
| `final` method | 5.0 |
| `final` constant | 8.1 |
| Combining with `static` | 5.0 |

## Error Messages

Match PHP error format:
- `Class X cannot extend final class Y`
- `Cannot override final method X::Y()`
- `Cannot override final constant X::Y`
- `Cannot use the final modifier on an abstract class`
- `Cannot use the abstract modifier on a final method`

## Implementation Order

1. Token and lexer
2. Final classes first (simpler validation)
3. Final methods
4. Final constants
5. All validation rules
6. Tests for each feature
