# Plan: Readonly Properties (PHP 8.1)

## Overview

Readonly properties were introduced in PHP 8.1 and provide immutability guarantees at the property level. A readonly property can only be initialized once (typically in the constructor) and cannot be modified after initialization. This feature is essential for creating immutable value objects and preventing accidental modifications to critical data.

In PHP 8.2, the `readonly` modifier was extended to entire classes, but this plan focuses on PHP 8.1's property-level readonly functionality.

**PHP Example:**
```php
// Before (without readonly)
class Point {
    public function __construct(
        public float $x,
        public float $y
    ) {}

    // Risk: properties can be modified accidentally
}

$p = new Point(1.0, 2.0);
$p->x = 5.0; // Modifies x

// After (with readonly)
class Point {
    public function __construct(
        public readonly float $x,
        public readonly float $y
    ) {}
}

$p = new Point(1.0, 2.0);
$p->x = 5.0; // Fatal error: Cannot modify readonly property
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/token.rs` | Add `Readonly` token |
| `src/lexer.rs` | Recognize `readonly` keyword |
| `src/ast/stmt.rs` | Add `readonly` field to `Property` struct |
| `src/parser/stmt.rs` | Parse `readonly` modifier in properties and constructor promotion |
| `src/interpreter/mod.rs` | Track property initialization and enforce readonly semantics |
| `src/interpreter/value.rs` | Add readonly tracking to `ObjectInstance` |
| `tests/classes/readonly_*.vhpt` | Create comprehensive test suite |
| `docs/roadmap.md` | Mark feature as complete |
| `AGENTS.md` | Update feature list and roadmap |
| `README.md` | Document readonly properties |

## Implementation Steps

### Step 1: Add Token (`src/token.rs`)

Add the `Readonly` token variant to the `TokenKind` enum.

**Location:** After line 42 (after `Insteadof`)

```rust
    Readonly,     // readonly
```

### Step 2: Update Lexer (`src/lexer.rs`)

Add recognition for the `readonly` keyword in the lexer's identifier matching logic.

**Location:** Around line 201 (in the keyword match statement, after `"insteadof"`)

```rust
            "readonly" => TokenKind::Readonly,
```

### Step 3: Extend AST (`src/ast/stmt.rs`)

Add a `readonly` boolean field to the `Property` struct to track whether a property is readonly.

**Location:** Modify the `Property` struct (around line 11-18)

**Current code:**
```rust
#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    #[allow(dead_code)] // Will be used for visibility enforcement
    pub visibility: Visibility,
    pub default: Option<Expr>,
}
```

**Updated code:**
```rust
#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    #[allow(dead_code)] // Will be used for visibility enforcement
    pub visibility: Visibility,
    pub default: Option<Expr>,
    pub readonly: bool, // PHP 8.1+
}
```

### Step 4: Update Parser (`src/parser/stmt.rs`)

#### 4a. Parse `readonly` modifier in property declarations

**Location:** Modify `parse_property` function (around line 570-599)

**Current function signature and initial code:**
```rust
fn parse_property(&mut self, visibility: Visibility) -> Result<Property, String> {
```

**Updates needed:**

1. After determining visibility, check for `readonly` keyword
2. Update the Property construction to include the `readonly` field

**Complete updated function:**
```rust
fn parse_property(&mut self, visibility: Visibility) -> Result<Property, String> {
    // Check for readonly modifier
    let readonly = if self.check(&TokenKind::Readonly) {
        self.advance();
        true
    } else {
        false
    };

    let name = if let TokenKind::Variable(name) = &self.current().kind {
        let name = name.clone();
        self.advance();
        name
    } else {
        return Err(format!(
            "Expected property name at line {}, column {}",
            self.current().line,
            self.current().column
        ));
    };

    let default = if self.check(&TokenKind::Assign) {
        self.advance();
        Some(self.parse_expression(Precedence::None)?)
    } else {
        None
    };

    // Readonly properties without default values must be uninitialized
    // (will be initialized in constructor)

    if self.check(&TokenKind::Semicolon) {
        self.advance();
    }

    Ok(Property {
        name,
        visibility,
        default,
        readonly,
    })
}
```

#### 4b. Parse `readonly` in constructor property promotion

**Location:** Find the `parse_function_param` function and update it to handle readonly in constructor context

When parsing constructor parameters with visibility (property promotion), we need to handle the `readonly` modifier.

**Location:** Around line 733-800 in `parse_function_param`

Look for the section that handles visibility in constructor parameters. You'll need to:

1. After parsing visibility, check for `readonly` keyword
2. Pass this information through the `FunctionParam` struct

However, since `FunctionParam` already has a `visibility` field for property promotion, we need to add a `readonly` field to it.

**First, update `src/ast/stmt.rs` FunctionParam struct (around line 145-155):**

**Current code:**
```rust
#[derive(Debug, Clone)]
pub struct FunctionParam {
    pub name: String,
    pub default: Option<Expr>,
    /// By-reference parameter (will be used when reference semantics are implemented)
    #[allow(dead_code)]
    pub by_ref: bool,
    /// Visibility for constructor property promotion (PHP 8.0)
    pub visibility: Option<Visibility>,
}
```

**Updated code:**
```rust
#[derive(Debug, Clone)]
pub struct FunctionParam {
    pub name: String,
    pub default: Option<Expr>,
    /// By-reference parameter (will be used when reference semantics are implemented)
    #[allow(dead_code)]
    pub by_ref: bool,
    /// Visibility for constructor property promotion (PHP 8.0)
    pub visibility: Option<Visibility>,
    /// Readonly modifier for constructor property promotion (PHP 8.1)
    pub readonly: bool,
}
```

**Then update parser at parse_function_param (around line 733+):**

Find where visibility is parsed for constructor parameters and add readonly parsing after it:

```rust
// After visibility is determined, check for readonly
let readonly = if self.check(&TokenKind::Readonly) {
    self.advance();
    true
} else {
    false
};
```

And update the `FunctionParam` construction to include `readonly`:

```rust
Ok(FunctionParam {
    name,
    default,
    by_ref,
    visibility,
    readonly,
})
```

#### 4c. Update class parsing to handle readonly before visibility

**Location:** In `parse_class` function (around line 380-500)

When parsing class members, we need to handle the case where `readonly` appears before the visibility modifier.

Find the section that parses visibility modifiers in the class body and update it:

**Add after checking for visibility modifiers:**

```rust
// Check for readonly modifier (can come before or after visibility)
let readonly_first = if self.check(&TokenKind::Readonly) {
    self.advance();
    true
} else {
    false
};

// Then check visibility
let visibility = if self.check(&TokenKind::Public)
    || self.check(&TokenKind::Private)
    || self.check(&TokenKind::Protected)
{
    self.parse_visibility()
} else {
    Visibility::Public
};

// Check for readonly after visibility (if not already found)
let readonly = readonly_first || if self.check(&TokenKind::Readonly) {
    self.advance();
    true
} else {
    false
};
```

But actually, to keep this simple and match PHP's behavior, `readonly` should come AFTER visibility:
- Valid: `public readonly string $name`
- Invalid: `readonly public string $name`

So the current implementation in Step 4a is correct - just check for `readonly` after getting visibility.

### Step 5: Update Interpreter (`src/interpreter/mod.rs`)

#### 5a. Track initialized readonly properties

**Location:** Modify `ObjectInstance` in `src/interpreter/value.rs` (around line 80-100)

Add a field to track which properties have been initialized (needed for readonly enforcement):

**Find ObjectInstance struct:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectInstance {
    pub class_name: String,
    pub properties: HashMap<String, Value>,
}
```

**Update to:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectInstance {
    pub class_name: String,
    pub properties: HashMap<String, Value>,
    pub readonly_properties: std::collections::HashSet<String>, // Track readonly property names
    pub initialized_readonly: std::collections::HashSet<String>, // Track which readonly props are initialized
}
```

#### 5b. Update object instantiation to populate readonly tracking

**Location:** In `src/interpreter/mod.rs`, find `eval_new` function (search for "eval_new")

After creating the object instance and initializing properties, populate the `readonly_properties` set:

```rust
// After creating properties HashMap
let mut readonly_properties = std::collections::HashSet::new();
let mut initialized_readonly = std::collections::HashSet::new();

// Collect readonly property names from class definition
for prop in &class.properties {
    if prop.readonly {
        readonly_properties.insert(prop.name.clone());
        // If it has a default value, it's initialized
        if prop.default.is_some() {
            initialized_readonly.insert(prop.name.clone());
        }
    }
}

// Also handle constructor promoted properties
if let Some(constructor) = class.methods.get("__construct") {
    for param in &constructor.params {
        if param.visibility.is_some() && param.readonly {
            readonly_properties.insert(param.name.clone());
        }
    }
}

let mut instance = ObjectInstance {
    class_name: class_name_lower.clone(),
    properties,
    readonly_properties,
    initialized_readonly,
};
```

#### 5c. Update constructor execution to mark readonly properties as initialized

**Location:** In the constructor execution logic (within `eval_new` or method call handling)

After the constructor executes, mark all readonly properties that were written to as initialized:

```rust
// After constructor completes, mark all current readonly properties as initialized
for prop_name in instance.readonly_properties.iter() {
    if instance.properties.contains_key(prop_name) {
        instance.initialized_readonly.insert(prop_name.clone());
    }
}
```

#### 5d. Enforce readonly in property assignment

**Location:** Find `eval_property_assign` function (search for "eval_property_assign")

Add a check before allowing property modification:

```rust
fn eval_property_assign(
    &mut self,
    object: &Expr,
    property: &str,
    value: &Expr,
) -> Result<Value, String> {
    let obj_val = self.eval_expr(object)?;

    if let Value::Object(mut instance) = obj_val {
        // Check if property is readonly and already initialized
        if instance.readonly_properties.contains(property)
            && instance.initialized_readonly.contains(property)
        {
            return Err(format!(
                "Cannot modify readonly property {}::${}",
                instance.class_name, property
            ));
        }

        let new_value = self.eval_expr(value)?;
        instance.properties.insert(property.to_string(), new_value.clone());

        // If this is a readonly property, mark it as initialized
        if instance.readonly_properties.contains(property) {
            instance.initialized_readonly.insert(property.to_string());
        }

        // Update the variable that holds the object
        if let Expr::Variable(var_name) = object {
            self.variables.insert(var_name.clone(), Value::Object(instance));
        }

        Ok(new_value)
    } else {
        Err(format!(
            "Cannot access property of non-object: {:?}",
            obj_val
        ))
    }
}
```

#### 5e. Handle constructor property promotion with readonly

**Location:** In the class/method execution code where constructor property promotion is handled

When a constructor parameter has `visibility` and `readonly` set, ensure the promoted property is marked as readonly:

```rust
// When promoting constructor parameters to properties
for param in &constructor.params {
    if let Some(visibility) = param.visibility {
        let prop = Property {
            name: param.name.clone(),
            visibility,
            default: None,
            readonly: param.readonly,
        };
        // Add to properties list
        // Mark in readonly_properties set if param.readonly is true
        if param.readonly {
            instance.readonly_properties.insert(param.name.clone());
        }
    }
}
```

### Step 6: Add Tests (`tests/classes/`)

Create comprehensive test files to verify readonly property behavior:

#### Test 1: `tests/classes/readonly_property_basic.vhpt`

```php
--TEST--
Readonly property - basic usage
--FILE--
<?php
class Point {
    public function __construct(
        public readonly float $x,
        public readonly float $y
    ) {}
}

$p = new Point(1.5, 2.5);
echo $p->x;
echo "\n";
echo $p->y;
--EXPECT--
1.5
2.5
```

#### Test 2: `tests/classes/readonly_property_error.vhpt`

```php
--TEST--
Readonly property - modification error
--FILE--
<?php
class Point {
    public function __construct(
        public readonly float $x,
        public readonly float $y
    ) {}
}

$p = new Point(1.5, 2.5);
$p->x = 10.0;
--EXPECT_ERROR--
Cannot modify readonly property
```

#### Test 3: `tests/classes/readonly_property_explicit.vhpt`

```php
--TEST--
Readonly property - explicit declaration
--FILE--
<?php
class User {
    public readonly string $name;

    public function __construct(string $name) {
        $this->name = $name;
    }
}

$u = new User("Alice");
echo $u->name;
--EXPECT--
Alice
```

#### Test 4: `tests/classes/readonly_property_explicit_error.vhpt`

```php
--TEST--
Readonly property - explicit declaration modification error
--FILE--
<?php
class User {
    public readonly string $name;

    public function __construct(string $name) {
        $this->name = $name;
    }
}

$u = new User("Alice");
$u->name = "Bob";
--EXPECT_ERROR--
Cannot modify readonly property
```

#### Test 5: `tests/classes/readonly_property_uninitialized.vhpt`

```php
--TEST--
Readonly property - can be set once in constructor
--FILE--
<?php
class Config {
    public readonly string $env;

    public function __construct() {
        $this->env = "production";
    }

    public function tryModify() {
        $this->env = "development";
    }
}

$c = new Config();
echo $c->env;
echo "\n";
$c->tryModify();
--EXPECT_ERROR--
Cannot modify readonly property
```

#### Test 6: `tests/classes/readonly_property_mixed.vhpt`

```php
--TEST--
Readonly property - mixed with regular properties
--FILE--
<?php
class Product {
    public readonly int $id;
    public string $name;
    public readonly float $price;

    public function __construct(int $id, string $name, float $price) {
        $this->id = $id;
        $this->name = $name;
        $this->price = $price;
    }
}

$p = new Product(1, "Widget", 9.99);
echo $p->id;
echo "\n";
echo $p->name;
echo "\n";
$p->name = "Gadget";
echo $p->name;
--EXPECT--
1
Widget
Gadget
```

#### Test 7: `tests/classes/readonly_property_all_visibilities.vhpt`

```php
--TEST--
Readonly property - all visibility modifiers
--FILE--
<?php
class Account {
    public readonly int $id;
    protected readonly string $username;
    private readonly string $password;

    public function __construct(int $id, string $username, string $password) {
        $this->id = $id;
        $this->username = $username;
        $this->password = $password;
    }

    public function getId() {
        return $this->id;
    }
}

$a = new Account(123, "alice", "secret");
echo $a->getId();
--EXPECT--
123
```

#### Test 8: `tests/classes/readonly_property_default_value.vhpt`

```php
--TEST--
Readonly property - with default value
--FILE--
<?php
class Logger {
    public readonly string $level = "info";

    public function getLevel() {
        return $this->level;
    }
}

$l = new Logger();
echo $l->getLevel();
--EXPECT--
info
```

#### Test 9: `tests/classes/readonly_property_default_value_error.vhpt`

```php
--TEST--
Readonly property - default value cannot be modified
--FILE--
<?php
class Logger {
    public readonly string $level = "info";
}

$l = new Logger();
$l->level = "debug";
--EXPECT_ERROR--
Cannot modify readonly property
```

### Step 7: Update Documentation

#### 7a. Update `docs/roadmap.md`

Change the Phase 5 remaining section from:

```markdown
**Remaining for Phase 5:**
- ✅ **Constructor Property Promotion** (PHP 8.0) - Shorthand syntax for declaring and initializing properties in constructor.
- Readonly Properties (PHP 8.1) & Classes (PHP 8.2)
- "Clone with" functionality (PHP 8.5)
```

To:

```markdown
**Remaining for Phase 5:**
- ✅ **Constructor Property Promotion** (PHP 8.0) - Shorthand syntax for declaring and initializing properties in constructor.
- ✅ **Readonly Properties** (PHP 8.1) - Properties that can only be initialized once.
- Readonly Classes (PHP 8.2)
- "Clone with" functionality (PHP 8.5)
```

#### 7b. Update `AGENTS.md`

In the "Current Features" section under "Classes & Objects", add:

```markdown
- [x] Constructor Property Promotion (PHP 8.0)
- [x] Readonly Properties (PHP 8.1)
```

Update the Phase 5 roadmap section:

```markdown
**Remaining for Phase 5 (future):**
- [x] Constructor Property Promotion (PHP 8.0)
- [x] Readonly Properties (PHP 8.1)
- [ ] Readonly Classes (PHP 8.2)
- [ ] "Clone with" functionality (PHP 8.5)
```

#### 7c. Update `README.md`

Add to the Classes & Objects feature list:

```markdown
- [x] Readonly properties (PHP 8.1)
```

## Key Considerations

### PHP Compatibility

1. **Initialization Rules**:
   - Readonly properties can only be initialized once
   - Must be initialized before being read (unless they have default values)
   - Can only be initialized from the declaring class's scope (constructor or methods)
   - Attempting to modify an initialized readonly property throws a fatal error

2. **Type Requirements** (Note: VHP doesn't have strict typing yet):
   - In PHP 8.1, readonly properties must have a type declaration
   - For VHP's initial implementation, we can be more lenient since we don't enforce types yet
   - Future enhancement: enforce type requirements when type system is implemented

3. **Visibility Combinations**:
   - `readonly` works with all visibility modifiers: public, protected, private
   - Syntax: `<visibility> readonly <type> $property`
   - Example: `public readonly string $name`

4. **Default Values**:
   - Readonly properties CAN have default values
   - If they have a default value, they are considered initialized
   - Attempting to modify them later still results in an error

5. **Constructor Property Promotion**:
   - `readonly` works seamlessly with constructor property promotion
   - Syntax: `public readonly string $name` in constructor parameters
   - Creates a readonly promoted property

### Edge Cases to Handle

1. **Uninitialized readonly property access**: Should return null or error? (PHP returns error if accessed before init, but VHP currently returns null for undefined properties - maintain consistency)

2. **Setting readonly property in constructor vs method**: Both should work on first write, subsequent writes should fail

3. **Readonly property with no default and no constructor initialization**: Should be allowed but property remains uninitialized

4. **Cloning objects with readonly properties**: When object cloning is implemented, readonly properties should be preserved but the clone should allow re-initialization (deferred to clone implementation)

### Error Messages

All error messages should clearly indicate:
- That the property is readonly
- The class and property name
- Use format: `"Cannot modify readonly property ClassName::$propertyName"`

## Test Cases Summary

The test suite should verify:

1. ✓ Basic readonly with constructor promotion
2. ✓ Modification attempt throws error
3. ✓ Explicit readonly property declaration
4. ✓ Explicit readonly property modification error
5. ✓ Readonly property set once in constructor, error on method modification
6. ✓ Mixed readonly and regular properties
7. ✓ Readonly with all visibility modifiers
8. ✓ Readonly property with default value
9. ✓ Readonly property with default value modification error

Additional test cases to consider:
- Readonly property in inherited class
- Readonly property override attempt
- Readonly property with reference types (arrays, objects)
- Multiple readonly properties with different initialization patterns

## Reference Implementation

For implementation patterns, reference these existing features:

| Pattern | Reference File | What to Learn |
|---------|---------------|---------------|
| Token addition | `src/token.rs` line 28-42 | How to add keywords |
| Keyword lexing | `src/lexer.rs` line 193-202 | Keyword recognition in lexer |
| Property struct | `src/ast/stmt.rs` line 11-18 | Existing property structure |
| Property parsing | `src/parser/stmt.rs` line 570-599 | How properties are parsed |
| Constructor promotion | Recent implementation | How visibility in constructor params works |
| Property access | `src/interpreter/mod.rs` | Property read/write logic |
| Object instance | `src/interpreter/value.rs` | ObjectInstance structure |
| Error handling | Any `eval_*` function | How to return errors in interpreter |

## Implementation Notes

1. **Start with AST changes first**: Modify `Property` and `FunctionParam` structs to include `readonly` field
2. **Then lexer/parser**: Add token and parsing logic
3. **Finally interpreter**: Implement runtime enforcement
4. **Test incrementally**: After each major change, run existing tests to ensure no regression

5. **Consider inheritance**: Readonly properties participate in inheritance but cannot be overridden with non-readonly properties (this can be deferred to when inheritance is more complete)

6. **Future enhancement**: In PHP 8.2, entire classes can be marked readonly, making all properties readonly by default. This can be a follow-up feature.

## Validation Checklist

Before considering this feature complete:

- [ ] Token `Readonly` added to `TokenKind`
- [ ] Lexer recognizes `readonly` keyword
- [ ] `Property` struct has `readonly: bool` field
- [ ] `FunctionParam` struct has `readonly: bool` field
- [ ] Parser handles `readonly` in property declarations
- [ ] Parser handles `readonly` in constructor property promotion
- [ ] `ObjectInstance` tracks readonly properties
- [ ] `ObjectInstance` tracks initialized readonly properties
- [ ] Property assignment checks readonly status
- [ ] Constructor initialization marks readonly properties as initialized
- [ ] All 9 test cases pass
- [ ] No regressions in existing tests
- [ ] Documentation updated (AGENTS.md, README.md, roadmap.md)

## Next Steps After Completion

After implementing readonly properties, the logical next features are:

1. **Readonly Classes (PHP 8.2)**: Apply `readonly` to entire class, making all properties readonly by default
2. **Enums (PHP 8.1)**: Another major PHP 8.1 feature, provides type-safe enumerations
3. **Attributes (PHP 8.0)**: Structured metadata system for classes, methods, and properties
