# Plan: Static Properties and Late Static Binding (PHP 5.3+)

## Overview

Static properties are class-level variables shared across all instances, accessed via `ClassName::$property` or `self::$property`. Late static binding (LSB) with the `static::` keyword allows referencing the called class rather than the defined class, enabling proper inheritance behavior.

**PHP Example:**
```php
<?php
// Basic static properties
class Counter {
    public static $count = 0;

    public function __construct() {
        self::$count++;
    }

    public static function getCount() {
        return self::$count;
    }
}

$a = new Counter();
$b = new Counter();
echo Counter::$count; // 2

// Late static binding
class Animal {
    protected static $type = "Generic";

    public static function getType() {
        return static::$type; // LSB: refers to called class
    }

    public static function describe() {
        return self::getType(); // Calls getType() via LSB
    }
}

class Dog extends Animal {
    protected static $type = "Canine";
}

echo Dog::getType(); // "Canine" (via static::)
echo Animal::getType(); // "Generic"
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/token.rs` | Add `Static` keyword token (may already exist) |
| `src/ast/stmt.rs` | Add `is_static` field to `Property` struct |
| `src/ast/expr.rs` | Add `StaticPropertyAccess` expression variant |
| `src/parser/stmt.rs` | Parse `static` keyword for properties |
| `src/parser/expr/primary.rs` | Parse `ClassName::$property` syntax |
| `src/interpreter/mod.rs` | Add `static_properties: HashMap<String, HashMap<String, Value>>` to Interpreter |
| `src/interpreter/objects/properties.rs` | Handle static property access and assignment |
| `src/interpreter/stmt_exec/definitions.rs` | Initialize static properties when class is defined |
| `tests/classes/static_*.vhpt` | Test files |

## Implementation Steps

### Step 1: Verify Static Keyword Token (`src/token.rs`)

The `Static` token should already exist for return types. Verify it's present:

```rust
pub enum TokenKind {
    // ... existing tokens ...
    Static,  // Should already exist
    // ...
}
```

If not present, add it and update the lexer in `src/lexer/mod.rs`:

```rust
// In keyword matching (around line 200-300)
match ident.to_lowercase().as_str() {
    // ... existing keywords ...
    "static" => TokenKind::Static,
    // ...
}
```

### Step 2: Add is_static to Property (`src/ast/stmt.rs`)

Update the `Property` struct (around line 122):

```rust
/// Class property definition
#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    pub visibility: Visibility,
    pub default: Option<Expr>,
    pub readonly: bool,             // PHP 8.1+
    pub is_static: bool,            // NEW: PHP 5.0+
    pub attributes: Vec<Attribute>, // PHP 8.0+
}
```

### Step 3: Add StaticPropertyAccess Expression (`src/ast/expr.rs`)

Add new expression variant (around line 140-160, near `StaticMethodCall`):

```rust
#[derive(Debug, Clone)]
pub enum Expr {
    // ... existing variants ...

    /// Static property access: ClassName::$property or static::$property
    StaticPropertyAccess {
        class: String, // Can be "self", "parent", or "static" for LSB
        property: String,
    },

    // ... rest of variants ...
}
```

### Step 4: Parse Static Properties in Class Declaration (`src/parser/stmt.rs`)

Update property parsing to handle `static` modifier. Find the property parsing logic (likely in `parse_class_member` or similar, around line 400-600):

```rust
fn parse_class_property(&mut self, visibility: Visibility, attributes: Vec<Attribute>) -> Result<Property, String> {
    // Check for 'static' keyword
    let is_static = if self.check(&TokenKind::Static) {
        self.advance();
        true
    } else {
        false
    };

    // Check for 'readonly' keyword (existing logic)
    let readonly = if self.check_identifier("readonly") {
        self.advance();
        true
    } else {
        false
    };

    // Parse property name: $name
    self.expect(&TokenKind::Variable)?;
    let name = if let TokenKind::Variable(n) = &self.previous().kind {
        n.clone()
    } else {
        return Err("Expected property name".to_string());
    };

    // Parse optional default value: = expr
    let default = if self.check(&TokenKind::Equals) {
        self.advance();
        Some(self.parse_expression()?)
    } else {
        None
    };

    self.expect(&TokenKind::Semicolon)?;

    Ok(Property {
        name,
        visibility,
        default,
        readonly,
        is_static,
        attributes,
    })
}
```

**Important**: Handle the order of modifiers:
- `public static $prop`
- `static public $prop`
- `private static readonly $prop` (PHP 8.1+)
- All variations should be supported

### Step 5: Parse Static Property Access (`src/parser/expr/primary.rs`)

Update the `::` parsing logic to handle both methods and properties. Find where `StaticMethodCall` is parsed (around line 200-300):

```rust
// After parsing ClassName::
if self.check(&TokenKind::Variable) {
    // Static property access: ClassName::$property
    self.advance();
    let property = if let TokenKind::Variable(name) = &self.previous().kind {
        name.clone()
    } else {
        return Err("Expected property name after ::$".to_string());
    };

    return Ok(Expr::StaticPropertyAccess {
        class: class_name,
        property,
    });
} else if self.check(&TokenKind::Identifier) {
    // Static method call: ClassName::method()
    // ... existing StaticMethodCall parsing ...
}
```

**Handle self, parent, and static keywords**:

```rust
// When parsing before ::
let class_name = match &self.current_token().kind {
    TokenKind::Identifier(name) => name.clone(),
    TokenKind::Static => {
        self.advance();
        "static".to_string() // Special marker for LSB
    }
    // Note: 'self' and 'parent' should be TokenKind::Identifier
    _ => return Err("Expected class name".to_string()),
};
```

### Step 6: Store Static Properties in Interpreter (`src/interpreter/mod.rs`)

Add static properties storage to the `Interpreter` struct (around line 140-180):

```rust
pub struct Interpreter<W: Write> {
    // ... existing fields ...

    /// Static properties: class_name_lowercase -> property_name -> value
    /// Initialized when class is defined, shared across all instances
    static_properties: HashMap<String, HashMap<String, Value>>,

    // ... rest of fields ...
}
```

Initialize in `new()` method:

```rust
pub fn new(output: W) -> Self {
    Self {
        // ... existing fields ...
        static_properties: HashMap::new(),
        // ...
    }
}
```

### Step 7: Initialize Static Properties When Class is Defined (`src/interpreter/stmt_exec/definitions.rs`)

Find where classes are stored (in `execute_class_declaration` or similar, around line 50-150):

```rust
// After storing the class definition in self.classes
fn execute_class_declaration(&mut self, class: &ClassDecl) -> Result<(), String> {
    // ... existing class storage logic ...

    // Initialize static properties for this class
    let class_key = class.name.to_lowercase();
    let mut static_props = HashMap::new();

    for prop in &class.properties {
        if prop.is_static {
            // Evaluate default value
            let value = if let Some(default_expr) = &prop.default {
                self.evaluate(default_expr)?
            } else {
                Value::Null
            };

            static_props.insert(prop.name.clone(), value);
        }
    }

    // Store static properties
    if !static_props.is_empty() {
        self.static_properties.insert(class_key, static_props);
    }

    Ok(())
}
```

**Note**: Static properties inherit from parent classes. Update to merge parent's static properties:

```rust
// Initialize static properties, including inherited ones
let class_key = class.name.to_lowercase();
let mut static_props = HashMap::new();

// First, copy parent's static properties if there's inheritance
if let Some(parent_name) = &class.parent {
    let parent_key = parent_name.to_lowercase();
    if let Some(parent_statics) = self.static_properties.get(&parent_key) {
        static_props.extend(parent_statics.clone());
    }
}

// Then add/override with this class's static properties
for prop in &class.properties {
    if prop.is_static {
        let value = if let Some(default_expr) = &prop.default {
            self.evaluate(default_expr)?
        } else {
            Value::Null
        };
        static_props.insert(prop.name.clone(), value);
    }
}

self.static_properties.insert(class_key, static_props);
```

### Step 8: Implement Static Property Access (`src/interpreter/expr_eval/mod.rs`)

Add evaluation logic in the main `evaluate()` match statement (around line 80-200):

```rust
Expr::StaticPropertyAccess { class, property } => {
    self.get_static_property(class, property)
}
```

Implement the helper in `src/interpreter/objects/properties.rs`:

```rust
impl<W: Write> Interpreter<W> {
    /// Get static property value
    pub(crate) fn get_static_property(
        &mut self,
        class: &str,
        property: &str,
    ) -> Result<Value, String> {
        // Resolve class name (handle self, parent, static)
        let resolved_class = self.resolve_static_class_name(class)?;

        let class_key = resolved_class.to_lowercase();

        // Get static properties for this class
        let static_props = self
            .static_properties
            .get(&class_key)
            .ok_or_else(|| format!("Class '{}' not found", resolved_class))?;

        // Get the property value
        static_props
            .get(property)
            .cloned()
            .ok_or_else(|| {
                format!(
                    "Access to undeclared static property {}::${}",
                    resolved_class, property
                )
            })
    }

    /// Set static property value
    pub(crate) fn set_static_property(
        &mut self,
        class: &str,
        property: &str,
        value: Value,
    ) -> Result<(), String> {
        // Resolve class name (handle self, parent, static)
        let resolved_class = self.resolve_static_class_name(class)?;

        let class_key = resolved_class.to_lowercase();

        // Get mutable reference to static properties
        let static_props = self
            .static_properties
            .get_mut(&class_key)
            .ok_or_else(|| format!("Class '{}' not found", resolved_class))?;

        // Check if property exists (PHP doesn't allow creating new static props at runtime)
        if !static_props.contains_key(property) {
            return Err(format!(
                "Access to undeclared static property {}::${}",
                resolved_class, property
            ));
        }

        // Set the value
        static_props.insert(property.to_string(), value);
        Ok(())
    }

    /// Resolve class name for static context
    /// Handles "self", "parent", and "static" (late static binding)
    fn resolve_static_class_name(&self, class: &str) -> Result<String, String> {
        match class.to_lowercase().as_str() {
            "self" => {
                // Return the current class context
                self.current_class
                    .clone()
                    .ok_or_else(|| "Cannot use 'self' outside of class context".to_string())
            }
            "parent" => {
                // Return the parent class
                let current = self.current_class
                    .as_ref()
                    .ok_or_else(|| "Cannot use 'parent' outside of class context".to_string())?;

                let class_def = self.classes
                    .get(&current.to_lowercase())
                    .ok_or_else(|| format!("Current class '{}' not found", current))?;

                class_def.parent
                    .clone()
                    .ok_or_else(|| "Cannot use 'parent' in class with no parent".to_string())
            }
            "static" => {
                // Late static binding: return the called class, not the defined class
                // This requires tracking the "called class" in the call stack
                self.called_class
                    .clone()
                    .ok_or_else(|| "Cannot use 'static' outside of class context".to_string())
            }
            _ => {
                // Regular class name
                Ok(class.to_string())
            }
        }
    }
}
```

### Step 9: Implement Assignment to Static Properties

Handle assignment in the interpreter. Find where assignments are handled (likely in `stmt_exec/mod.rs` or `expr_eval/mod.rs`):

```rust
// In assignment handling
fn execute_assignment(&mut self, target: &Expr, value: Value) -> Result<(), String> {
    match target {
        Expr::StaticPropertyAccess { class, property } => {
            self.set_static_property(class, property, value)?;
            Ok(())
        }
        // ... existing cases (Variable, PropertyAccess, ArrayAccess) ...
        _ => Err("Invalid assignment target".to_string()),
    }
}
```

### Step 10: Add Late Static Binding Context (`src/interpreter/mod.rs`)

Add fields to track the "called class" for LSB (around line 140-180):

```rust
pub struct Interpreter<W: Write> {
    // ... existing fields ...

    /// Current class context (for 'self' and 'parent')
    current_class: Option<String>,

    /// Called class context (for 'static' - late static binding)
    called_class: Option<String>,

    // ... rest of fields ...
}
```

Initialize in `new()`:

```rust
pub fn new(output: W) -> Self {
    Self {
        // ... existing fields ...
        current_class: None,
        called_class: None,
        // ...
    }
}
```

Update method call handling in `src/interpreter/objects/methods.rs` to set `called_class`:

```rust
// When calling a static method
pub(crate) fn call_static_method(
    &mut self,
    class_name: &str,
    method_name: &str,
    args: &[Expr],
) -> Result<Value, String> {
    // Save previous called_class
    let prev_called_class = self.called_class.clone();

    // Set called_class for late static binding
    self.called_class = Some(class_name.to_string());

    // ... execute the method ...

    // Restore called_class
    self.called_class = prev_called_class;

    // Return result
}
```

### Step 11: Add Tests

**tests/classes/static_property_basic.vhpt**
```
--TEST--
Basic static property access
--FILE--
<?php
class Counter {
    public static $count = 0;
}

Counter::$count = 5;
echo Counter::$count;
--EXPECT--
5
```

**tests/classes/static_property_increment.vhpt**
```
--TEST--
Static property shared across instances
--FILE--
<?php
class Counter {
    public static $count = 0;

    public function increment() {
        self::$count++;
    }
}

$a = new Counter();
$b = new Counter();
$a->increment();
$b->increment();
echo Counter::$count;
--EXPECT--
2
```

**tests/classes/static_property_self.vhpt**
```
--TEST--
Static property access with self::
--FILE--
<?php
class Config {
    private static $debug = false;

    public static function setDebug($value) {
        self::$debug = $value;
    }

    public static function isDebug() {
        return self::$debug;
    }
}

Config::setDebug(true);
echo Config::isDebug() ? "yes" : "no";
--EXPECT--
yes
```

**tests/classes/static_property_inheritance.vhpt**
```
--TEST--
Static property inheritance
--FILE--
<?php
class Base {
    protected static $value = "base";

    public static function getValue() {
        return self::$value;
    }
}

class Child extends Base {
    protected static $value = "child";
}

echo Base::getValue() . "\n";
echo Child::getValue();
--EXPECT--
base
child
```

**tests/classes/static_late_binding.vhpt**
```
--TEST--
Late static binding with static::
--FILE--
<?php
class Animal {
    protected static $type = "Animal";

    public static function getType() {
        return static::$type; // LSB
    }
}

class Dog extends Animal {
    protected static $type = "Dog";
}

echo Animal::getType() . "\n";
echo Dog::getType();
--EXPECT--
Animal
Dog
```

**tests/classes/static_vs_self.vhpt**
```
--TEST--
Difference between self:: and static::
--FILE--
<?php
class Base {
    protected static $name = "Base";

    public static function withSelf() {
        return self::$name;
    }

    public static function withStatic() {
        return static::$name;
    }
}

class Child extends Base {
    protected static $name = "Child";
}

echo "self: " . Child::withSelf() . "\n";
echo "static: " . Child::withStatic();
--EXPECT--
self: Base
static: Child
```

**tests/classes/static_property_visibility.vhpt**
```
--TEST--
Static property with visibility modifiers
--FILE--
<?php
class Secret {
    private static $password = "secret123";
    protected static $token = "abc";
    public static $api_key = "xyz";
}

echo Secret::$api_key;
--EXPECT--
xyz
```

**tests/classes/static_property_undefined.vhpt**
```
--TEST--
Error on accessing undefined static property
--FILE--
<?php
class Test {
    public static $exists = 1;
}

echo Test::$missing;
--EXPECT_ERROR--
Access to undeclared static property
```

**tests/classes/static_property_initialization.vhpt**
```
--TEST--
Static property with complex initialization
--FILE--
<?php
class Math {
    public static $pi = 3.14159;
    public static $doubled;
}

Math::$doubled = Math::$pi * 2;
echo Math::$doubled;
--EXPECT--
6.28318
```

**tests/classes/static_property_array.vhpt**
```
--TEST--
Static property with array value
--FILE--
<?php
class Collection {
    public static $items = [];
}

Collection::$items[] = "a";
Collection::$items[] = "b";
echo count(Collection::$items);
--EXPECT--
2
```

**tests/classes/static_readonly.vhpt**
```
--TEST--
Static readonly property (PHP 8.3+)
--FILE--
<?php
class Config {
    public static readonly $version = "1.0.0";
}

echo Config::$version . "\n";
Config::$version = "2.0.0";
--EXPECT_ERROR--
Cannot modify readonly property
```

## PHP Compatibility Notes

| Feature | PHP Version |
|---------|-------------|
| Basic static properties | 5.0 |
| Late static binding (`static::`) | 5.3 |
| Static property inheritance | 5.0 |
| Static readonly properties | 8.3 (in progress) |

## Key Considerations

1. **Shared State**: Static properties are shared across all instances of a class
2. **Inheritance**: Child classes inherit parent's static properties but have separate storage
3. **Late Static Binding**: `static::` refers to the called class, `self::` to the defined class
4. **Initialization**: Static properties are initialized when the class is defined, not instantiated
5. **Visibility**: Static properties respect visibility modifiers (public/protected/private)
6. **Context Tracking**: LSB requires tracking both current_class and called_class
7. **Property Declaration**: PHP doesn't allow creating undeclared static properties at runtime

## Implementation Order

1. Add `is_static` to Property AST
2. Parse static property declarations
3. Parse static property access expressions
4. Store and initialize static properties in interpreter
5. Implement get/set operations
6. Add context tracking for self/parent resolution
7. Implement late static binding (static::)
8. Add tests for all scenarios

## Edge Cases

1. **Accessing undeclared property**: Should error
2. **Creating new static property at runtime**: Not allowed in PHP
3. **Static property in trait**: Allowed, but with special inheritance rules
4. **Static property and instance property with same name**: Allowed, separate storage
5. **Readonly static**: PHP 8.3+ feature, may need special handling
6. **Array operations on static properties**: `Counter::$items[] = 1` should work
7. **self:: in static method**: Should work
8. **static:: in instance method**: Should use the object's class

## Reference Implementations

- Instance properties: `src/interpreter/objects/properties.rs`
- Static methods: `src/interpreter/objects/methods.rs` (already implemented)
- Class definitions: `src/interpreter/stmt_exec/definitions.rs`
