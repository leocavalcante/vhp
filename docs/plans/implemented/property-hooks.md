# Plan: Property Hooks (PHP 8.4)

## Overview

Property hooks allow adding get/set logic directly to property declarations without explicit getter/setter methods. They provide a clean syntax for computed properties, validation, and side effects while maintaining the property access syntax.

**PHP Example:**
```php
<?php
// Before (traditional getters/setters)
class User {
    private string $firstName;
    private string $lastName;

    public function getFullName(): string {
        return $this->firstName . ' ' . $this->lastName;
    }
}

// After (property hooks)
class User {
    public string $firstName;
    public string $lastName;

    public string $fullName {
        get => $this->firstName . ' ' . $this->lastName;
    }
}

$user = new User();
$user->firstName = "John";
$user->lastName = "Doe";
echo $user->fullName; // "John Doe"
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/token.rs` | Add `Get`, `Set` keyword tokens (may conflict with identifiers) |
| `src/ast/stmt.rs` | Add `PropertyHook` struct and extend `Property` |
| `src/parser/stmt/class.rs` | Parse property hook syntax |
| `src/interpreter/objects/properties.rs` | Execute hooks on property access |
| `tests/classes/property_hooks_*.vhpt` | Test files |

## Implementation Steps

### Step 1: Add Get/Set Tokens (`src/token.rs`)

Property hooks use contextual keywords `get` and `set`. In PHP, these are only keywords within property declarations, not globally.

Add tokens (around line 20-100):

```rust
pub enum TokenKind {
    // ... existing tokens ...

    // Property hooks (PHP 8.4) - contextual keywords
    Get,  // 'get' keyword for property hooks
    Set,  // 'set' keyword for property hooks

    // ... rest of tokens ...
}
```

### Step 2: Update Lexer for Contextual Keywords (`src/lexer.rs`)

The `get` and `set` keywords should only be recognized as tokens in specific contexts (within property declarations with braces). For now, add them to keyword matching:

```rust
// In keyword matching (around line 200-300)
match ident.to_lowercase().as_str() {
    // ... existing keywords ...
    "get" => TokenKind::Get,
    "set" => TokenKind::Set,
    // ...
}
```

**Note**: These become identifiers in most contexts. The parser will handle context-sensitive interpretation.

### Step 3: Extend Property AST (`src/ast/stmt.rs`)

Add property hook structures (insert after `Property` struct, around line 131):

```rust
/// Property hook type (PHP 8.4)
#[derive(Debug, Clone)]
pub enum PropertyHookType {
    Get,
    Set,
}

/// Property hook definition (PHP 8.4)
#[derive(Debug, Clone)]
pub struct PropertyHook {
    pub hook_type: PropertyHookType,
    pub body: PropertyHookBody,
}

/// Property hook body can be expression or statements
#[derive(Debug, Clone)]
pub enum PropertyHookBody {
    /// Short syntax: get => expr
    Expression(Box<Expr>),
    /// Block syntax: get { statements }
    Block(Vec<Stmt>),
}

/// Class property definition
#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    pub visibility: Visibility,
    pub default: Option<Expr>,
    pub readonly: bool,             // PHP 8.1+
    pub is_static: bool,            // PHP 5.0+
    pub attributes: Vec<Attribute>, // PHP 8.0+
    pub hooks: Vec<PropertyHook>,   // NEW: PHP 8.4+
    pub type_hint: Option<TypeHint>, // Property type (may already exist)
}
```

**Note**: Check if `type_hint` field already exists in `Property`. If not, add it.

### Step 4: Parse Property Hooks (`src/parser/stmt/class.rs`)

Update property parsing to handle hooks. Find the property parsing logic (around line 200-400):

```rust
fn parse_property(
    &mut self,
    visibility: Visibility,
    is_static: bool,
    readonly: bool,
    attributes: Vec<Attribute>,
) -> Result<Property, String> {
    // Parse type hint if present
    let type_hint = if self.check_type_hint() {
        Some(self.parse_type_hint()?)
    } else {
        None
    };

    // Parse property name: $name
    let name = self.expect_variable()?;

    // Check for property hooks (PHP 8.4)
    let hooks = if self.check(&TokenKind::LeftBrace) {
        self.parse_property_hooks()?
    } else {
        // Parse optional default value: = expr
        let default = if self.check(&TokenKind::Equals) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.expect(&TokenKind::Semicolon)?;

        return Ok(Property {
            name,
            visibility,
            default,
            readonly,
            is_static,
            attributes,
            hooks: vec![],
            type_hint,
        });
    };

    // If we have hooks, no semicolon expected (hooks have their own braces)
    Ok(Property {
        name,
        visibility,
        default: None, // Properties with hooks can't have default values
        readonly,
        is_static,
        attributes,
        hooks,
        type_hint,
    })
}

fn parse_property_hooks(&mut self) -> Result<Vec<PropertyHook>, String> {
    // Expect opening brace
    self.expect(&TokenKind::LeftBrace)?;

    let mut hooks = Vec::new();

    while !self.check(&TokenKind::RightBrace) && !self.is_eof() {
        // Parse hook: get => expr; or set => expr; or get { ... } or set { ... }
        let hook_type = if self.check(&TokenKind::Get) {
            self.advance();
            PropertyHookType::Get
        } else if self.check(&TokenKind::Set) {
            self.advance();
            PropertyHookType::Set
        } else {
            return Err(format!(
                "Expected 'get' or 'set' in property hook at line {}",
                self.current().line
            ));
        };

        let body = if self.check(&TokenKind::Arrow) {
            // Short syntax: get => expr;
            self.advance();
            let expr = self.parse_expression()?;
            self.expect(&TokenKind::Semicolon)?;
            PropertyHookBody::Expression(Box::new(expr))
        } else if self.check(&TokenKind::LeftBrace) {
            // Block syntax: get { statements }
            self.advance();
            let mut statements = Vec::new();

            while !self.check(&TokenKind::RightBrace) && !self.is_eof() {
                if let Some(stmt) = self.parse_statement()? {
                    statements.push(stmt);
                }
            }

            self.expect(&TokenKind::RightBrace)?;
            PropertyHookBody::Block(statements)
        } else {
            return Err(format!(
                "Expected '=>' or '{{' after hook type at line {}",
                self.current().line
            ));
        };

        hooks.push(PropertyHook { hook_type, body });
    }

    self.expect(&TokenKind::RightBrace)?;

    // Validate hooks
    if hooks.is_empty() {
        return Err("Property hooks cannot be empty".to_string());
    }

    // Check for duplicate hooks
    let has_get = hooks.iter().any(|h| matches!(h.hook_type, PropertyHookType::Get));
    let has_set = hooks.iter().any(|h| matches!(h.hook_type, PropertyHookType::Set));
    let get_count = hooks.iter().filter(|h| matches!(h.hook_type, PropertyHookType::Get)).count();
    let set_count = hooks.iter().filter(|h| matches!(h.hook_type, PropertyHookType::Set)).count();

    if get_count > 1 {
        return Err("Duplicate 'get' hook in property".to_string());
    }
    if set_count > 1 {
        return Err("Duplicate 'set' hook in property".to_string());
    }

    Ok(hooks)
}
```

### Step 5: Store Property Hooks in ClassDefinition (`src/interpreter/mod.rs`)

The `Property` AST already contains hooks. When storing class definitions, preserve them. No changes needed to storage, but verify the `ClassDefinition` struct uses the `Property` struct directly.

Check around line 100-200:

```rust
pub struct ClassDefinition {
    pub name: String,
    pub parent: Option<String>,
    pub interfaces: Vec<String>,
    pub properties: Vec<Property>, // Should already include hooks field
    pub methods: Vec<Method>,
    // ... other fields ...
}
```

### Step 6: Execute Get Hook on Property Access (`src/interpreter/objects/properties.rs`)

Find where property access is handled (around line 50-150):

```rust
pub(crate) fn get_property(
    &mut self,
    object_id: usize,
    property: &str,
) -> Result<Value, String> {
    // Get the object
    let object = self
        .objects
        .get(&object_id)
        .ok_or_else(|| "Object not found".to_string())?;

    let class_key = object.class_name.to_lowercase();

    // Get class definition
    let class = self
        .classes
        .get(&class_key)
        .ok_or_else(|| format!("Class '{}' not found", object.class_name))?
        .clone();

    // Find the property definition
    let prop_def = class
        .properties
        .iter()
        .find(|p| p.name == property);

    if let Some(prop) = prop_def {
        // Check if property has a 'get' hook
        if let Some(get_hook) = prop.hooks.iter().find(|h| matches!(h.hook_type, PropertyHookType::Get)) {
            // Execute the get hook in the context of this object
            return self.execute_property_get_hook(object_id, get_hook);
        }
    }

    // No hook - get value from object storage
    let props = &object.properties;
    Ok(props.get(property).cloned().unwrap_or(Value::Null))
}

fn execute_property_get_hook(
    &mut self,
    object_id: usize,
    hook: &PropertyHook,
) -> Result<Value, String> {
    // Save current $this context
    let prev_this = self.current_this;
    self.current_this = Some(object_id);

    let result = match &hook.body {
        PropertyHookBody::Expression(expr) => {
            // Evaluate the expression
            self.evaluate(expr)
        }
        PropertyHookBody::Block(statements) => {
            // Execute statements, capture return value
            let mut result = Value::Null;
            for stmt in statements {
                if let Stmt::Return(expr) = stmt {
                    result = if let Some(e) = expr {
                        self.evaluate(e)?
                    } else {
                        Value::Null
                    };
                    break;
                }
                self.execute_statement(stmt)?;
            }
            Ok(result)
        }
    };

    // Restore $this context
    self.current_this = prev_this;

    result
}
```

### Step 7: Execute Set Hook on Property Assignment (`src/interpreter/objects/properties.rs`)

Find where property assignment is handled (around line 200-300):

```rust
pub(crate) fn set_property(
    &mut self,
    object_id: usize,
    property: &str,
    value: Value,
) -> Result<(), String> {
    // Get the object (need to check class definition first)
    let class_name = {
        let object = self
            .objects
            .get(&object_id)
            .ok_or_else(|| "Object not found".to_string())?;
        object.class_name.clone()
    };

    let class_key = class_name.to_lowercase();

    // Get class definition
    let class = self
        .classes
        .get(&class_key)
        .ok_or_else(|| format!("Class '{}' not found", class_name))?
        .clone();

    // Find the property definition
    let prop_def = class
        .properties
        .iter()
        .find(|p| p.name == property);

    if let Some(prop) = prop_def {
        // Check readonly
        if prop.readonly {
            // Check if property is already initialized
            let object = self.objects.get(&object_id).unwrap();
            if object.properties.contains_key(property) {
                return Err(format!(
                    "Cannot modify readonly property {}::${}",
                    class_name, property
                ));
            }
        }

        // Check if property has a 'set' hook
        if let Some(set_hook) = prop.hooks.iter().find(|h| matches!(h.hook_type, PropertyHookType::Set)) {
            // Execute the set hook in the context of this object
            return self.execute_property_set_hook(object_id, set_hook, value);
        }
    }

    // No hook - set value directly in object storage
    let object = self
        .objects
        .get_mut(&object_id)
        .ok_or_else(|| "Object not found".to_string())?;

    object.properties.insert(property.to_string(), value);
    Ok(())
}

fn execute_property_set_hook(
    &mut self,
    object_id: usize,
    hook: &PropertyHook,
    value: Value,
) -> Result<(), String> {
    // Save current $this context
    let prev_this = self.current_this;
    self.current_this = Some(object_id);

    // Store the incoming value in a special variable $value
    // PHP 8.4 uses implicit parameter - the hook receives the value being set
    let prev_value = self.variables.get("value").cloned();
    self.variables.insert("value".to_string(), value);

    let result = match &hook.body {
        PropertyHookBody::Expression(expr) => {
            // For set hooks, expression form doesn't make much sense
            // But PHP allows it - just evaluate the expression (side effects)
            self.evaluate(expr)?;
            Ok(())
        }
        PropertyHookBody::Block(statements) => {
            // Execute statements
            for stmt in statements {
                self.execute_statement(stmt)?;
            }
            Ok(())
        }
    };

    // Restore context
    self.current_this = prev_this;
    if let Some(v) = prev_value {
        self.variables.insert("value".to_string(), v);
    } else {
        self.variables.remove("value");
    }

    result
}
```

### Step 8: Handle Virtual Properties

Properties with only a `get` hook and no `set` hook are read-only (virtual/computed properties):

```rust
// In set_property, before executing set hook:
if let Some(prop) = prop_def {
    let has_get = prop.hooks.iter().any(|h| matches!(h.hook_type, PropertyHookType::Get));
    let has_set = prop.hooks.iter().any(|h| matches!(h.hook_type, PropertyHookType::Set));

    // Virtual property (get-only)
    if has_get && !has_set {
        return Err(format!(
            "Cannot write to read-only property {}::${}",
            class_name, property
        ));
    }

    // ... rest of set logic ...
}
```

### Step 9: Add Tests

**tests/classes/property_hooks_get_basic.vhpt**
```
--TEST--
Basic property get hook
--FILE--
<?php
class Circle {
    public float $radius = 5.0;

    public float $diameter {
        get => $this->radius * 2;
    }
}

$c = new Circle();
echo $c->diameter;
--EXPECT--
10
```

**tests/classes/property_hooks_set_basic.vhpt**
```
--TEST--
Basic property set hook
--FILE--
<?php
class Temperature {
    private float $celsius = 0;

    public float $fahrenheit {
        get => $this->celsius * 9/5 + 32;
        set {
            $this->celsius = ($value - 32) * 5/9;
        }
    }
}

$t = new Temperature();
$t->fahrenheit = 212;
echo $t->fahrenheit;
--EXPECT--
212
```

**tests/classes/property_hooks_validation.vhpt**
```
--TEST--
Property set hook with validation
--FILE--
<?php
class User {
    private string $email_value = "";

    public string $email {
        get => $this->email_value;
        set {
            if (strpos($value, '@') === false) {
                throw new Exception("Invalid email");
            }
            $this->email_value = $value;
        }
    }
}

$u = new User();
$u->email = "test@example.com";
echo $u->email;
--EXPECT--
test@example.com
```

**tests/classes/property_hooks_computed.vhpt**
```
--TEST--
Computed property (get-only)
--FILE--
<?php
class Person {
    public string $firstName = "John";
    public string $lastName = "Doe";

    public string $fullName {
        get => $this->firstName . ' ' . $this->lastName;
    }
}

$p = new Person();
echo $p->fullName;
--EXPECT--
John Doe
```

**tests/classes/property_hooks_readonly_error.vhpt**
```
--TEST--
Error when writing to computed property
--FILE--
<?php
class Circle {
    public float $radius = 5.0;

    public float $area {
        get => 3.14159 * $this->radius * $this->radius;
    }
}

$c = new Circle();
$c->area = 100;
--EXPECT_ERROR--
Cannot write to read-only property
```

**tests/classes/property_hooks_block_syntax.vhpt**
```
--TEST--
Property hook with block syntax
--FILE--
<?php
class Logger {
    private array $log = [];

    public string $message {
        set {
            $this->log[] = $value;
            echo "Logged: " . $value;
        }
    }
}

$logger = new Logger();
$logger->message = "Hello";
--EXPECT--
Logged: Hello
```

**tests/classes/property_hooks_expression_get.vhpt**
```
--TEST--
Property get hook with expression
--FILE--
<?php
class Box {
    public int $width = 10;
    public int $height = 20;

    public int $area {
        get => $this->width * $this->height;
    }
}

$b = new Box();
echo $b->area;
--EXPECT--
200
```

**tests/classes/property_hooks_both_hooks.vhpt**
```
--TEST--
Property with both get and set hooks
--FILE--
<?php
class Wrapper {
    private mixed $value;

    public mixed $data {
        get {
            return $this->value ?? "default";
        }
        set {
            $this->value = strtoupper($value);
        }
    }
}

$w = new Wrapper();
$w->data = "hello";
echo $w->data;
--EXPECT--
HELLO
```

**tests/classes/property_hooks_static_error.vhpt**
```
--TEST--
Static properties cannot have hooks (PHP 8.4 limitation)
--FILE--
<?php
class Test {
    public static int $count {
        get => 0;
    }
}
--EXPECT_ERROR--
Static properties cannot have hooks
```

**tests/classes/property_hooks_no_default.vhpt**
```
--TEST--
Properties with hooks cannot have default values
--FILE--
<?php
class Test {
    public int $value = 10 {
        get => $this->value;
    }
}
--EXPECT_ERROR--
Property with hooks cannot have default value
```

**tests/classes/property_hooks_inheritance.vhpt**
```
--TEST--
Property hook inheritance
--FILE--
<?php
class Base {
    public int $value {
        get => 100;
    }
}

class Child extends Base {
}

$c = new Child();
echo $c->value;
--EXPECT--
100
```

## Key Considerations

1. **Virtual Properties**: Properties with only `get` hooks are read-only computed properties
2. **Implicit Parameter**: In `set` hooks, the value being assigned is available as `$value`
3. **Context**: Hooks execute in the object's context with `$this` available
4. **No Default Values**: Properties with hooks cannot have default values in the declaration
5. **Static Properties**: PHP 8.4 does not support hooks on static properties
6. **Type Validation**: Property type hints are checked before set hooks execute
7. **Readonly Conflict**: Hooks and `readonly` keyword cannot be combined
8. **Inheritance**: Child classes inherit parent property hooks unless overridden

## PHP Compatibility Notes

| Feature | PHP Version | Notes |
|---------|-------------|-------|
| Property hooks | 8.4 | New feature |
| Short syntax (`=>`) | 8.4 | For single-expression hooks |
| Block syntax (`{}`) | 8.4 | For multi-statement hooks |
| Implicit `$value` in set | 8.4 | Automatically available |

## Implementation Order

1. Add `Get` and `Set` tokens to lexer
2. Define `PropertyHook` and related AST structures
3. Update `Property` struct to include hooks
4. Parse property hook syntax in class parser
5. Implement get hook execution on property access
6. Implement set hook execution on property assignment
7. Handle virtual properties (get-only)
8. Add validation (no static hooks, no default values with hooks)
9. Add comprehensive tests

## Edge Cases

1. **Get-only properties**: Should error on write attempts
2. **Set-only properties**: Can be written but returns `null` when read (unless backed by storage)
3. **Type checking**: Property type hints validated before hooks execute
4. **Exceptions in hooks**: Should propagate to caller
5. **Recursive access**: Hook accessing same property could cause infinite loop (PHP behavior)
6. **Backed properties**: Hooks can access private backing storage
7. **Visibility**: Hooks respect property visibility
8. **Static properties**: Not allowed with hooks in PHP 8.4
9. **Readonly + hooks**: Not allowed to combine
10. **Default values**: Cannot specify default value if hooks present

## Reference Implementations

- Property access: `src/interpreter/objects/properties.rs`
- Property storage: `src/interpreter/mod.rs` (Object struct)
- Class definitions: `src/interpreter/mod.rs` (ClassDefinition)
- Method execution context: `src/interpreter/objects/methods.rs` (for `$this` handling)

## Additional Notes

**Contextual Keywords**: `get` and `set` are contextual keywords - they're only keywords within property hook declarations. Elsewhere they're regular identifiers. The parser should handle this by checking context.

**Backed vs Virtual Properties**:
- Properties with hooks can have backing storage (private fields)
- Pure computed properties (get-only) don't need backing storage
- Set hooks typically write to a backing field

**Performance**: Since hooks execute code on every property access, they have overhead. Document this for users.

**Future Enhancements** (not in PHP 8.4):
- `final` hooks (prevent override in child classes)
- Hook parameters with types
- Asymmetric visibility combined with hooks
