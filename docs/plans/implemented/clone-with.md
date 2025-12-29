# Plan: Clone and Clone With (PHP 5.0 + PHP 8.4)

## Overview

This plan implements PHP's object cloning functionality in two parts:
1. **Basic `clone` operator** (PHP 5.0+) - Creates a shallow copy of an object
2. **`clone with` syntax** (PHP 8.4+) - Creates a copy while modifying specific properties

The `clone` operator creates a new instance with the same property values. The `clone with` syntax extends this by allowing property modifications in a single expression, making it easier to create modified copies of objects (especially useful with readonly properties and immutable patterns).

**PHP Example:**
```php
// Basic clone (PHP 5.0+)
class Point {
    public function __construct(
        public float $x,
        public float $y
    ) {}
}

$p1 = new Point(1.0, 2.0);
$p2 = clone $p1;
$p2->x = 3.0;
echo $p2->x; // 3.0

// Clone with (PHP 8.4+)
class ImmutablePoint {
    public function __construct(
        public readonly float $x,
        public readonly float $y
    ) {}
}

$p1 = new ImmutablePoint(1.0, 2.0);
$p2 = clone $p1 with { x: 3.0 };
echo $p2->x; // 3.0
echo $p1->x; // 1.0 (original unchanged)
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/token.rs` | Add `Clone` and `With` tokens |
| `src/lexer.rs` | Recognize `clone` and `with` keywords |
| `src/ast/expr.rs` | Add `Clone` and `CloneWith` expression variants |
| `src/parser/expr.rs` | Parse `clone` and `clone with` expressions |
| `src/interpreter/mod.rs` | Implement cloning logic and property modification |
| `src/interpreter/value.rs` | Update ObjectInstance cloning behavior |
| `tests/classes/clone_*.vhpt` | Add comprehensive test suite |
| `docs/features.md` | Document clone functionality |
| `AGENTS.md` | Update feature list |
| `README.md` | Update feature list |

## Implementation Steps

### Step 1: Add Tokens (`src/token.rs`)

Add the `Clone` and `With` tokens to the `TokenKind` enum after line 44 (after `Enum`):

```rust
    Enum,         // enum (PHP 8.1)
    Clone,        // clone (PHP 5.0)
    With,         // with (PHP 8.4) - for clone with syntax
```

### Step 2: Update Lexer (`src/lexer.rs`)

Add keyword recognition in the identifier matching section. Find the `match ident.to_lowercase().as_str()` block (around line 300-350) and add:

```rust
    "clone" => TokenKind::Clone,
    "with" => TokenKind::With,
```

These should be added alphabetically in the keyword list.

### Step 3: Extend AST (`src/ast/expr.rs`)

Add two new expression variants to the `Expr` enum after line 132 (after `EnumCase`):

```rust
    // Enum case access: EnumName::CASE
    EnumCase {
        enum_name: String,
        case_name: String,
    },

    // Clone expression: clone $obj
    Clone {
        object: Box<Expr>,
    },

    // Clone with expression: clone $obj with { prop: value, ... }
    CloneWith {
        object: Box<Expr>,
        modifications: Vec<PropertyModification>,
    },
}
```

Also add a new struct before the `Expr` enum definition (around line 24):

```rust
/// Property modification for clone with syntax (PHP 8.4)
#[derive(Debug, Clone)]
pub struct PropertyModification {
    pub property: String,
    pub value: Box<Expr>,
}
```

Update the import in `src/ast/mod.rs` to export the new struct (around line 1-5):

```rust
pub use expr::{ArrayElement, Argument, Expr, MatchArm, PropertyModification};
```

### Step 4: Update Parser (`src/parser/expr.rs`)

#### 4.1 Add `clone` to primary expression parsing

Find the `parse_primary` method (around line 20-30) and add a case for `TokenKind::Clone` after the `TokenKind::New` case:

```rust
            TokenKind::New => {
                self.advance();
                self.parse_new()
            }

            TokenKind::Clone => {
                self.advance();
                self.parse_clone()
            }
```

#### 4.2 Implement `parse_clone` method

Add this new method to the `Parser` impl block (around line 600, near other parsing methods):

```rust
    /// Parse clone or clone with expression
    /// clone $obj
    /// clone $obj with { prop: value, ... }
    fn parse_clone(&mut self) -> Result<Expr, String> {
        // Parse the object expression
        let object = Box::new(self.parse_unary()?);

        // Check if followed by 'with'
        if self.match_token(&TokenKind::With) {
            self.advance(); // consume 'with'

            // Expect opening brace
            if !self.match_token(&TokenKind::LeftBrace) {
                return Err(format!(
                    "Expected '{{' after 'with' at line {}",
                    self.current().line
                ));
            }
            self.advance(); // consume '{'

            let mut modifications = Vec::new();

            // Parse property modifications
            loop {
                // Check for closing brace
                if self.match_token(&TokenKind::RightBrace) {
                    self.advance();
                    break;
                }

                // Parse property name (identifier)
                let property = match &self.current().kind {
                    TokenKind::Identifier(name) => name.clone(),
                    _ => {
                        return Err(format!(
                            "Expected property name at line {}",
                            self.current().line
                        ))
                    }
                };
                self.advance();

                // Expect colon
                if !self.match_token(&TokenKind::Colon) {
                    return Err(format!(
                        "Expected ':' after property name at line {}",
                        self.current().line
                    ));
                }
                self.advance(); // consume ':'

                // Parse value expression
                let value = Box::new(self.parse_expression()?);

                modifications.push(PropertyModification { property, value });

                // Check for comma or closing brace
                if self.match_token(&TokenKind::Comma) {
                    self.advance();
                    // Allow trailing comma before closing brace
                    if self.match_token(&TokenKind::RightBrace) {
                        self.advance();
                        break;
                    }
                } else if self.match_token(&TokenKind::RightBrace) {
                    self.advance();
                    break;
                } else {
                    return Err(format!(
                        "Expected ',' or '}}' after property value at line {}",
                        self.current().line
                    ));
                }
            }

            if modifications.is_empty() {
                return Err(format!(
                    "Clone with syntax requires at least one property modification at line {}",
                    self.current().line
                ));
            }

            Ok(Expr::CloneWith {
                object,
                modifications,
            })
        } else {
            // Simple clone without modifications
            Ok(Expr::Clone { object })
        }
    }
```

### Step 5: Update Interpreter (`src/interpreter/mod.rs`)

#### 5.1 Add evaluation cases

Find the `eval_expr` method's match statement (around line 150-200) and add cases for `Expr::Clone` and `Expr::CloneWith` after the `Expr::EnumCase` case:

```rust
            Expr::EnumCase {
                enum_name,
                case_name,
            } => self.eval_enum_case(enum_name, case_name),

            Expr::Clone { object } => self.eval_clone(object),

            Expr::CloneWith {
                object,
                modifications,
            } => self.eval_clone_with(object, modifications),
```

#### 5.2 Implement `eval_clone` method

Add this method to the `Interpreter` impl block (around line 1200, after other eval methods):

```rust
    /// Evaluate clone expression: clone $obj
    fn eval_clone(&mut self, object_expr: &Expr) -> Result<Value, String> {
        let object_value = self.eval_expr(object_expr)?;

        match object_value {
            Value::Object(instance) => {
                // Create a deep clone of the object
                let mut cloned_instance = ObjectInstance {
                    class_name: instance.class_name.clone(),
                    properties: instance.properties.clone(),
                    readonly_properties: instance.readonly_properties.clone(),
                    initialized_readonly: std::collections::HashSet::new(), // Reset initialization tracking
                };

                // For a cloned object, readonly properties can be re-initialized
                // This is PHP's behavior: clone creates a new object context
                Ok(Value::Object(cloned_instance))
            }
            _ => Err(format!(
                "__clone method called on non-object ({})",
                object_value.get_type()
            )),
        }
    }

    /// Evaluate clone with expression: clone $obj with { prop: value, ... }
    fn eval_clone_with(
        &mut self,
        object_expr: &Expr,
        modifications: &[PropertyModification],
    ) -> Result<Value, String> {
        let object_value = self.eval_expr(object_expr)?;

        match object_value {
            Value::Object(instance) => {
                // Create a deep clone of the object
                let mut cloned_instance = ObjectInstance {
                    class_name: instance.class_name.clone(),
                    properties: instance.properties.clone(),
                    readonly_properties: instance.readonly_properties.clone(),
                    initialized_readonly: std::collections::HashSet::new(), // Reset for clone
                };

                // Apply modifications
                for modification in modifications {
                    let property_name = &modification.property;

                    // Check if property exists in the original object
                    if !cloned_instance.properties.contains_key(property_name) {
                        return Err(format!(
                            "Property '{}' does not exist on class '{}'",
                            property_name, cloned_instance.class_name
                        ));
                    }

                    // Evaluate the new value
                    let new_value = self.eval_expr(&modification.value)?;

                    // Set the property value
                    cloned_instance
                        .properties
                        .insert(property_name.clone(), new_value);

                    // Mark readonly property as initialized if it's readonly
                    if cloned_instance.readonly_properties.contains(property_name) {
                        cloned_instance
                            .initialized_readonly
                            .insert(property_name.clone());
                    }
                }

                Ok(Value::Object(cloned_instance))
            }
            _ => Err(format!(
                "Clone with called on non-object ({})",
                object_value.get_type()
            )),
        }
    }
```

### Step 6: Add Tests (`tests/classes/`)

Create comprehensive test files to cover all clone scenarios:

#### Test 1: `tests/classes/clone_basic.vhpt`

```php
--TEST--
Basic clone - simple object cloning
--FILE--
<?php
class Point {
    public function __construct(
        public float $x,
        public float $y
    ) {}
}

$p1 = new Point(1.0, 2.0);
$p2 = clone $p1;

echo "p1: " . $p1->x . ", " . $p1->y . "\n";
echo "p2: " . $p2->x . ", " . $p2->y . "\n";

// Modify clone
$p2->x = 3.0;
$p2->y = 4.0;

echo "After modification:\n";
echo "p1: " . $p1->x . ", " . $p1->y . "\n";
echo "p2: " . $p2->x . ", " . $p2->y . "\n";
--EXPECT--
p1: 1.0, 2.0
p2: 1.0, 2.0
After modification:
p1: 1.0, 2.0
p2: 3.0, 4.0
```

#### Test 2: `tests/classes/clone_with_basic.vhpt`

```php
--TEST--
Clone with - basic property modification
--FILE--
<?php
class Point {
    public function __construct(
        public float $x,
        public float $y
    ) {}
}

$p1 = new Point(1.0, 2.0);
$p2 = clone $p1 with { x: 3.0 };

echo "p1: " . $p1->x . ", " . $p1->y . "\n";
echo "p2: " . $p2->x . ", " . $p2->y . "\n";
--EXPECT--
p1: 1.0, 2.0
p2: 3.0, 2.0
```

#### Test 3: `tests/classes/clone_with_multiple.vhpt`

```php
--TEST--
Clone with - multiple property modifications
--FILE--
<?php
class Point {
    public function __construct(
        public float $x,
        public float $y
    ) {}
}

$p1 = new Point(1.0, 2.0);
$p2 = clone $p1 with { x: 3.0, y: 4.0 };

echo "p1: " . $p1->x . ", " . $p1->y . "\n";
echo "p2: " . $p2->x . ", " . $p2->y . "\n";
--EXPECT--
p1: 1.0, 2.0
p2: 3.0, 4.0
```

#### Test 4: `tests/classes/clone_with_readonly.vhpt`

```php
--TEST--
Clone with - readonly properties can be modified during clone
--FILE--
<?php
class ImmutablePoint {
    public function __construct(
        public readonly float $x,
        public readonly float $y
    ) {}
}

$p1 = new ImmutablePoint(1.0, 2.0);
$p2 = clone $p1 with { x: 3.0 };

echo "p1: " . $p1->x . ", " . $p1->y . "\n";
echo "p2: " . $p2->x . ", " . $p2->y . "\n";
--EXPECT--
p1: 1.0, 2.0
p2: 3.0, 2.0
```

#### Test 5: `tests/classes/clone_with_readonly_class.vhpt`

```php
--TEST--
Clone with - works with readonly classes
--FILE--
<?php
readonly class Point {
    public function __construct(
        public float $x,
        public float $y
    ) {}
}

$p1 = new Point(1.0, 2.0);
$p2 = clone $p1 with { x: 3.0, y: 4.0 };

echo "p1: " . $p1->x . ", " . $p1->y . "\n";
echo "p2: " . $p2->x . ", " . $p2->y . "\n";
--EXPECT--
p1: 1.0, 2.0
p2: 3.0, 4.0
```

#### Test 6: `tests/classes/clone_with_trailing_comma.vhpt`

```php
--TEST--
Clone with - trailing comma allowed
--FILE--
<?php
class Point {
    public function __construct(
        public float $x,
        public float $y
    ) {}
}

$p1 = new Point(1.0, 2.0);
$p2 = clone $p1 with { x: 3.0, y: 4.0, };

echo "p2: " . $p2->x . ", " . $p2->y;
--EXPECT--
p2: 3.0, 4.0
```

#### Test 7: `tests/classes/clone_with_expressions.vhpt`

```php
--TEST--
Clone with - property values can be expressions
--FILE--
<?php
class Point {
    public function __construct(
        public float $x,
        public float $y
    ) {}
}

$p1 = new Point(1.0, 2.0);
$factor = 2.0;
$p2 = clone $p1 with { x: $p1->x * $factor, y: $p1->y + 10.0 };

echo "p2: " . $p2->x . ", " . $p2->y;
--EXPECT--
p2: 2.0, 12.0
```

#### Test 8: `tests/classes/clone_nested_objects.vhpt`

```php
--TEST--
Clone - nested objects are shallow copied
--FILE--
<?php
class Inner {
    public function __construct(public int $value) {}
}

class Outer {
    public function __construct(public Inner $inner) {}
}

$inner = new Inner(10);
$o1 = new Outer($inner);
$o2 = clone $o1;

// Modify inner object through clone
$o2->inner->value = 20;

// Original's inner object is also changed (shallow copy)
echo "o1 inner: " . $o1->inner->value . "\n";
echo "o2 inner: " . $o2->inner->value;
--EXPECT--
o1 inner: 20
o2 inner: 20
```

#### Test 9: `tests/classes/clone_non_object_error.vhpt`

```php
--TEST--
Clone - error when cloning non-object
--FILE--
<?php
$x = 42;
$y = clone $x;
--EXPECT_ERROR--
__clone method called on non-object
```

#### Test 10: `tests/classes/clone_with_nonexistent_property_error.vhpt`

```php
--TEST--
Clone with - error when modifying non-existent property
--FILE--
<?php
class Point {
    public function __construct(
        public float $x,
        public float $y
    ) {}
}

$p1 = new Point(1.0, 2.0);
$p2 = clone $p1 with { z: 3.0 };
--EXPECT_ERROR--
Property 'z' does not exist on class 'Point'
```

#### Test 11: `tests/classes/clone_with_empty_error.vhpt`

```php
--TEST--
Clone with - error when no modifications provided
--FILE--
<?php
class Point {
    public function __construct(
        public float $x,
        public float $y
    ) {}
}

$p1 = new Point(1.0, 2.0);
$p2 = clone $p1 with { };
--EXPECT_ERROR--
Clone with syntax requires at least one property modification
```

#### Test 12: `tests/classes/clone_with_computed_property.vhpt`

```php
--TEST--
Clone with - using computed values from original object
--FILE--
<?php
class Rectangle {
    public function __construct(
        public float $width,
        public float $height
    ) {}
}

$r1 = new Rectangle(10.0, 5.0);
$r2 = clone $r1 with { width: $r1->width * 2.0 };

echo "r1: " . $r1->width . "x" . $r1->height . "\n";
echo "r2: " . $r2->width . "x" . $r2->height;
--EXPECT--
r1: 10.0x5.0
r2: 20.0x5.0
```

### Step 7: Update Documentation

#### Update `docs/features.md`

Add a new section after "Readonly Classes":

```markdown
### Object Cloning (PHP 5.0+)

#### Basic Clone

The `clone` operator creates a shallow copy of an object:

```php
class Point {
    public function __construct(
        public float $x,
        public float $y
    ) {}
}

$p1 = new Point(1.0, 2.0);
$p2 = clone $p1;
$p2->x = 3.0;

echo $p1->x; // 1.0 (original unchanged)
echo $p2->x; // 3.0
```

**Shallow Copy Behavior**: Object properties that reference other objects are copied by reference, not deeply cloned.

#### Clone With (PHP 8.4+)

The `clone with` syntax creates a copy while modifying specific properties in a single expression:

```php
readonly class ImmutablePoint {
    public function __construct(
        public float $x,
        public float $y
    ) {}
}

$p1 = new ImmutablePoint(1.0, 2.0);
$p2 = clone $p1 with { x: 3.0 };

echo $p2->x; // 3.0
echo $p2->y; // 2.0 (unchanged)
```

**Key Features**:
- Multiple properties can be modified: `clone $obj with { prop1: val1, prop2: val2 }`
- Property values can be expressions: `clone $obj with { x: $obj->x * 2 }`
- Works with readonly properties (they can be re-initialized in the clone)
- Trailing commas are allowed
- At least one property modification is required

**Use Cases**:
- Creating modified copies of immutable objects
- Avoiding mutation of original objects
- Functional programming patterns
```

#### Update `AGENTS.md`

Find the "### Classes & Objects" section (around line 195-200) and add:

```markdown
- [x] Constructor Property Promotion (PHP 8.0)
- [x] Readonly Properties (PHP 8.1)
- [x] Readonly Classes (PHP 8.2)
- [x] Object cloning with `clone` keyword (PHP 5.0)
- [x] Clone with property modification syntax (PHP 8.4)
```

Find the roadmap Phase 5 section (around line 421) and update:

```markdown
**Remaining for Phase 5 (future):**
- [x] Readonly Classes (PHP 8.2)
- [x] "Clone with" functionality (PHP 8.4)
```

#### Update `README.md`

Find the Classes & Objects section and add:

```markdown
- [x] Readonly Classes (PHP 8.2)
- [x] Object cloning (`clone` operator)
- [x] Clone with property modification (`clone $obj with { prop: value }`)
```

## Key Considerations

### PHP Compatibility

1. **Shallow Copy**: The `clone` operator performs a shallow copy. Object properties containing references to other objects are copied by reference, not deeply cloned.

2. **Readonly Properties**: In a cloned object, readonly properties can be re-initialized. The clone creates a new object context where readonly properties are not yet initialized.

3. **`__clone()` Magic Method**: PHP supports a `__clone()` magic method that is called after cloning. This is deferred to a future implementation but should be considered when extending this feature.

4. **Property Visibility**: The `clone with` syntax respects property visibility rules. Only accessible properties can be modified.

### Edge Cases

1. **Non-Object Clone**: Attempting to clone non-objects throws an error: `__clone method called on non-object`

2. **Non-Existent Property**: Attempting to modify a property that doesn't exist in `clone with` throws an error

3. **Empty Modifications**: `clone $obj with { }` is an error - at least one property modification is required

4. **Trailing Comma**: PHP allows trailing commas in `clone with` syntax: `clone $obj with { x: 1, }`

5. **Expression Evaluation**: Property values in `clone with` can be any expression, evaluated at clone time

### Interaction with Existing Features

1. **Readonly Properties**: Clone with allows re-initialization of readonly properties in the cloned object

2. **Readonly Classes**: Works seamlessly with readonly classes, allowing property modifications during clone

3. **Constructor Promotion**: Cloned objects properly copy properties created through constructor promotion

4. **Inheritance**: Cloning works with inherited properties

## Test Cases

The test suite covers:
- Basic clone functionality
- Clone with single property modification
- Clone with multiple property modifications
- Clone with readonly properties
- Clone with readonly classes
- Trailing comma support
- Expression evaluation in property values
- Nested object shallow copy behavior
- Error: cloning non-objects
- Error: modifying non-existent properties
- Error: empty modification list

## Reference Implementation

Similar patterns in the existing codebase:

| Feature | File | Line Range | Notes |
|---------|------|------------|-------|
| `new` expression parsing | `src/parser/expr.rs` | ~500-530 | Similar unary operator pattern |
| `new` expression evaluation | `src/interpreter/mod.rs` | ~1006-1100 | Object instantiation logic |
| Object property assignment | `src/interpreter/mod.rs` | ~1150-1200 | Property modification pattern |
| Readonly property handling | `src/interpreter/mod.rs` | ~1020-1036 | Readonly initialization tracking |
| Match expression parsing | `src/parser/expr.rs` | ~600-700 | Similar brace-delimited syntax |

## Notes for Implementation

1. **Parser precedence**: `clone` should be parsed as a unary prefix operator, similar to `new`

2. **Error messages**: Follow PHP's error message format for consistency

3. **Performance**: Cloning involves copying all property values. For large objects, this may have performance implications.

4. **Future enhancements**:
   - Deep clone support (requires `__clone()` magic method)
   - Clone with visibility enforcement (currently all properties accessible)
   - Serialization/deserialization for deep copies

5. **Testing strategy**: Focus on readonly properties interaction since that's the primary use case for `clone with`
