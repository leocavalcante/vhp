# Plan: Readonly Classes (PHP 8.2)

## Overview

Readonly classes are a shorthand way to make all properties in a class readonly. When a class is declared with the `readonly` modifier, all its properties automatically become readonly, eliminating the need to mark each property individually.

**PHP Example:**

```php
// Before (PHP 8.1 - marking each property)
class Point {
    public function __construct(
        public readonly float $x,
        public readonly float $y
    ) {}
}

// After (PHP 8.2 - readonly class)
readonly class Point {
    public function __construct(
        public float $x,
        public float $y
    ) {}
}

// All properties are implicitly readonly
$p = new Point(1.5, 2.5);
echo $p->x; // OK
$p->x = 3.0; // Error: Cannot modify readonly property
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/ast/stmt.rs` | Add `readonly` field to `Stmt::Class` enum variant |
| `src/parser/stmt.rs` | Parse `readonly` modifier before `class` keyword |
| `src/interpreter/mod.rs` | Add `readonly` field to `ClassDefinition` struct, enforce readonly semantics |
| `tests/classes/*.vhpt` | Add test files for readonly classes |
| `AGENTS.md` | Mark feature as complete |
| `README.md` | Update roadmap and features list |
| `docs/features.md` | Document readonly classes |
| `docs/roadmap.md` | Mark as complete |

## Implementation Steps

### Step 1: Extend AST (`src/ast/stmt.rs`)

**Location:** Line 128-135 (in the `Stmt::Class` variant)

**Change:** Add `readonly` field to the `Class` variant.

**Before:**
```rust
    Class {
        name: String,
        parent: Option<String>,
        interfaces: Vec<String>,
        trait_uses: Vec<TraitUse>,
        properties: Vec<Property>,
        methods: Vec<Method>,
    },
```

**After:**
```rust
    Class {
        name: String,
        readonly: bool, // PHP 8.2+: all properties are implicitly readonly
        parent: Option<String>,
        interfaces: Vec<String>,
        trait_uses: Vec<TraitUse>,
        properties: Vec<Property>,
        methods: Vec<Method>,
    },
```

### Step 2: Update Parser (`src/parser/stmt.rs`)

**Location:** Line 1135 (in `parse_class` function)

**Changes:**

1. Parse `readonly` modifier before `class` keyword
2. Validate that properties in readonly classes don't have explicit `readonly` modifier (warn or error)
3. If class is readonly and parent exists, ensure proper inheritance validation

**Code to add after line 1135:**

```rust
    pub fn parse_class(&mut self) -> Result<Stmt, String> {
        // Check for readonly modifier before class keyword
        let readonly = if self.check(&TokenKind::Readonly) {
            self.advance();
            true
        } else {
            false
        };

        self.consume(TokenKind::Class, "Expected 'class' keyword")?;

        let name = if let TokenKind::Identifier(name) = &self.current().kind {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(format!(
                "Expected class name at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        };

        // ... rest of the function remains the same until the return statement
```

**Location:** At the end of `parse_class`, before returning the `Stmt::Class` (around line 1330)

**Code to modify:**

Find where `Stmt::Class` is returned and add validation for readonly class properties:

```rust
        // If class is readonly, validate properties
        if readonly {
            for property in &properties {
                if property.readonly {
                    return Err(format!(
                        "Property '{}' cannot have explicit 'readonly' modifier in readonly class '{}' at line {}, column {}",
                        property.name,
                        name,
                        self.current().line,
                        self.current().column
                    ));
                }
            }
        }

        Ok(Stmt::Class {
            name,
            readonly, // Add this field
            parent,
            interfaces,
            trait_uses,
            properties,
            methods,
        })
```

### Step 3: Update Interpreter - ClassDefinition (`src/interpreter/mod.rs`)

**Location:** Line 36-44 (in `ClassDefinition` struct)

**Change:** Add `readonly` field.

**Before:**
```rust
pub struct ClassDefinition {
    pub name: String,
    #[allow(dead_code)] // Will be used for inheritance support
    pub parent: Option<String>,
    pub properties: Vec<Property>,
    pub methods: HashMap<String, UserFunction>,
    #[allow(dead_code)] // Will be used for visibility enforcement
    pub method_visibility: HashMap<String, Visibility>,
}
```

**After:**
```rust
pub struct ClassDefinition {
    pub name: String,
    pub readonly: bool, // PHP 8.2+: if true, all properties are implicitly readonly
    #[allow(dead_code)] // Will be used for inheritance support
    pub parent: Option<String>,
    pub properties: Vec<Property>,
    pub methods: HashMap<String, UserFunction>,
    #[allow(dead_code)] // Will be used for visibility enforcement
    pub method_visibility: HashMap<String, Visibility>,
}
```

### Step 4: Update Interpreter - Class Registration (`src/interpreter/mod.rs`)

**Location:** Line 1771 (in `execute_statement` where `Stmt::Class` is handled)

**Changes:**

1. Store the `readonly` flag in `ClassDefinition`
2. If parent class exists and current class is readonly, validate parent is also readonly
3. When parent is not readonly but child is readonly, allow it (child can be more restrictive)

**Find this code (around line 1771):**

```rust
            Stmt::Class {
                name,
                parent,
                interfaces,
                trait_uses,
                properties,
                methods,
            } => {
```

**Update to:**

```rust
            Stmt::Class {
                name,
                readonly,
                parent,
                interfaces,
                trait_uses,
                properties,
                methods,
            } => {
```

**Location:** Around line 1895-1920 (where ClassDefinition is created and stored)

**Find code similar to:**

```rust
                let class_def = ClassDefinition {
                    name: name.clone(),
                    parent: parent.clone(),
                    properties: all_properties,
                    methods: methods_map,
                    method_visibility: visibility_map,
                };

                self.classes.insert(name.to_lowercase(), class_def);
```

**Update to:**

```rust
                // Validate readonly inheritance: child readonly class can extend non-readonly parent,
                // but we need to ensure all parent properties behave as readonly in child
                if *readonly {
                    if let Some(parent_name) = parent {
                        let parent_name_lower = parent_name.to_lowercase();
                        if let Some(parent_class) = self.classes.get(&parent_name_lower) {
                            // Note: In PHP 8.2, a readonly class CAN extend a non-readonly class
                            // The readonly constraint only applies to the child class's own properties
                            // However, we should track this for proper property access enforcement
                        }
                    }
                }

                let class_def = ClassDefinition {
                    name: name.clone(),
                    readonly: *readonly, // Add this field
                    parent: parent.clone(),
                    properties: all_properties,
                    methods: methods_map,
                    method_visibility: visibility_map,
                };

                self.classes.insert(name.to_lowercase(), class_def);
```

### Step 5: Update Interpreter - Property Write Enforcement (`src/interpreter/mod.rs`)

**Location:** Search for where property assignments are handled in `Expr::PropertySet`

**Find:** The code that handles property modification (likely in `evaluate_expression` or similar)

Use grep to locate:

```bash
grep -n "PropertySet" src/interpreter/mod.rs
```

**Expected location:** Around where property writes are handled

**Add validation:** Before allowing property write, check if:
1. The property is explicitly readonly, OR
2. The class is readonly (making all properties implicitly readonly)

**Code pattern to look for:**

```rust
Expr::PropertySet { object, property, value } => {
    // ... existing code to get object and check property exists

    // ADD THIS CHECK:
    // Check if property is readonly or class is readonly
    let class_name = /* get class name from object */;
    if let Some(class_def) = self.classes.get(&class_name.to_lowercase()) {
        // Check if class is readonly (making all properties readonly)
        if class_def.readonly {
            // Check if we're in constructor context (allowed) or not (error)
            // For now, we'll use the same logic as explicit readonly properties

            // Find the property definition
            for prop in &class_def.properties {
                if prop.name.eq_ignore_ascii_case(property) {
                    // If class is readonly, property is implicitly readonly
                    if self.readonly_properties.get(&object_id).map(|props| props.contains(property)).unwrap_or(false) {
                        return Err(io::Error::other(
                            format!("Cannot modify readonly property {}::${}", class_name, property)
                        ));
                    }
                    break;
                }
            }
        }
    }

    // ... rest of existing property set logic
}
```

**More specific location:** Find where readonly property enforcement happens (added in readonly properties implementation) and extend it to check `class_def.readonly` as well.

### Step 6: Update Interpreter - Object Instantiation (`src/interpreter/mod.rs`)

**Location:** Where `Expr::New` is handled (object instantiation)

**Find code that creates new object instances and initializes readonly_properties tracking:**

**Add logic:** When creating a new object from a readonly class, mark ALL properties as readonly in the `readonly_properties` HashMap after constructor completes.

**Expected location:** After constructor execution in `Expr::New` handler

**Code to add:**

```rust
// After constructor execution, if class is readonly, mark all properties as readonly
if class_def.readonly {
    let property_names: Vec<String> = instance.properties.keys().cloned().collect();
    self.readonly_properties.insert(object_id, property_names);
}
```

**More detailed pattern:**

Look for where constructor is called and readonly properties are tracked after initialization. It should be in the `Expr::New` match arm. After the constructor execution completes and initial readonly properties are set, add:

```rust
// If class itself is readonly (PHP 8.2), all properties are implicitly readonly
if class_def.readonly {
    // Get all property names from the instance
    let all_property_names: Vec<String> = instance.properties.keys()
        .map(|k| k.to_string())
        .collect();

    // Mark all properties as readonly
    let readonly_props = self.readonly_properties.entry(object_id).or_insert_with(Vec::new);
    for prop_name in all_property_names {
        if !readonly_props.contains(&prop_name) {
            readonly_props.push(prop_name);
        }
    }
}
```

### Step 7: Add Tests (`tests/classes/`)

Create the following test files:

#### Test 1: `readonly_class_basic.vhpt`

```php
--TEST--
Readonly class - basic usage with constructor promotion
--FILE--
<?php
readonly class Point {
    public function __construct(
        public float $x,
        public float $y
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

#### Test 2: `readonly_class_modification_error.vhpt`

```php
--TEST--
Readonly class - cannot modify properties after initialization
--FILE--
<?php
readonly class Point {
    public function __construct(
        public float $x,
        public float $y
    ) {}
}

$p = new Point(1.5, 2.5);
$p->x = 3.0;
--EXPECT_ERROR--
Cannot modify readonly property
```

#### Test 3: `readonly_class_explicit_properties.vhpt`

```php
--TEST--
Readonly class - explicit property declarations
--FILE--
<?php
readonly class User {
    public string $name;
    private int $age;

    public function __construct(string $name, int $age) {
        $this->name = $name;
        $this->age = $age;
    }

    public function getAge(): int {
        return $this->age;
    }
}

$user = new User("John", 30);
echo $user->name;
echo "\n";
echo $user->getAge();
--EXPECT--
John
30
```

#### Test 4: `readonly_class_explicit_modifier_error.vhpt`

```php
--TEST--
Readonly class - error when property has explicit readonly modifier
--FILE--
<?php
readonly class Point {
    public readonly float $x;
    public readonly float $y;
}
--EXPECT_ERROR--
cannot have explicit 'readonly' modifier in readonly class
```

#### Test 5: `readonly_class_mixed_properties.vhpt`

```php
--TEST--
Readonly class - all visibility modifiers work
--FILE--
<?php
readonly class Data {
    public string $public_prop;
    protected string $protected_prop;
    private string $private_prop;

    public function __construct() {
        $this->public_prop = "public";
        $this->protected_prop = "protected";
        $this->private_prop = "private";
    }

    public function getProtected(): string {
        return $this->protected_prop;
    }

    public function getPrivate(): string {
        return $this->private_prop;
    }
}

$d = new Data();
echo $d->public_prop;
echo "\n";
echo $d->getProtected();
echo "\n";
echo $d->getPrivate();
--EXPECT--
public
protected
private
```

#### Test 6: `readonly_class_method_modification_error.vhpt`

```php
--TEST--
Readonly class - cannot modify property in method after initialization
--FILE--
<?php
readonly class Counter {
    public int $value;

    public function __construct(int $value) {
        $this->value = $value;
    }

    public function increment(): void {
        $this->value++; // Error: cannot modify readonly property
    }
}

$c = new Counter(5);
$c->increment();
--EXPECT_ERROR--
Cannot modify readonly property
```

#### Test 7: `readonly_class_inheritance_readonly_parent.vhpt`

```php
--TEST--
Readonly class - readonly child can extend readonly parent
--FILE--
<?php
readonly class Point2D {
    public function __construct(
        public float $x,
        public float $y
    ) {}
}

readonly class Point3D extends Point2D {
    public function __construct(
        float $x,
        float $y,
        public float $z
    ) {
        parent::__construct($x, $y);
    }
}

$p = new Point3D(1.0, 2.0, 3.0);
echo $p->x;
echo "\n";
echo $p->y;
echo "\n";
echo $p->z;
--EXPECT--
1
2
3
```

#### Test 8: `readonly_class_inheritance_normal_parent.vhpt`

```php
--TEST--
Readonly class - readonly child can extend non-readonly parent
--FILE--
<?php
class Point2D {
    public function __construct(
        public float $x,
        public float $y
    ) {}
}

readonly class Point3D extends Point2D {
    public function __construct(
        float $x,
        float $y,
        public float $z
    ) {
        parent::__construct($x, $y);
    }
}

$p = new Point3D(1.0, 2.0, 3.0);
echo $p->z;
$p->z = 5.0; // Error: child class is readonly
--EXPECT_ERROR--
Cannot modify readonly property
```

### Step 8: Update Documentation

#### Update `AGENTS.md`

**Location:** Line 395-397 (Phase 5 remaining features)

**Change:**
```markdown
**Remaining for Phase 5 (future):**
- [x] Readonly Classes (PHP 8.2)
- [ ] "Clone with" functionality (PHP 8.5)
```

**Also update:** Line 194 (Current Features list)

**Add:**
```markdown
- [x] Readonly Classes (PHP 8.2)
```

#### Update `README.md`

Find the roadmap section and mark readonly classes as complete. Also add to the features list.

#### Update `docs/features.md`

Add section documenting readonly classes:

```markdown
### Readonly Classes (PHP 8.2)

Readonly classes provide a shorthand way to make all properties in a class readonly:

```php
readonly class Point {
    public function __construct(
        public float $x,
        public float $y
    ) {}
}

$p = new Point(1.5, 2.5);
echo $p->x; // OK
$p->x = 3.0; // Error: Cannot modify readonly property
```

- All properties are implicitly readonly
- Cannot use explicit `readonly` modifier on individual properties
- Properties can have any visibility modifier
- Readonly classes can extend both readonly and non-readonly parent classes
```

#### Update `docs/roadmap.md`

Mark readonly classes as complete in the Phase 5 section.

## Key Considerations

### PHP Compatibility

1. **Implicit readonly**: In a readonly class, ALL properties are implicitly readonly, even without the `readonly` keyword
2. **No explicit readonly**: Properties cannot have explicit `readonly` modifier in a readonly class (redundant and error)
3. **Inheritance**:
   - Readonly class CAN extend non-readonly class (child is more restrictive)
   - Readonly class CAN extend readonly class
   - Non-readonly class CANNOT extend readonly class in PHP 8.2 (but we won't validate this initially)
4. **Constructor behavior**: Properties can be set in constructor (same as explicit readonly properties)
5. **Method behavior**: Properties cannot be modified in methods after initialization (same as explicit readonly)

### Edge Cases

1. **Constructor promotion with readonly class**: Properties don't need `readonly` keyword in constructor parameters
2. **Trait usage**: If a readonly class uses traits, trait properties also become readonly
3. **Parent property access**: When readonly child extends non-readonly parent, only child properties are readonly
4. **Dynamic properties**: Not allowed in readonly classes (but we don't support dynamic properties yet)

### Interaction with Existing Features

1. **Readonly properties**: Extend existing readonly property enforcement to check class-level readonly flag
2. **Constructor promotion**: Works seamlessly with readonly classes
3. **Inheritance**: Validate parent class readonly status when child is readonly
4. **Traits**: Trait properties automatically become readonly when used in readonly class

### Error Messages

Ensure clear error messages for:
- Attempting to modify readonly class property after initialization
- Using explicit `readonly` on property in readonly class
- Any inheritance validation issues (if implemented)

## Test Cases

The test cases above cover:

1. **Basic functionality**: Readonly class with constructor promotion
2. **Error handling**: Cannot modify properties after initialization
3. **Explicit properties**: Readonly class with explicit property declarations
4. **Parser validation**: Error when explicit readonly modifier used
5. **Visibility modifiers**: All visibility levels work with readonly classes
6. **Method modification**: Cannot modify in methods after constructor
7. **Inheritance**: Readonly child with readonly parent
8. **Mixed inheritance**: Readonly child with non-readonly parent

## Reference Implementation

Use existing readonly properties implementation as reference:

- **Token**: `TokenKind::Readonly` already exists (line 43 in `src/token.rs`)
- **Lexer**: Already recognizes `readonly` keyword
- **AST Property**: `Property::readonly` field exists (line 18 in `src/ast/stmt.rs`)
- **Parser**: Readonly parsing pattern in `parse_class` (lines 1203-1219 in `src/parser/stmt.rs`)
- **Interpreter**: Readonly tracking with `readonly_properties: HashMap<usize, Vec<String>>` in Interpreter struct
- **Enforcement**: Property write checking in `Expr::PropertySet` handler

The implementation should follow the same patterns but apply readonly at the class level, affecting all properties automatically.

## Implementation Checklist

- [ ] Add `readonly` field to `Stmt::Class` in AST
- [ ] Parse `readonly` modifier before `class` keyword in parser
- [ ] Validate properties don't have explicit `readonly` in readonly classes
- [ ] Add `readonly` field to `ClassDefinition` in interpreter
- [ ] Store readonly flag when registering class
- [ ] Extend property write enforcement to check class-level readonly
- [ ] Mark all properties as readonly during object instantiation for readonly classes
- [ ] Add 8 comprehensive test files
- [ ] Update AGENTS.md documentation
- [ ] Update README.md
- [ ] Update docs/features.md
- [ ] Update docs/roadmap.md
- [ ] Run full test suite to ensure no regressions
- [ ] Verify clippy passes with no warnings
