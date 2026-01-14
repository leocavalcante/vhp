# Serialization Support

## Status: Planned

## Overview

Implement serialize(), unserialize(), and __serialize()/__unserialize() magic methods for object serialization and deserialization.

## Current Status

Serialization functions are not implemented.

## Background

Serialization is essential for:
- Caching objects
- Session storage
- Data persistence
- Object transport over network
- Deep cloning

PHP 7.4 introduced the modern __serialize()/__unserialize() magic methods, replacing __sleep()/__wakeup().

## Requirements

### Procedural Functions

1. **serialize**
   ```php
   serialize($value): string
   ```
   - Generate storable representation of a value
   - Support: null, bool, int, float, string, array, object
   - Generate PHP-compatible serialized format

2. **unserialize**
   ```php
   unserialize($string, $options = []): mixed|false
   ```
   - Create PHP value from stored representation
   - Reconstruct objects
   - Handle class autoloading
   - Support options:
     - `allowed_classes`: array of allowed class names or true for all classes
     - `max_depth`: maximum depth of structures

### Serialized Format

PHP's serialized format is a text-based format:

```php
// Scalars
serialize(42);                    // i:42;
serialize(3.14);                  // d:3.14;
serialize(true);                   // b:1;
serialize(null);                   // N;
serialize("hello");                // s:5:"hello";

// Arrays
serialize([1, 2, 3]);            // a:3:{i:0;i:1;i:1;i:2;i:2;i:3;}
serialize(["a" => 1, "b" => 2]); // a:2:{s:1:"a";i:1;s:1:"b";i:2;}

// Objects
serialize($obj);                   // O:8:"ClassName":2:{s:4:"prop";s:5:"value";}

// Custom serialization (PHP 7.4+)
serialize($obj);                   // C:8:"ClassName":23:{...serialized data...}
```

### Magic Methods

1. **__sleep()** (Legacy)
   ```php
   public function __sleep(): array
   ```
   - Return array of property names to serialize
   - Called before serialization
   - Can exclude properties from serialization

2. **__wakeup()** (Legacy)
   ```php
   public function __wakeup(): void
   ```
   - Called during unserialization
   - Reinitialize resources, database connections, etc.

3. **__serialize()** (PHP 7.4+)
   ```php
   public function __serialize(): array
   ```
   - Return array of data to serialize
   - Replaces __sleep()
   - Called during serialization

4. **__unserialize()** (PHP 7.4+)
   ```php
   public function __unserialize(array $data): void
   ```
   - Called during unserialization with serialized data
   - Reconstruct object state
   - Replaces __wakeup()

### Serializable Interface

```php
interface Serializable {
    public function serialize(): string;
    public function unserialize($serialized): void;
}
```

This is the old interface (PHP 5.1+), superseded by __serialize()/__unserialize() in PHP 7.4.

### Object Serialization Behavior

1. **Normal Objects**
   - Serialize all properties
   - Include private/protected properties with class prefix
   - Preserve property values

2. **Objects with __serialize()**
   - Call __serialize() and serialize returned array
   - Use "C:" format instead of "O:" format
   - More efficient than legacy methods

3. **Objects with __sleep()** (Legacy)
   - Get list of properties to serialize
   - Serialize only those properties
   - Use "O:" format

4. **Objects implementing Serializable**
   - Call serialize() method
   - Serialize returned string
   - Use "C:" format with custom data

### Private/Protected Property Serialization

Properties are prefixed with class name in serialization:

```php
class Base {
    private $private = 1;
    protected $protected = 2;
}

class Child extends Base {
    private $private = 3;
}

$child = new Child();
serialize($child);
// Private properties are prefixed with class name
// Protected properties are prefixed with *
```

Example:
```php
class Test {
    private $private = "priv";
    protected $protected = "prot";
}

serialize(new Test());
// O:4:"Test":2:{s:13:"\0Test\0private";s:4:"priv";s:12:"\0*\0protected";s:4:"prot";}
```

### Circular References

PHP handles circular references using reference IDs:

```php
$a = [1, 2];
$a[] = &$a;  // Circular reference
serialize($a);
// a:3:{i:0;i:1;i:1;i:2;i:2;R:2;}
```

### Unserialization Security

PHP 7.0+ introduced security features:
- `allowed_classes` option to restrict which classes can be unserialized
- Default: `allowed_classes => false` (no classes allowed, only scalar/array)
- `allowed_classes => true`: allow all classes
- `allowed_classes => ['ClassName']`: allow specific classes

```php
// Secure unserialization
$data = unserialize($string, ['allowed_classes' => ['AllowedClass']]);
```

## Implementation Plan

### Phase 1: Scalar Serialization

**File:** `runtime/serialization.rs` (new)

**Tasks:**
- [ ] Create serializer module
- [ ] Implement null serialization
- [ ] Implement bool serialization
- [ ] Implement int serialization
- [ ] Implement float serialization
- [ ] Implement string serialization (with escape sequences)
- [ ] Add tests

### Phase 2: Array Serialization

**File:** `runtime/serialization.rs` (extend)

**Tasks:**
- [ ] Implement array serialization
- [ ] Handle numeric keys
- [ ] Handle string keys
- [ ] Handle nested arrays
- [ ] Add tests

### Phase 3: Object Serialization (Basic)

**File:** `runtime/serialization.rs` (extend)

**Tasks:**
- [ ] Implement object serialization
- [ ] Serialize public properties
- [ ] Serialize protected properties with prefix
- [ ] Serialize private properties with class prefix
- [ ] Handle inheritance
- [ ] Add tests

### Phase 4: Magic Methods (Legacy)

**File:** `runtime/serialization.rs` (extend)

**Tasks:**
- [ ] Check for __sleep() method
- [ ] Call __sleep() during serialization
- [ ] Check for __wakeup() method
- [ ] Call __wakeup() during unserialization
- [ ] Add tests

### Phase 5: Magic Methods (Modern PHP 7.4+)

**File:** `runtime/serialization.rs` (extend)

**Tasks:**
- [ ] Check for __serialize() method
- [ ] Call __serialize() during serialization
- [ ] Use "C:" format for __serialize() objects
- [ ] Check for __unserialize() method
- [ ] Call __unserialize() during unserialization
- [ ] Add tests

### Phase 6: Serializable Interface

**File:** `runtime/serialization.rs` (extend)

**Tasks:**
- [ ] Check if class implements Serializable
- [ ] Call serialize() method
- [ ] Call unserialize() method
- [ ] Add tests

### Phase 7: Scalar Unserialization

**File:** `runtime/serialization.rs` (extend)

**Tasks:**
- [ ] Create parser for serialized format
- [ ] Parse null: "N;"
- [ ] Parse bool: "b:0;" or "b:1;"
- [ ] Parse int: "i:123;"
- [ ] Parse float: "d:3.14;"
- [ ] Parse string: "s:5:"hello";"
- [ ] Add tests

### Phase 8: Array Unserialization

**File:** `runtime/serialization.rs` (extend)

**Tasks:**
- [ ] Parse array: "a:3:{...}"
- [ ] Handle numeric keys
- [ ] Handle string keys
- [ ] Handle nested arrays
- [ ] Add tests

### Phase 9: Object Unserialization (Basic)

**File:** `runtime/serialization.rs` (extend)

**Tasks:**
- [ ] Parse object: "O:8:"ClassName":2:{...}"
- [ ] Resolve class name (with autoloading)
- [ ] Create object instance
- [ ] Set public properties
- [ ] Set protected properties
- [ ] Set private properties
- [ ] Handle inheritance
- [ ] Add tests

### Phase 10: Reference Handling

**File:** `runtime/serialization.rs` (extend)

**Tasks:**
- [ ] Track references during serialization
- [ ] Generate reference IDs: "R:2;"
- [ ] Resolve references during unserialization
- [ ] Handle circular references
- [ ] Add tests

### Phase 11: Security Features

**File:** `runtime/serialization.rs` (extend)

**Tasks:**
- [ ] Implement allowed_classes option
- [ ] Validate classes during unserialization
- [ ] Handle false (no classes)
- [ ] Handle true (all classes)
- [ ] Handle array of allowed class names
- [ ] Add security tests

### Phase 12: Error Handling

**File:** `runtime/serialization.rs` (extend)

**Tasks:**
- [ ] Handle parse errors
- [ ] Handle invalid data types
- [ ] Handle missing classes
- [ ] Handle class autoloading failures
- [ ] Provide clear error messages
- [ ] Add error tests

### Phase 13: Tests

**File:** `tests/serialization/` (new directory)

Test coverage:
- Scalar serialization/unserialization
- Array serialization/unserialization
- Object serialization/unserialization
- Private/protected properties
- Inheritance
- __sleep()/__wakeup() (legacy)
- __serialize()/__unserialize() (PHP 7.4+)
- Serializable interface
- Circular references
- Security features (allowed_classes)
- Error conditions
- Edge cases

**Example tests:**

```
--TEST--
Scalar serialization
--FILE--
<?php
echo serialize(null) . "\n";
echo serialize(true) . "\n";
echo serialize(42) . "\n";
echo serialize(3.14) . "\n";
echo serialize("hello") . "\n";
--EXPECT--
N;
b:1;
i:42;
d:3.14;
s:5:"hello";
```

```
--TEST--
Array serialization
--FILE--
<?php
echo serialize([1, 2, 3]) . "\n";
echo serialize(["a" => 1, "b" => 2]) . "\n";
--EXPECT--
a:3:{i:0;i:1;i:1;i:2;i:2;i:3;}
a:2:{s:1:"a";i:1;s:1:"b";i:2;}
```

```
--TEST--
Object with __serialize()
--FILE--
<?php
class Test {
    public $data = "value";

    public function __serialize(): array {
        return ['custom' => $this->data];
    }

    public function __unserialize(array $data): void {
        $this->data = $data['custom'];
    }
}

$obj = new Test();
$serialized = serialize($obj);
$unserialized = unserialize($serialized);
echo $unserialized->data;
--EXPECT--
value
```

```
--TEST--
Security: allowed_classes
--FILE--
<?php
class Allowed {}
class Forbidden {}

$obj = new Allowed();
$serialized = serialize($obj);

// Should work - class is allowed
$unserialized = unserialize($serialized, ['allowed_classes' => ['Allowed']]);
echo get_class($unserialized) . "\n";

// Should fail - class is not allowed
$result = unserialize($serialized, ['allowed_classes' => ['OtherClass']]);
var_dump($result);
--EXPECT--
Allowed
bool(false)
```

## Implementation Details

### Serialization Format

```rust
enum SerializedData {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<(Value, Value)>),
    Object { class: String, properties: Vec<(String, Value)> },
    CustomObject { class: String, data: String },
}
```

### Serializer

```rust
pub struct Serializer {
    reference_map: HashMap<usize, u32>,  // object pointer -> reference ID
    next_reference_id: u32,
}

impl Serializer {
    pub fn serialize(&mut self, value: &Value) -> String {
        match value {
            Value::Null => "N;".to_string(),
            Value::Bool(b) => format!("b:{};", if *b { 1 } else { 0 }),
            Value::Int(n) => format!("i:{};", n),
            Value::Float(f) => format!("d:{:?};", f),
            Value::String(s) => format!("s:{}:\"{}\";", s.len(), escape_string(s)),
            Value::Array(arr) => self.serialize_array(arr),
            Value::Object(obj) => self.serialize_object(obj),
        }
    }

    fn serialize_object(&mut self, obj: &Instance) -> String {
        // Check for __serialize()
        if let Some(serialize_method) = obj.get_method("__serialize") {
            // Call __serialize() and serialize returned array
            let data = self.call_method(obj, serialize_method, [])?;
            return format!("C:{}:{}:{}",
                obj.get_class_name().len(),
                obj.get_class_name(),
                self.serialize(&data)
            );
        }

        // Check for __sleep()
        if let Some(sleep_method) = obj.get_method("__sleep") {
            // Get list of properties to serialize
            let props = self.call_method(obj, sleep_method, [])?;
            // Serialize only those properties
        }

        // Default serialization
        self.serialize_object_default(obj)
    }
}
```

### Unserializer

```rust
pub struct Unserializer {
    input: String,
    pos: usize,
    references: Vec<Option<Value>>,  // reference ID -> value
    options: UnserializeOptions,
}

pub struct UnserializeOptions {
    allowed_classes: AllowedClasses,
    max_depth: usize,
}

pub enum AllowedClasses {
    None,
    All,
    Specific(HashSet<String>),
}

impl Unserializer {
    pub fn unserialize(&mut self) -> Result<Value, String> {
        match self.peek_char() {
            'N' => self.unserialize_null(),
            'b' => self.unserialize_bool(),
            'i' => self.unserialize_int(),
            'd' => self.unserialize_float(),
            's' => self.unserialize_string(),
            'a' => self.unserialize_array(),
            'O' => self.unserialize_object(),
            'C' => self.unserialize_custom_object(),
            'R' => self.unserialize_reference(),
            c => Err(format!("Unknown type: {}", c)),
        }
    }

    fn unserialize_object(&mut self) -> Result<Value, String> {
        self.expect_char('O');
        self.expect_char(':');

        let class_name = self.parse_length_string()?;
        let property_count = self.parse_length()?;

        // Check if class is allowed
        let class = self.resolve_class(&class_name)?;

        // Create object without calling __construct
        let obj = Instance::new_without_construct(class.clone())?;

        self.expect_char('{');

        for _ in 0..property_count {
            let (prop_name, prop_value) = self.unserialize_property()?;
            obj.set_property(&prop_name, prop_value)?;
        }

        self.expect_char('}');

        // Call __wakeup() if exists
        if let Some(wakeup_method) = obj.get_method("__wakeup") {
            self.call_method(&obj, wakeup_method, [])?;
        }

        Ok(obj.into())
    }
}
```

### Property Name Encoding

```rust
fn encode_property_name(class_name: &str, property_name: &str, visibility: Visibility) -> String {
    match visibility {
        Visibility::Public => property_name.to_string(),
        Visibility::Protected => format!("\0*\0{}", property_name),
        Visibility::Private => format!("\0{}\0{}", class_name, property_name),
    }
}
```

## Dependencies

- Existing class system
- Existing reflection system
- Existing object model

## Testing Strategy

1. **Unit Tests**: Each serialization/unserialization function
2. **Integration Tests**: Complete object serialization cycle
3. **Security Tests**: allowed_classes restrictions
4. **Reference Tests**: Circular references
5. **Compatibility Tests**: Match PHP 8.x format

## Success Criteria

- serialize() works for all value types
- unserialize() works for all value types
- __serialize()/__unserialize() implemented
- Legacy __sleep()/__wakeup() implemented
- Serializable interface supported
- Security features implemented
- All tests pass

## Performance Considerations

- Efficient string building
- Fast property access
- Optimize reference tracking
- Minimize memory allocations

## Security Considerations

- Implement allowed_classes by default
- Validate class names
- Prevent object injection attacks
- Limit recursion depth
- Safe autoloading

## Open Questions

- Should we implement Serializable interface or only __serialize()/__unserialize()?
- How to handle class autoloading during unserialization?

## References

- PHP serialize documentation: https://www.php.net/manual/en/function.serialize.php
- PHP unserialize documentation: https://www.php.net/manual/en/function.unserialize.php
- PHP 7.4 Serialization RFC: https://wiki.php.net/rfc/serialize_exceptions

## Related Plans

- Object cloning (already implemented)
- Reflection system (already implemented)
