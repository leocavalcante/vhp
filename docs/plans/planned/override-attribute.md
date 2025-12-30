# Plan: #[\Override] Attribute (PHP 8.3)

## Overview

The `#[\Override]` attribute explicitly marks methods that override parent class or interface methods. The PHP engine validates that the method actually overrides something, catching typos and refactoring errors at runtime.

**PHP Example:**
```php
<?php
// Without #[\Override]
class Animal {
    public function makeSound() {
        return "...";
    }
}

class Dog extends Animal {
    // Typo! Should be makeSound, but no error
    public function makeSounds() {
        return "Woof!";
    }
}

// With #[\Override] (PHP 8.3+)
class Cat extends Animal {
    #[\Override]
    public function makeSound() {
        return "Meow!"; // OK: actually overrides
    }
}

class Bird extends Animal {
    #[\Override]
    public function makeSounds() { // Error: doesn't override anything
        return "Tweet!";
    }
}
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/interpreter/stmt_exec/definitions.rs` | Validate #[\Override] when defining classes |
| `src/interpreter/objects/classes.rs` | Helper methods to check if method overrides |
| `tests/classes/override_*.vhpt` | Test files |

## Implementation Steps

### Step 1: Understand Existing Attribute Infrastructure

VHP already has:
- Attribute parsing (PHP 8.0)
- Attribute storage in AST (`attributes: Vec<Attribute>` on methods)
- Attribute reflection API

No changes needed to token.rs, lexer, or parser - attributes already work!

### Step 2: Add Validation on Class Definition (`src/interpreter/stmt_exec/definitions.rs`)

Find where classes are defined (in `execute_class_declaration` or similar):

```rust
// After storing the class in self.classes, validate #[\Override] attributes
fn execute_class_declaration(&mut self, class: &ClassDecl) -> Result<(), String> {
    // ... existing class storage logic ...

    // Validate #[\Override] attributes on methods
    for method in &class.methods {
        self.validate_override_attribute(class, method)?;
    }

    Ok(())
}
```

### Step 3: Implement Override Validation

Add validation logic in `src/interpreter/objects/classes.rs` or `definitions.rs`:

```rust
impl<W: Write> Interpreter<W> {
    /// Validate that methods marked with #[\Override] actually override something
    fn validate_override_attribute(
        &self,
        class: &ClassDecl,
        method: &Method,
    ) -> Result<(), String> {
        // Check if method has #[\Override] attribute
        let has_override = method.attributes.iter().any(|attr| {
            attr.name.eq_ignore_ascii_case("Override")
        });

        if !has_override {
            return Ok(()); // No validation needed
        }

        // Method must override something from:
        // 1. Parent class
        // 2. Implemented interfaces
        // 3. Used traits

        let method_name_lower = method.name.to_lowercase();
        let mut found_in_parent = false;
        let mut found_in_interface = false;
        let mut found_in_trait = false;

        // Check parent class
        if let Some(parent_name) = &class.parent {
            if self.class_has_method(parent_name, &method_name_lower)? {
                found_in_parent = true;
            }
        }

        // Check interfaces
        for interface_name in &class.interfaces {
            if self.interface_has_method(interface_name, &method_name_lower)? {
                found_in_interface = true;
                break;
            }
        }

        // Check traits
        for trait_name in &class.traits {
            if self.trait_has_method(trait_name, &method_name_lower)? {
                found_in_trait = true;
                break;
            }
        }

        // If not found anywhere, error
        if !found_in_parent && !found_in_interface && !found_in_trait {
            return Err(format!(
                "{}::{} has #[\\Override] attribute, but no matching parent method exists",
                class.name, method.name
            ));
        }

        Ok(())
    }

    /// Check if a class (including ancestors) has a method
    fn class_has_method(&self, class_name: &str, method_name: &str) -> Result<bool, String> {
        let class_key = class_name.to_lowercase();

        let class_def = self.classes.get(&class_key)
            .ok_or_else(|| format!("Class '{}' not found", class_name))?;

        // Check if method exists in this class
        if class_def.methods.iter().any(|m| m.name.to_lowercase() == method_name) {
            return Ok(true);
        }

        // Check parent class recursively
        if let Some(parent_name) = &class_def.parent {
            return self.class_has_method(parent_name, method_name);
        }

        Ok(false)
    }

    /// Check if an interface (including parent interfaces) has a method
    fn interface_has_method(&self, interface_name: &str, method_name: &str) -> Result<bool, String> {
        let interface_key = interface_name.to_lowercase();

        let interface_def = self.interfaces.get(&interface_key)
            .ok_or_else(|| format!("Interface '{}' not found", interface_name))?;

        // Check if method exists in this interface
        if interface_def.methods.iter().any(|m| m.name.to_lowercase() == method_name) {
            return Ok(true);
        }

        // Check parent interfaces recursively
        for parent_interface in &interface_def.extends {
            if self.interface_has_method(parent_interface, method_name)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Check if a trait has a method
    fn trait_has_method(&self, trait_name: &str, method_name: &str) -> Result<bool, String> {
        let trait_key = trait_name.to_lowercase();

        let trait_def = self.traits.get(&trait_key)
            .ok_or_else(|| format!("Trait '{}' not found", trait_name))?;

        // Check if method exists in this trait
        if trait_def.methods.iter().any(|m| m.name.to_lowercase() == method_name) {
            return Ok(true);
        }

        // Check traits used by this trait
        for nested_trait_name in &trait_def.traits {
            if self.trait_has_method(nested_trait_name, method_name)? {
                return Ok(true);
            }
        }

        Ok(false)
    }
}
```

### Step 4: Add Tests

**tests/classes/override_basic.vhpt**
```
--TEST--
Basic #[\Override] attribute on valid override
--FILE--
<?php
class Animal {
    public function speak() {
        return "...";
    }
}

class Dog extends Animal {
    #[\Override]
    public function speak() {
        return "Woof!";
    }
}

$dog = new Dog();
echo $dog->speak();
--EXPECT--
Woof!
```

**tests/classes/override_error.vhpt**
```
--TEST--
#[\Override] attribute error when method doesn't override
--FILE--
<?php
class Animal {
    public function speak() {
        return "...";
    }
}

class Dog extends Animal {
    #[\Override]
    public function bark() {
        return "Woof!";
    }
}
--EXPECT_ERROR--
has #[\Override] attribute, but no matching parent method exists
```

**tests/classes/override_interface.vhpt**
```
--TEST--
#[\Override] attribute validates interface methods
--FILE--
<?php
interface Drawable {
    public function draw();
}

class Circle implements Drawable {
    #[\Override]
    public function draw() {
        return "Drawing circle";
    }
}

$circle = new Circle();
echo $circle->draw();
--EXPECT--
Drawing circle
```

**tests/classes/override_interface_error.vhpt**
```
--TEST--
#[\Override] error when interface method doesn't exist
--FILE--
<?php
interface Drawable {
    public function draw();
}

class Circle implements Drawable {
    public function draw() {
        return "Drawing circle";
    }

    #[\Override]
    public function paint() {
        return "Painting";
    }
}
--EXPECT_ERROR--
has #[\Override] attribute, but no matching parent method exists
```

**tests/classes/override_trait.vhpt**
```
--TEST--
#[\Override] attribute validates trait methods
--FILE--
<?php
trait Greetable {
    public function greet() {
        return "Hello";
    }
}

class Person {
    use Greetable;

    #[\Override]
    public function greet() {
        return "Hi there!";
    }
}

$person = new Person();
echo $person->greet();
--EXPECT--
Hi there!
```

**tests/classes/override_no_parent.vhpt**
```
--TEST--
#[\Override] error when class has no parent
--FILE--
<?php
class Standalone {
    #[\Override]
    public function test() {
        return "fail";
    }
}
--EXPECT_ERROR--
has #[\Override] attribute, but no matching parent method exists
```

**tests/classes/override_grandparent.vhpt**
```
--TEST--
#[\Override] validates against grandparent methods
--FILE--
<?php
class GrandParent {
    public function legacy() {
        return "old";
    }
}

class Parent extends GrandParent {
}

class Child extends Parent {
    #[\Override]
    public function legacy() {
        return "new";
    }
}

$child = new Child();
echo $child->legacy();
--EXPECT--
new
```

**tests/classes/override_case_insensitive.vhpt**
```
--TEST--
#[\Override] is case-insensitive
--FILE--
<?php
class Base {
    public function Method() {
        return "base";
    }
}

class Child extends Base {
    #[\override]  // lowercase
    public function method() {
        return "child";
    }
}

$child = new Child();
echo $child->method();
--EXPECT--
child
```

**tests/classes/override_abstract.vhpt**
```
--TEST--
#[\Override] with abstract method implementation
--FILE--
<?php
abstract class Shape {
    abstract public function area();
}

class Circle extends Shape {
    private $radius = 5;

    #[\Override]
    public function area() {
        return 3.14 * $this->radius * $this->radius;
    }
}

$circle = new Circle();
echo $circle->area();
--EXPECT--
78.5
```

**tests/classes/override_multiple_interfaces.vhpt**
```
--TEST--
#[\Override] with method from one of multiple interfaces
--FILE--
<?php
interface A {
    public function foo();
}

interface B {
    public function bar();
}

class C implements A, B {
    #[\Override]
    public function foo() {
        return "foo";
    }

    #[\Override]
    public function bar() {
        return "bar";
    }
}

$c = new C();
echo $c->foo() . $c->bar();
--EXPECT--
foobar
```

**tests/classes/override_magic_method.vhpt**
```
--TEST--
#[\Override] on magic method override
--FILE--
<?php
class Base {
    public function __toString() {
        return "Base";
    }
}

class Child extends Base {
    #[\Override]
    public function __toString() {
        return "Child";
    }
}

$child = new Child();
echo $child;
--EXPECT--
Child
```

## PHP Compatibility Notes

| Feature | PHP Version |
|---------|-------------|
| #[\Override] attribute | 8.3 |
| Validates parent class methods | 8.3 |
| Validates interface methods | 8.3 |
| Validates trait methods | 8.3 |
| Validates abstract methods | 8.3 |
| Case-insensitive | 8.3 |

## Key Considerations

1. **Validation timing**: Check at class definition time, not method call time
2. **Inheritance chain**: Must check entire parent chain, not just immediate parent
3. **Interfaces**: Must check all implemented interfaces and their parents
4. **Traits**: Must check all used traits and their nested traits
5. **Abstract methods**: Implementing abstract methods counts as overriding
6. **Case insensitivity**: Method name matching is case-insensitive
7. **Magic methods**: Can use #[\Override] on magic methods if parent defines them
8. **Private methods**: Cannot override private methods (PHP doesn't allow it anyway)

## Override Sources

A method marked with #[\Override] must exist in one of these:

1. **Parent class** (any ancestor)
2. **Implemented interface** (any interface in the hierarchy)
3. **Used trait** (any trait or nested trait)

## Error Messages

- `<Class>::<method> has #[\Override] attribute, but no matching parent method exists`

## Implementation Order

1. Add validation function `validate_override_attribute()`
2. Add helper methods: `class_has_method()`, `interface_has_method()`, `trait_has_method()`
3. Call validation when class is defined
4. Add tests for all scenarios

## Edge Cases

1. **Method in trait, overridden in class**: Valid use of #[\Override]
2. **Method in parent interface**: Valid if class implements that interface
3. **Method in grandparent**: Valid, must check entire chain
4. **Private parent method**: Cannot override (PHP restriction), so #[\Override] should fail
5. **Static methods**: Can use #[\Override] on static method overrides
6. **Constructor**: Can use #[\Override] on `__construct` if parent has it
7. **Final methods**: Cannot override anyway (PHP restriction)
8. **Trait conflict resolution**: If trait method is overridden, #[\Override] is valid

## Reference Implementations

- Attribute parsing: Already implemented (PHP 8.0)
- Attribute storage: `attributes: Vec<Attribute>` on Method struct
- Class hierarchy: `parent: Option<String>` on ClassDecl
- Interface hierarchy: `extends: Vec<String>` on InterfaceDecl
- Method lookup: Already used for method calls

## Performance Considerations

- Validation happens at class definition time (once), not on every method call
- Recursive lookups through inheritance chain, but typically shallow
- Attribute check is simple string comparison
- No runtime overhead after class is defined

## Additional Notes

The #[\Override] attribute:
- Is a built-in attribute (doesn't need to be declared as a class)
- Has no parameters
- Can only be applied to methods
- Is primarily a developer safety feature (catches refactoring errors)
- Complements other safety features like type hints and final

Common use cases:
- Catch typos in method names during refactoring
- Document intent that a method overrides parent
- Ensure child classes stay in sync with parent API changes
