# Plan: Constants in Traits (PHP 8.2)

## Overview

PHP 8.2 allows traits to define constants, just like classes and interfaces. These constants are accessible via the class that uses the trait, and can be overridden in the class if needed.

**PHP Example:**
```php
<?php
// Before PHP 8.2: Traits couldn't define constants
trait OldTrait {
    // Not allowed: const VERSION = '1.0';
    // Had to use methods instead
    public function getVersion() {
        return '1.0';
    }
}

// PHP 8.2+: Traits can define constants
trait Config {
    const VERSION = '1.0';
    const PREFIX = 'APP_';

    public function getConfig() {
        return self::PREFIX . self::VERSION;
    }
}

class Application {
    use Config;
}

echo Application::VERSION; // 1.0
echo Application::PREFIX;   // APP_

$app = new Application();
echo $app->getConfig(); // APP_1.0
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/ast/stmt.rs` | Add `constants: Vec<Constant>` to `TraitDecl` |
| `src/parser/stmt.rs` | Parse const declarations inside traits |
| `src/interpreter/stmt_exec/definitions.rs` | Handle trait constants when trait is used |
| `src/interpreter/objects/classes.rs` | Include trait constants in class constants |
| `tests/traits/trait_const_*.vhpt` | Test files |

## Implementation Steps

### Step 1: Add Constants to TraitDecl (`src/ast/stmt.rs`)

Find the `TraitDecl` struct and add a constants field:

```rust
/// Trait declaration
#[derive(Debug, Clone)]
pub struct TraitDecl {
    pub name: String,
    pub properties: Vec<Property>,
    pub methods: Vec<Method>,
    pub constants: Vec<Constant>,  // NEW: PHP 8.2+
    pub traits: Vec<String>,       // Traits used by this trait
    pub attributes: Vec<Attribute>,
}
```

If `Constant` doesn't exist yet, add it:

```rust
/// Class/Interface/Trait constant
#[derive(Debug, Clone)]
pub struct Constant {
    pub name: String,
    pub value: Expr,
    pub visibility: Visibility,  // PHP 7.1+
    pub is_final: bool,          // PHP 8.1+
    pub attributes: Vec<Attribute>, // PHP 8.0+
}
```

### Step 2: Parse Constants in Traits (`src/parser/stmt.rs`)

Update trait parsing to handle const declarations. Find `parse_trait()` method:

```rust
fn parse_trait(&mut self, attributes: Vec<Attribute>) -> Result<Stmt, String> {
    self.expect(&TokenKind::Trait)?;

    let name = self.expect_identifier()?;

    self.expect(&TokenKind::LeftBrace)?;

    let mut properties = vec![];
    let mut methods = vec![];
    let mut constants = vec![];  // NEW
    let mut trait_uses = vec![];

    while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
        // Parse attributes if present
        let member_attributes = self.parse_attributes()?;

        match &self.current_token().kind {
            TokenKind::Use => {
                // Trait using another trait
                let used_traits = self.parse_trait_use()?;
                trait_uses.extend(used_traits);
            }
            TokenKind::Const => {
                // NEW: Parse constant
                let constant = self.parse_constant(member_attributes)?;
                constants.push(constant);
            }
            TokenKind::Public | TokenKind::Protected | TokenKind::Private => {
                // Parse visibility
                let visibility = self.parse_visibility()?;

                if self.check(&TokenKind::Function) {
                    // Method
                    let method = self.parse_method(visibility, member_attributes)?;
                    methods.push(method);
                } else if self.check(&TokenKind::Const) {
                    // NEW: Constant with visibility
                    let constant = self.parse_constant_with_visibility(
                        visibility,
                        member_attributes,
                    )?;
                    constants.push(constant);
                } else {
                    // Property
                    let property = self.parse_class_property(visibility, member_attributes)?;
                    properties.push(property);
                }
            }
            TokenKind::Function => {
                // Public method (no visibility keyword)
                let method = self.parse_method(Visibility::Public, member_attributes)?;
                methods.push(method);
            }
            _ => {
                return Err(format!(
                    "Unexpected token in trait body: {:?}",
                    self.current_token()
                ));
            }
        }
    }

    self.expect(&TokenKind::RightBrace)?;

    Ok(Stmt::TraitDecl(TraitDecl {
        name,
        properties,
        methods,
        constants,  // NEW
        traits: trait_uses,
        attributes,
    }))
}
```

If `parse_constant()` doesn't exist yet, add it:

```rust
fn parse_constant(&mut self, attributes: Vec<Attribute>) -> Result<Constant, String> {
    self.expect(&TokenKind::Const)?;

    let name = self.expect_identifier()?;

    self.expect(&TokenKind::Assign)?;

    let value = self.parse_expression()?;

    self.expect(&TokenKind::Semicolon)?;

    Ok(Constant {
        name,
        value,
        visibility: Visibility::Public, // Default visibility
        is_final: false,
        attributes,
    })
}

fn parse_constant_with_visibility(
    &mut self,
    visibility: Visibility,
    attributes: Vec<Attribute>,
) -> Result<Constant, String> {
    // Check for 'final' keyword
    let is_final = if self.check(&TokenKind::Final) {
        self.advance();
        true
    } else {
        false
    };

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

### Step 3: Include Trait Constants in Classes (`src/interpreter/stmt_exec/definitions.rs`)

When a class uses a trait, copy the trait's constants to the class. Update class definition:

```rust
fn execute_class_declaration(&mut self, class: &ClassDecl) -> Result<(), String> {
    // ... existing code to build class definition ...

    // Collect constants from traits
    let mut trait_constants = vec![];

    for trait_name in &class.traits {
        let trait_key = trait_name.to_lowercase();

        if let Some(trait_def) = self.traits.get(&trait_key) {
            // Add trait constants to the class
            for constant in &trait_def.constants {
                trait_constants.push(constant.clone());
            }
        }
    }

    // Merge trait constants with class constants
    // Class constants override trait constants with same name
    let mut all_constants = trait_constants;

    for class_const in &class.constants {
        // Remove any trait constant with same name
        all_constants.retain(|c| !c.name.eq_ignore_ascii_case(&class_const.name));
        // Add class constant
        all_constants.push(class_const.clone());
    }

    // Store constants in class definition
    let mut class_def = /* ... build class definition ... */;
    class_def.constants = all_constants;

    // ... store class_def in self.classes ...

    Ok(())
}
```

### Step 4: Access Trait Constants

Constant access is already implemented for classes. Ensure `ClassName::CONSTANT` works when the constant comes from a trait:

```rust
// In constant access evaluation (likely in expr_eval/mod.rs)
fn evaluate_constant_access(&mut self, class: &str, constant: &str) -> Result<Value, String> {
    let class_key = class.to_lowercase();

    let class_def = self.classes.get(&class_key)
        .ok_or_else(|| format!("Class '{}' not found", class))?;

    // Find constant (could be from class or from trait)
    let const_def = class_def.constants.iter()
        .find(|c| c.name.eq_ignore_ascii_case(constant))
        .ok_or_else(|| {
            format!("Undefined constant {}::{}", class, constant)
        })?;

    // Evaluate constant value
    self.evaluate(&const_def.value)
}
```

### Step 5: Handle Trait Constant Conflicts

If multiple traits define the same constant, it's an error unless the class explicitly defines it:

```rust
fn check_trait_constant_conflicts(&self, class: &ClassDecl) -> Result<(), String> {
    let mut constant_sources: HashMap<String, Vec<String>> = HashMap::new();

    // Track which traits define each constant
    for trait_name in &class.traits {
        let trait_key = trait_name.to_lowercase();

        if let Some(trait_def) = self.traits.get(&trait_key) {
            for constant in &trait_def.constants {
                let const_key = constant.name.to_lowercase();
                constant_sources
                    .entry(const_key)
                    .or_insert_with(Vec::new)
                    .push(trait_name.clone());
            }
        }
    }

    // Check for conflicts
    for (const_name, sources) in &constant_sources {
        if sources.len() > 1 {
            // Check if class defines this constant (resolves conflict)
            let class_defines = class.constants.iter()
                .any(|c| c.name.eq_ignore_ascii_case(const_name));

            if !class_defines {
                return Err(format!(
                    "Class {} inherits constant {} from multiple traits: {}",
                    class.name,
                    const_name,
                    sources.join(", ")
                ));
            }
        }
    }

    Ok(())
}
```

### Step 6: Add Tests

**tests/traits/trait_const_basic.vhpt**
```
--TEST--
Basic trait constant
--FILE--
<?php
trait Config {
    const VERSION = '1.0';
    const NAME = 'App';
}

class Application {
    use Config;
}

echo Application::VERSION . "\n";
echo Application::NAME;
--EXPECT--
1.0
App
```

**tests/traits/trait_const_access_in_method.vhpt**
```
--TEST--
Access trait constant from trait method
--FILE--
<?php
trait Versioned {
    const VERSION = '2.0';

    public function getVersion() {
        return self::VERSION;
    }
}

class Product {
    use Versioned;
}

$product = new Product();
echo $product->getVersion();
--EXPECT--
2.0
```

**tests/traits/trait_const_override.vhpt**
```
--TEST--
Class can override trait constant
--FILE--
<?php
trait Config {
    const ENV = 'dev';
}

class App {
    use Config;

    const ENV = 'prod'; // Override
}

echo App::ENV;
--EXPECT--
prod
```

**tests/traits/trait_const_visibility.vhpt**
```
--TEST--
Trait constant with visibility modifiers
--FILE--
<?php
trait Data {
    public const PUBLIC_VALUE = 1;
    protected const PROTECTED_VALUE = 2;
    private const PRIVATE_VALUE = 3;

    public function getValues() {
        return self::PUBLIC_VALUE . self::PROTECTED_VALUE . self::PRIVATE_VALUE;
    }
}

class Container {
    use Data;
}

echo Container::PUBLIC_VALUE . "\n";
$c = new Container();
echo $c->getValues();
--EXPECT--
1
123
```

**tests/traits/trait_const_multiple_traits.vhpt**
```
--TEST--
Multiple traits with different constants
--FILE--
<?php
trait T1 {
    const VALUE1 = 'A';
}

trait T2 {
    const VALUE2 = 'B';
}

class Combined {
    use T1, T2;
}

echo Combined::VALUE1 . Combined::VALUE2;
--EXPECT--
AB
```

**tests/traits/trait_const_conflict_error.vhpt**
```
--TEST--
Error when multiple traits define same constant
--FILE--
<?php
trait T1 {
    const VALUE = 'A';
}

trait T2 {
    const VALUE = 'B';
}

class Conflict {
    use T1, T2;
}
--EXPECT_ERROR--
inherits constant VALUE from multiple traits
```

**tests/traits/trait_const_conflict_resolved.vhpt**
```
--TEST--
Resolve constant conflict by defining in class
--FILE--
<?php
trait T1 {
    const VALUE = 'A';
}

trait T2 {
    const VALUE = 'B';
}

class Resolved {
    use T1, T2;

    const VALUE = 'C'; // Resolves conflict
}

echo Resolved::VALUE;
--EXPECT--
C
```

**tests/traits/trait_const_nested.vhpt**
```
--TEST--
Trait using another trait's constant
--FILE--
<?php
trait Base {
    const PREFIX = 'BASE_';
}

trait Extended {
    use Base;

    const SUFFIX = '_END';

    public function getFull() {
        return self::PREFIX . 'middle' . self::SUFFIX;
    }
}

class Final {
    use Extended;
}

$obj = new Final();
echo $obj->getFull();
--EXPECT--
BASE_middle_END
```

**tests/traits/trait_const_final.vhpt**
```
--TEST--
Final trait constant (PHP 8.1+)
--FILE--
<?php
trait Config {
    final const VERSION = '1.0';
}

class App {
    use Config;

    const VERSION = '2.0'; // Try to override final constant
}
--EXPECT_ERROR--
Cannot override final constant
```

**tests/traits/trait_const_in_expression.vhpt**
```
--TEST--
Use trait constant in expressions
--FILE--
<?php
trait Math {
    const PI = 3.14159;
}

class Calculator {
    use Math;

    public function circumference($radius) {
        return 2 * self::PI * $radius;
    }
}

$calc = new Calculator();
echo $calc->circumference(10);
--EXPECT--
62.8318
```

## PHP Compatibility Notes

| Feature | PHP Version |
|---------|-------------|
| Trait constants | 8.2 |
| Constant visibility (public/protected/private) | 7.1 (for classes), 8.2 (for traits) |
| Final constants | 8.1 |
| Constant attributes | 8.0 |

## Key Considerations

1. **Inheritance**: Trait constants are copied to the class that uses the trait
2. **Override**: Class constants override trait constants with the same name
3. **Conflicts**: Multiple traits with same constant name cause error (unless class defines it)
4. **Visibility**: Trait constants support public/protected/private (PHP 7.1+)
5. **Final**: Trait constants can be final (PHP 8.1+), preventing override
6. **Access**: Access via `ClassName::CONSTANT` or `self::CONSTANT` inside class
7. **Nested traits**: Trait using another trait inherits its constants

## Conflict Resolution Rules

When multiple traits define the same constant:

1. **Error if**: Class doesn't define the constant
2. **OK if**: Class defines the constant (overrides both trait constants)
3. **No `insteadof`**: Unlike methods, constants don't use `insteadof` syntax

## Error Messages

- `Class X inherits constant Y from multiple traits: T1, T2`
- `Cannot override final constant X::Y`
- `Undefined constant X::Y`
- `Cannot access private constant X::Y`

## Implementation Order

1. Add `constants` field to TraitDecl
2. Parse const declarations in traits
3. Copy trait constants to class when trait is used
4. Handle constant conflicts
5. Validate final constant overrides
6. Add tests for all scenarios

## Edge Cases

1. **Same constant in parent and trait**: Trait constant takes precedence
2. **Three+ traits with same constant**: Error unless class defines it
3. **Nested trait constants**: Should be inherited transitively
4. **Private trait constant**: Accessible in class that uses trait
5. **Final constant override**: Should error
6. **Constant in expression**: Should be evaluated when needed
7. **Case insensitivity**: Constant names are case-sensitive

## Reference Implementations

- Class constants: Already implemented
- Interface constants: Already implemented
- Trait parsing: `src/parser/stmt.rs`
- Trait usage: `src/interpreter/stmt_exec/definitions.rs`
- Constant access: `src/interpreter/expr_eval/mod.rs`
