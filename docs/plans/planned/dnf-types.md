# Plan: DNF Types (PHP 8.2)

## Overview

DNF (Disjunctive Normal Form) types allow complex type declarations combining union (`|`) and intersection (`&`) types in a normalized form: `(A&B)|C` or `(A&B)|(C&D)`. This enables precise type constraints like "either an object that implements both interfaces A and B, or just interface C".

**PHP Example:**
```php
<?php
// Before DNF (PHP < 8.2): Cannot express this constraint
interface Loggable {
    public function log();
}

interface Serializable {
    public function serialize();
}

interface Cacheable {
    public function cache();
}

// With DNF (PHP 8.2+): Accept either (Loggable AND Serializable) OR Cacheable
function process((Loggable&Serializable)|Cacheable $obj): void {
    if ($obj instanceof Loggable) {
        $obj->log();
    }
    if ($obj instanceof Cacheable) {
        $obj->cache();
    }
}

// Valid calls:
class A implements Loggable, Serializable {
    public function log() {}
    public function serialize() {}
}

class B implements Cacheable {
    public function cache() {}
}

process(new A()); // OK: matches (Loggable&Serializable)
process(new B()); // OK: matches Cacheable
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/ast/stmt.rs` | Add `DNF` variant to `TypeHint` enum |
| `src/parser/types.rs` | Parse DNF type syntax with parentheses |
| `src/interpreter/types.rs` | Validate DNF types at runtime |
| `src/interpreter/value.rs` | Check if value matches DNF type |
| `tests/types/dnf_*.vhpt` | Test files |

## Implementation Steps

### Step 1: Update TypeHint AST (`src/ast/stmt.rs`)

Find the `TypeHint` enum (around line 59-80) and add DNF variant:

```rust
/// Type hint for parameters and return values
#[derive(Debug, Clone, PartialEq)]
pub enum TypeHint {
    /// Simple type: int, string, float, bool, array, object, callable, mixed
    Simple(String),
    /// Nullable type: ?int, ?string, etc.
    Nullable(Box<TypeHint>),
    /// Union type (PHP 8.0+): int|string, int|null
    Union(Vec<TypeHint>),
    /// Intersection type (PHP 8.1+): Iterator&Countable
    Intersection(Vec<TypeHint>),
    /// DNF type (PHP 8.2+): (A&B)|C, (A&B)|(C&D)
    /// Each element is an intersection (Vec<TypeHint>), the outer vec is a union
    DNF(Vec<Vec<TypeHint>>),
    /// Class/interface type
    Class(String),
    /// void (only for return types)
    Void,
    /// never (PHP 8.1+, only for return types)
    Never,
    /// static (PHP 8.0+, only for return types)
    Static,
    /// self/parent (in class context)
    SelfType,
    ParentType,
}
```

### Step 2: Parse DNF Types (`src/parser/types.rs` or in `parser/mod.rs`)

Create or update type parsing to handle parentheses and DNF:

```rust
impl Parser {
    /// Parse a type hint (simple, union, intersection, DNF, nullable)
    pub fn parse_type_hint(&mut self) -> Result<TypeHint, String> {
        // Check for nullable prefix: ?Type
        if self.check(&TokenKind::Question) {
            self.advance();
            let inner = self.parse_non_nullable_type()?;
            return Ok(TypeHint::Nullable(Box::new(inner)));
        }

        self.parse_non_nullable_type()
    }

    fn parse_non_nullable_type(&mut self) -> Result<TypeHint, String> {
        // Parse first component (could be parenthesized intersection)
        let first = self.parse_type_component()?;

        // Check for union operator: |
        if self.check(&TokenKind::Pipe) {
            return self.parse_union_or_dnf(first);
        }

        Ok(first)
    }

    fn parse_type_component(&mut self) -> Result<TypeHint, String> {
        // Check for parenthesized intersection: (A&B)
        if self.check(&TokenKind::LeftParen) {
            self.advance(); // consume '('
            let intersection = self.parse_intersection()?;
            self.expect(&TokenKind::RightParen)?;

            // An intersection in parentheses
            return Ok(TypeHint::Intersection(intersection));
        }

        // Parse single type
        self.parse_single_type()
    }

    fn parse_union_or_dnf(&mut self, first: TypeHint) -> Result<TypeHint, String> {
        // We have: <first> |
        // Could be: A|B (union) or (A&B)|C (DNF) or (A&B)|(C&D) (DNF)

        let mut components = vec![first.clone()];
        let mut has_intersection = matches!(first, TypeHint::Intersection(_));

        while self.check(&TokenKind::Pipe) {
            self.advance(); // consume '|'

            let component = self.parse_type_component()?;

            if matches!(component, TypeHint::Intersection(_)) {
                has_intersection = true;
            }

            components.push(component);
        }

        // Determine if this is DNF or simple union
        if has_intersection {
            // Convert to DNF format: Vec<Vec<TypeHint>>
            let dnf_components: Result<Vec<Vec<TypeHint>>, String> = components
                .into_iter()
                .map(|comp| match comp {
                    TypeHint::Intersection(types) => Ok(types),
                    // Single type is intersection of one
                    other => Ok(vec![other]),
                })
                .collect();

            Ok(TypeHint::DNF(dnf_components?))
        } else {
            // Simple union
            Ok(TypeHint::Union(components))
        }
    }

    fn parse_intersection(&mut self) -> Result<Vec<TypeHint>, String> {
        let mut types = vec![self.parse_single_type()?];

        while self.check(&TokenKind::Ampersand) {
            self.advance(); // consume '&'
            types.push(self.parse_single_type()?);
        }

        Ok(types)
    }

    fn parse_single_type(&mut self) -> Result<TypeHint, String> {
        match &self.current_token().kind {
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();

                match name.to_lowercase().as_str() {
                    "int" | "string" | "float" | "bool" | "array" | "object"
                    | "callable" | "iterable" | "mixed" | "null" => {
                        Ok(TypeHint::Simple(name.to_lowercase()))
                    }
                    "void" => Ok(TypeHint::Void),
                    "never" => Ok(TypeHint::Never),
                    "self" => Ok(TypeHint::SelfType),
                    "parent" => Ok(TypeHint::ParentType),
                    "static" => Ok(TypeHint::Static),
                    _ => Ok(TypeHint::Class(name)),
                }
            }
            _ => Err("Expected type name".to_string()),
        }
    }
}
```

### Step 3: Validate DNF Types at Runtime (`src/interpreter/types.rs`)

Update type validation to handle DNF:

```rust
impl<W: Write> Interpreter<W> {
    /// Validate that a value matches a type hint
    pub fn validate_type(&self, value: &Value, type_hint: &TypeHint) -> Result<(), String> {
        match type_hint {
            TypeHint::DNF(intersections) => {
                // DNF: (A&B)|(C&D)|E
                // Value must match at least one intersection group
                for intersection_group in intersections {
                    if self.matches_intersection(value, intersection_group) {
                        return Ok(()); // Found a matching group
                    }
                }

                // None of the intersection groups matched
                Err(format!(
                    "Expected type {}, got {}",
                    self.format_dnf_type(intersections),
                    value.type_name()
                ))
            }
            // ... existing type validation for other variants ...
            _ => self.validate_type_existing(value, type_hint),
        }
    }

    /// Check if value matches all types in an intersection
    fn matches_intersection(&self, value: &Value, types: &[TypeHint]) -> bool {
        types.iter().all(|t| self.value_matches_type(value, t))
    }

    /// Check if a value matches a single type (no error, just bool)
    fn value_matches_type(&self, value: &Value, type_hint: &TypeHint) -> bool {
        match type_hint {
            TypeHint::Simple(name) => match (name.as_str(), value) {
                ("int", Value::Integer(_)) => true,
                ("float", Value::Float(_)) | ("float", Value::Integer(_)) => true,
                ("string", Value::String(_)) => true,
                ("bool", Value::Bool(_)) => true,
                ("array", Value::Array(_)) => true,
                ("object", Value::Object(_)) => true,
                ("null", Value::Null) => true,
                ("mixed", _) => true,
                _ => false,
            },
            TypeHint::Class(class_name) => {
                self.value_matches_class(value, class_name)
            }
            TypeHint::Union(types) => {
                types.iter().any(|t| self.value_matches_type(value, t))
            }
            TypeHint::Intersection(types) => {
                types.iter().all(|t| self.value_matches_type(value, t))
            }
            TypeHint::Nullable(inner) => {
                matches!(value, Value::Null) || self.value_matches_type(value, inner)
            }
            TypeHint::DNF(intersections) => {
                intersections.iter().any(|group| self.matches_intersection(value, group))
            }
            _ => false,
        }
    }

    /// Check if value matches a class/interface type
    fn value_matches_class(&self, value: &Value, class_name: &str) -> bool {
        match value {
            Value::Object(obj) => {
                let obj_ref = obj.borrow();
                self.is_instance_of(&obj_ref.class_name, class_name)
            }
            _ => false,
        }
    }

    /// Check if a class is an instance of another class/interface (including hierarchy)
    fn is_instance_of(&self, object_class: &str, target_class: &str) -> bool {
        if object_class.eq_ignore_ascii_case(target_class) {
            return true;
        }

        // Check if object_class extends or implements target_class
        if let Some(class_def) = self.classes.get(&object_class.to_lowercase()) {
            // Check parent chain
            if let Some(parent) = &class_def.parent {
                if self.is_instance_of(parent, target_class) {
                    return true;
                }
            }

            // Check interfaces
            for interface in &class_def.interfaces {
                if self.is_instance_of(interface, target_class) {
                    return true;
                }
            }
        }

        // Check if it's an interface extending another interface
        if let Some(interface_def) = self.interfaces.get(&object_class.to_lowercase()) {
            for parent_interface in &interface_def.extends {
                if self.is_instance_of(parent_interface, target_class) {
                    return true;
                }
            }
        }

        false
    }

    /// Format DNF type for error messages
    fn format_dnf_type(&self, intersections: &[Vec<TypeHint>]) -> String {
        intersections
            .iter()
            .map(|group| {
                if group.len() == 1 {
                    self.format_type_hint(&group[0])
                } else {
                    format!(
                        "({})",
                        group
                            .iter()
                            .map(|t| self.format_type_hint(t))
                            .collect::<Vec<_>>()
                            .join("&")
                    )
                }
            })
            .collect::<Vec<_>>()
            .join("|")
    }

    fn format_type_hint(&self, type_hint: &TypeHint) -> String {
        match type_hint {
            TypeHint::Simple(name) => name.clone(),
            TypeHint::Class(name) => name.clone(),
            TypeHint::Nullable(inner) => format!("?{}", self.format_type_hint(inner)),
            TypeHint::Union(types) => types
                .iter()
                .map(|t| self.format_type_hint(t))
                .collect::<Vec<_>>()
                .join("|"),
            TypeHint::Intersection(types) => types
                .iter()
                .map(|t| self.format_type_hint(t))
                .collect::<Vec<_>>()
                .join("&"),
            TypeHint::DNF(intersections) => self.format_dnf_type(intersections),
            TypeHint::Void => "void".to_string(),
            TypeHint::Never => "never".to_string(),
            TypeHint::Static => "static".to_string(),
            TypeHint::SelfType => "self".to_string(),
            TypeHint::ParentType => "parent".to_string(),
        }
    }
}
```

### Step 4: Add Tests

**tests/types/dnf_basic.vhpt**
```
--TEST--
Basic DNF type: (A&B)|C
--FILE--
<?php
interface A {
    public function a();
}

interface B {
    public function b();
}

interface C {
    public function c();
}

class AB implements A, B {
    public function a() {}
    public function b() {}
}

class JustC implements C {
    public function c() {}
}

function process((A&B)|C $obj): void {
    echo "OK";
}

process(new AB());
echo "\n";
process(new JustC());
--EXPECT--
OK
OK
```

**tests/types/dnf_error.vhpt**
```
--TEST--
DNF type error when neither branch matches
--FILE--
<?php
interface A {
    public function a();
}

interface B {
    public function b();
}

interface C {
    public function c();
}

class OnlyA implements A {
    public function a() {}
}

function process((A&B)|C $obj): void {
    echo "OK";
}

process(new OnlyA());
--EXPECT_ERROR--
Expected type (A&B)|C
```

**tests/types/dnf_multiple_intersections.vhpt**
```
--TEST--
DNF with multiple intersections: (A&B)|(C&D)
--FILE--
<?php
interface A {}
interface B {}
interface C {}
interface D {}

class AB implements A, B {}
class CD implements C, D {}

function test((A&B)|(C&D) $obj): string {
    return get_class($obj);
}

echo test(new AB()) . "\n";
echo test(new CD());
--EXPECT--
AB
CD
```

**tests/types/dnf_with_class.vhpt**
```
--TEST--
DNF with class types: (Stringable&Countable)|ArrayAccess
--FILE--
<?php
interface Stringable {
    public function __toString();
}

interface Countable {
    public function count();
}

interface ArrayAccess {
    public function offsetGet($offset);
}

class Both implements Stringable, Countable {
    public function __toString() { return "Both"; }
    public function count() { return 1; }
}

class Array implements ArrayAccess {
    public function offsetGet($offset) { return null; }
}

function handle((Stringable&Countable)|ArrayAccess $obj): void {
    echo "valid";
}

handle(new Both());
echo "\n";
handle(new Array());
--EXPECT--
valid
valid
```

**tests/types/dnf_return_type.vhpt**
```
--TEST--
DNF type as return type
--FILE--
<?php
interface I1 {}
interface I2 {}
interface I3 {}

class A implements I1, I2 {}

function get(): (I1&I2)|I3 {
    return new A();
}

$result = get();
echo $result instanceof I1 ? "yes" : "no";
--EXPECT--
yes
```

**tests/types/dnf_three_branches.vhpt**
```
--TEST--
DNF with three union branches: (A&B)|(C&D)|E
--FILE--
<?php
interface A {}
interface B {}
interface C {}
interface D {}
interface E {}

class AB implements A, B {}
class CD implements C, D {}
class JustE implements E {}

function test((A&B)|(C&D)|E $obj): int {
    return 1;
}

echo test(new AB());
echo test(new CD());
echo test(new JustE());
--EXPECT--
111
```

**tests/types/dnf_single_type_in_union.vhpt**
```
--TEST--
DNF with single type in union: (A&B)|C (C is not an intersection)
--FILE--
<?php
interface A {}
interface B {}
interface C {}

class AB implements A, B {}
class JustC implements C {}

function test((A&B)|C $obj): string {
    return "ok";
}

echo test(new AB()) . "\n";
echo test(new JustC());
--EXPECT--
ok
ok
```

**tests/types/dnf_nullable.vhpt**
```
--TEST--
Nullable DNF type: ?((A&B)|C)
--FILE--
<?php
interface A {}
interface B {}
interface C {}

class AB implements A, B {}

function test(?((A&B)|C) $obj): string {
    return $obj === null ? "null" : "object";
}

echo test(new AB()) . "\n";
echo test(null);
--EXPECT--
object
null
```

**tests/types/dnf_parse_complex.vhpt**
```
--TEST--
Complex DNF parsing with parentheses
--FILE--
<?php
interface I1 {}
interface I2 {}
interface I3 {}
interface I4 {}

class A implements I1, I2 {}
class B implements I3, I4 {}

function complex((I1&I2)|(I3&I4) $x): void {
    echo "valid";
}

complex(new A());
echo "\n";
complex(new B());
--EXPECT--
valid
valid
```

**tests/types/dnf_inheritance.vhpt**
```
--TEST--
DNF with inheritance: subclass satisfies parent interface
--FILE--
<?php
interface Base {}
interface Extra {}

class Parent implements Base {}
class Child extends Parent implements Extra {}

function test((Base&Extra)|Child $obj): void {
    echo "ok";
}

// Child implements Extra and inherits Base
$child = new Child();
test($child);
--EXPECT--
ok
```

## PHP Compatibility Notes

| Feature | PHP Version |
|---------|-------------|
| DNF types | 8.2 |
| Syntax: `(A&B)\|C` | 8.2 |
| Multiple intersections: `(A&B)\|(C&D)` | 8.2 |
| Nullable DNF: `?((A&B)\|C)` | 8.2 |

## Key Considerations

1. **DNF Form**: Must be in Disjunctive Normal Form: `(A&B)|C` is valid, but `A&(B|C)` is not
2. **Parentheses required**: Intersections in unions must use parentheses: `(A&B)|C`
3. **Type mixing**: Can mix single types with intersections: `(A&B)|C|D`
4. **Validation**: Must check at runtime that object implements all interfaces in an intersection
5. **Error messages**: Should clearly show the expected DNF type
6. **Inheritance**: Subclass that implements required interfaces should match
7. **Nullable**: DNF can be wrapped in nullable: `?((A&B)|C)`

## DNF Rules

1. **Format**: `(T1&T2)|T3|(T4&T5)` where each group can be:
   - Single type: `T3`
   - Intersection: `(T1&T2)`
2. **Restrictions**:
   - Cannot mix `&` and `|` without parentheses
   - Intersections must be in parentheses when part of union
   - Only class/interface types in intersections (not int, string, etc.)
3. **Matching**:
   - Value must match at least one "branch" of the union
   - To match an intersection branch, must match ALL types in that intersection

## Error Messages

- `Expected type (A&B)|C, got <type>`
- `Cannot use scalar types in intersection types`
- `DNF types must be in form (A&B)|C`

## Implementation Order

1. Add `DNF` variant to TypeHint enum
2. Update parser to handle parenthesized intersections
3. Parse union of intersections as DNF
4. Implement DNF validation logic
5. Add intersection matching helper
6. Format DNF for error messages
7. Add comprehensive tests

## Edge Cases

1. **Single intersection**: `(A&B)` is technically DNF with one branch
2. **All single types**: `A|B|C` is union, not DNF
3. **Nested parentheses**: Not allowed, only one level
4. **Scalar types**: Cannot be in intersections (only class/interface)
5. **Three+ interfaces**: `(A&B&C)|D` is valid DNF
6. **Redundant parentheses**: `(A)|(B)` should work but is unusual
7. **null in DNF**: `(A&B)|null` should work (union with null)

## Reference Implementations

- Union types: Already implemented (PHP 8.0)
- Intersection types: Already parsed (PHP 8.1)
- Type validation: `src/interpreter/types.rs`
- instanceof checks: Already implemented for class types
