# Plan: Asymmetric Visibility (PHP 8.4)

## Overview

Asymmetric visibility allows different visibility modifiers for reading and writing properties. This enables the common pattern of public read access but restricted write access (e.g., `public private(set) $property`).

**PHP Example:**
```php
<?php
// Before (PHP < 8.4): Had to use getter methods
class User {
    private string $name;

    public function __construct(string $name) {
        $this->name = $name;
    }

    public function getName(): string {
        return $this->name;
    }
}

// After (PHP 8.4): Direct property access with asymmetric visibility
class User {
    public private(set) string $name;

    public function __construct(string $name) {
        $this->name = $name;
    }
}

$user = new User("Alice");
echo $user->name; // OK: public read
$user->name = "Bob"; // Error: private write
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/ast/stmt.rs` | Add `write_visibility` field to `Property` struct |
| `src/parser/stmt.rs` | Parse asymmetric visibility syntax |
| `src/interpreter/objects/properties.rs` | Check write visibility on assignment |
| `tests/classes/asymmetric_*.vhpt` | Test files |

## Implementation Steps

### Step 1: Update Property AST (`src/ast/stmt.rs`)

Find the `Property` struct (around line 146-155) and add `write_visibility`:

```rust
/// Class property definition
#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    pub visibility: Visibility,            // Read visibility
    pub write_visibility: Option<Visibility>, // Write visibility (PHP 8.4+), None means same as read
    pub default: Option<Expr>,
    pub readonly: bool,                    // PHP 8.1+
    pub is_static: bool,                   // PHP 5.0+ (if implemented)
    pub attributes: Vec<Attribute>,        // PHP 8.0+
    pub hooks: Vec<PropertyHook>,          // PHP 8.4+
}
```

### Step 2: Parse Asymmetric Visibility (`src/parser/stmt.rs`)

Update property parsing to handle the `visibility(set)` syntax. Find the property parsing logic (around line 600-700):

```rust
fn parse_class_property(
    &mut self,
    visibility: Visibility,
    attributes: Vec<Attribute>,
) -> Result<Property, String> {
    // Parse write visibility if present
    let write_visibility = if self.check(&TokenKind::LeftParen) {
        self.advance(); // consume '('

        // Expect 'set' keyword
        if !self.check_identifier("set") {
            return Err("Expected 'set' after '(' in asymmetric visibility".to_string());
        }
        self.advance(); // consume 'set'

        self.expect(&TokenKind::RightParen)?;

        // The write visibility is the already-parsed visibility
        // The read visibility will be parsed next
        Some(visibility)
    } else {
        None
    };

    // If we have write_visibility, we need to parse the read visibility now
    let read_visibility = if write_visibility.is_some() {
        self.parse_visibility()?
    } else {
        visibility
    };

    // Check for 'static' keyword
    let is_static = if self.check(&TokenKind::Static) {
        self.advance();
        true
    } else {
        false
    };

    // Check for 'readonly' keyword
    let readonly = if self.check_identifier("readonly") {
        self.advance();
        true
    } else {
        false
    };

    // Validation: readonly and asymmetric visibility are incompatible
    if readonly && write_visibility.is_some() {
        return Err(
            "Readonly properties cannot have asymmetric visibility".to_string()
        );
    }

    // Parse type hint if present
    let type_hint = if self.is_type_start() {
        Some(self.parse_type_hint()?)
    } else {
        None
    };

    // Parse property name: $name
    self.expect_variable()?;
    let name = if let TokenKind::Variable(n) = &self.previous().kind {
        n.clone()
    } else {
        return Err("Expected property name".to_string());
    };

    // Parse optional default value
    let default = if self.check(&TokenKind::Assign) {
        self.advance();
        Some(self.parse_expression()?)
    } else {
        None
    };

    // Parse property hooks if present (PHP 8.4)
    let hooks = if self.check(&TokenKind::LeftBrace) {
        self.parse_property_hooks()?
    } else {
        vec![]
    };

    if hooks.is_empty() {
        self.expect(&TokenKind::Semicolon)?;
    }

    Ok(Property {
        name,
        visibility: read_visibility,
        write_visibility,
        default,
        readonly,
        is_static,
        attributes,
        hooks,
    })
}
```

**Parse visibility helper** (if not already present):

```rust
fn parse_visibility(&mut self) -> Result<Visibility, String> {
    match &self.current_token().kind {
        TokenKind::Public => {
            self.advance();
            Ok(Visibility::Public)
        }
        TokenKind::Protected => {
            self.advance();
            Ok(Visibility::Protected)
        }
        TokenKind::Private => {
            self.advance();
            Ok(Visibility::Private)
        }
        _ => Err("Expected visibility modifier (public, protected, or private)".to_string()),
    }
}
```

### Step 3: Check Write Visibility on Assignment (`src/interpreter/objects/properties.rs`)

Update property assignment to check write visibility. Find the property assignment logic:

```rust
pub(crate) fn set_property(
    &mut self,
    object: &Rc<RefCell<Object>>,
    property: &str,
    value: Value,
) -> Result<(), String> {
    let mut obj = object.borrow_mut();

    // Get property definition from class
    let class_def = self.classes
        .get(&obj.class_name.to_lowercase())
        .ok_or_else(|| format!("Class '{}' not found", obj.class_name))?
        .clone();

    // Find property definition
    if let Some(prop_def) = class_def.properties.iter().find(|p| p.name == property) {
        // Check write visibility
        let write_vis = prop_def.write_visibility.unwrap_or(prop_def.visibility);

        // Determine the context for visibility check
        let can_write = match write_vis {
            Visibility::Public => true,
            Visibility::Protected => {
                // Can write if we're in the same class or a subclass
                self.current_class
                    .as_ref()
                    .map(|current| {
                        current.eq_ignore_ascii_case(&obj.class_name)
                            || self.is_subclass(current, &obj.class_name)
                    })
                    .unwrap_or(false)
            }
            Visibility::Private => {
                // Can write only if we're in the same class
                self.current_class
                    .as_ref()
                    .map(|current| current.eq_ignore_ascii_case(&obj.class_name))
                    .unwrap_or(false)
            }
        };

        if !can_write {
            return Err(format!(
                "Cannot modify {} property {}::${}",
                match write_vis {
                    Visibility::Public => "public",
                    Visibility::Protected => "protected",
                    Visibility::Private => "private",
                },
                obj.class_name,
                property
            ));
        }

        // Check readonly
        if prop_def.readonly && obj.properties.contains_key(property) {
            return Err(format!(
                "Cannot modify readonly property {}::${}",
                obj.class_name, property
            ));
        }
    }

    // Set the property value
    obj.properties.insert(property.to_string(), value);
    Ok(())
}
```

### Step 4: Add Tests

**tests/classes/asymmetric_basic.vhpt**
```
--TEST--
Basic asymmetric visibility
--FILE--
<?php
class User {
    public private(set) string $name;

    public function __construct(string $name) {
        $this->name = $name;
    }
}

$user = new User("Alice");
echo $user->name;
--EXPECT--
Alice
```

**tests/classes/asymmetric_write_error.vhpt**
```
--TEST--
Cannot write to property with private(set)
--FILE--
<?php
class User {
    public private(set) string $name;

    public function __construct(string $name) {
        $this->name = $name;
    }
}

$user = new User("Alice");
$user->name = "Bob";
--EXPECT_ERROR--
Cannot modify private property
```

**tests/classes/asymmetric_protected.vhpt**
```
--TEST--
Asymmetric visibility with protected(set)
--FILE--
<?php
class Base {
    public protected(set) int $count = 0;
}

class Child extends Base {
    public function increment() {
        $this->count++; // OK: protected write in subclass
    }
}

$child = new Child();
echo $child->count . "\n"; // OK: public read
$child->increment();
echo $child->count;
--EXPECT--
0
1
```

**tests/classes/asymmetric_protected_error.vhpt**
```
--TEST--
Cannot write to protected(set) from outside
--FILE--
<?php
class Base {
    public protected(set) int $count = 0;
}

$obj = new Base();
$obj->count = 5;
--EXPECT_ERROR--
Cannot modify protected property
```

**tests/classes/asymmetric_in_class.vhpt**
```
--TEST--
Asymmetric property can be written inside class
--FILE--
<?php
class Counter {
    public private(set) int $value = 0;

    public function add(int $n) {
        $this->value += $n; // OK: private write inside class
    }

    public function getValue(): int {
        return $this->value;
    }
}

$counter = new Counter();
$counter->add(5);
echo $counter->getValue();
--EXPECT--
5
```

**tests/classes/asymmetric_static.vhpt**
```
--TEST--
Asymmetric visibility on static properties
--FILE--
<?php
class Config {
    public private(set) static int $version = 1;

    public static function upgrade() {
        self::$version++; // OK: write inside class
    }
}

echo Config::$version . "\n"; // OK: public read
Config::upgrade();
echo Config::$version;
--EXPECT--
1
2
```

**tests/classes/asymmetric_static_error.vhpt**
```
--TEST--
Cannot write to static property with private(set) from outside
--FILE--
<?php
class Config {
    public private(set) static int $version = 1;
}

Config::$version = 2;
--EXPECT_ERROR--
Cannot modify private property
```

**tests/classes/asymmetric_readonly_error.vhpt**
```
--TEST--
Readonly and asymmetric visibility are incompatible
--FILE--
<?php
class User {
    public private(set) readonly string $name;
}
--EXPECT_ERROR--
Readonly properties cannot have asymmetric visibility
```

**tests/classes/asymmetric_multiple_properties.vhpt**
```
--TEST--
Multiple properties with different asymmetric visibility
--FILE--
<?php
class Data {
    public private(set) int $id;
    public protected(set) string $name;
    public int $value; // symmetric

    public function __construct(int $id, string $name, int $value) {
        $this->id = $id;
        $this->name = $name;
        $this->value = $value;
    }
}

$data = new Data(1, "test", 42);
echo $data->id . "\n";
echo $data->name . "\n";
echo $data->value;
--EXPECT--
1
test
42
```

**tests/classes/asymmetric_inheritance.vhpt**
```
--TEST--
Asymmetric visibility with inheritance
--FILE--
<?php
class Base {
    public protected(set) string $status = "pending";
}

class Child extends Base {
    public function approve() {
        $this->status = "approved"; // OK in subclass
    }
}

$child = new Child();
echo $child->status . "\n";
$child->approve();
echo $child->status;
--EXPECT--
pending
approved
```

## PHP Compatibility Notes

| Feature | PHP Version |
|---------|-------------|
| Asymmetric visibility | 8.4 |
| Syntax: `public private(set)` | 8.4 |
| Syntax: `public protected(set)` | 8.4 |
| Works with static properties | 8.4 |

## Key Considerations

1. **Syntax**: `<read-visibility> <write-visibility>(set)` where write must be more restrictive
2. **Validation**: Write visibility must be equal to or more restrictive than read visibility:
   - `public private(set)` ✓
   - `public protected(set)` ✓
   - `protected private(set)` ✓
   - `private public(set)` ✗ (invalid)
3. **Readonly incompatibility**: Cannot combine with readonly (they solve the same problem)
4. **Property hooks incompatibility**: Cannot combine with set hooks (they control writes)
5. **Static properties**: Works with static properties
6. **Inheritance**: Write visibility is inherited and respected
7. **Context**: Write visibility checked based on current class context

## Validation Rules

When parsing asymmetric visibility:

```rust
// Validate that write visibility is more restrictive than read
fn validate_asymmetric_visibility(
    read: Visibility,
    write: Visibility,
) -> Result<(), String> {
    let valid = match (read, write) {
        (Visibility::Public, Visibility::Public) => false,    // No point
        (Visibility::Public, Visibility::Protected) => true,
        (Visibility::Public, Visibility::Private) => true,
        (Visibility::Protected, Visibility::Protected) => false, // No point
        (Visibility::Protected, Visibility::Private) => true,
        (Visibility::Private, _) => false, // Private is most restrictive
        _ => false,
    };

    if !valid {
        Err("Write visibility must be more restrictive than read visibility".to_string())
    } else {
        Ok(())
    }
}
```

## Error Messages

- `Cannot modify <visibility> property <Class>::$<property>`
- `Readonly properties cannot have asymmetric visibility`
- `Write visibility must be more restrictive than read visibility`
- `Property hooks cannot be combined with asymmetric visibility`
- `Expected 'set' after '(' in asymmetric visibility`

## Implementation Order

1. Add `write_visibility` field to Property AST
2. Parse asymmetric visibility syntax
3. Validate visibility combinations
4. Update property assignment to check write visibility
5. Add tests for all scenarios
6. Update error messages

## Edge Cases

1. **Constructor property promotion**: Can have asymmetric visibility
2. **Property hooks**: Incompatible with asymmetric visibility (conflict)
3. **Readonly**: Incompatible with asymmetric visibility (redundant)
4. **Reflection**: Reflection should expose both read and write visibility
5. **Trait properties**: Can have asymmetric visibility
6. **Magic methods**: `__set()` should still be called if property doesn't exist

## Reference Implementations

- Property parsing: `src/parser/stmt.rs` (existing property parsing)
- Property assignment: `src/interpreter/objects/properties.rs`
- Visibility checking: Already implemented for instance properties
- Readonly properties: Similar pattern for write restrictions
