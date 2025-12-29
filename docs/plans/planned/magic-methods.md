# Plan: Magic Methods

## Overview

Magic methods are special methods with names starting with double underscore (`__`) that PHP calls automatically in certain situations. They enable operator overloading, property overloading, and other dynamic behaviors.

**PHP Example:**
```php
<?php
class MagicBox {
    private array $data = [];
    
    // Called when accessing undefined properties
    public function __get(string $name): mixed {
        return $this->data[$name] ?? null;
    }
    
    // Called when setting undefined properties
    public function __set(string $name, mixed $value): void {
        $this->data[$name] = $value;
    }
    
    // Called when isset() on undefined property
    public function __isset(string $name): bool {
        return isset($this->data[$name]);
    }
    
    // Called when unset() on undefined property
    public function __unset(string $name): void {
        unset($this->data[$name]);
    }
    
    // Called when object used as string
    public function __toString(): string {
        return json_encode($this->data);
    }
    
    // Called when object used as function
    public function __invoke(mixed $arg): mixed {
        return "Invoked with: $arg";
    }
    
    // Called for undefined method calls
    public function __call(string $name, array $args): mixed {
        return "Called $name with " . count($args) . " args";
    }
    
    // Called for undefined static method calls  
    public static function __callStatic(string $name, array $args): mixed {
        return "Static called $name";
    }
}

$box = new MagicBox();
$box->key = "value";    // __set
echo $box->key;         // __get
echo $box;              // __toString
echo $box("arg");       // __invoke
echo $box->missing();   // __call
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/interpreter/objects/mod.rs` | Detect and call magic methods |
| `src/interpreter/expr_eval/mod.rs` | Handle __toString, __invoke |
| `src/interpreter/builtins/mod.rs` | Hook __isset, __unset |
| `tests/classes/magic_*.vhpt` | Test files |

Note: No token or AST changes needed - magic methods are regular methods with special names.

## Magic Methods to Implement

| Method | When Called | Priority |
|--------|-------------|----------|
| `__construct` | ✅ Already implemented | - |
| `__toString` | String conversion | High |
| `__invoke` | Object called as function | High |
| `__get` | Read undefined property | High |
| `__set` | Write undefined property | High |
| `__isset` | isset() on undefined prop | Medium |
| `__unset` | unset() on undefined prop | Medium |
| `__call` | Call undefined method | Medium |
| `__callStatic` | Call undefined static method | Medium |
| `__clone` | ✅ Already implemented | - |
| `__debugInfo` | var_dump() output | Low |
| `__serialize` | serialize() (PHP 7.4+) | Low |
| `__unserialize` | unserialize() (PHP 7.4+) | Low |
| `__sleep` | serialize() (legacy) | Low |
| `__wakeup` | unserialize() (legacy) | Low |
| `__destruct` | Object destroyed | Low |

## Implementation Steps

### Step 1: Helper to Check for Magic Method

Add utility to check if a class has a magic method:

```rust
impl Class {
    /// Find a magic method by name (case-insensitive)
    pub fn get_magic_method(&self, name: &str) -> Option<&Method> {
        self.methods.iter().find(|m| m.name.eq_ignore_ascii_case(name))
    }
    
    pub fn has_magic_method(&self, name: &str) -> bool {
        self.get_magic_method(name).is_some()
    }
}
```

### Step 2: Implement __toString (`src/interpreter/expr_eval/mod.rs`)

Called when object is used in string context:

```rust
/// Convert value to string, calling __toString if available
fn value_to_string(&mut self, value: &Value) -> Result<String, String> {
    match value {
        Value::String(s) => Ok(s.clone()),
        Value::Integer(i) => Ok(i.to_string()),
        Value::Float(f) => Ok(f.to_string()),
        Value::Bool(true) => Ok("1".to_string()),
        Value::Bool(false) => Ok("".to_string()),
        Value::Null => Ok("".to_string()),
        Value::Array(_) => Err("Array to string conversion".to_string()),
        Value::Object(obj) => {
            // Check for __toString magic method
            let class = self.get_class(&obj.class_name)?;
            if let Some(method) = class.get_magic_method("__toString") {
                let result = self.call_method(obj.clone(), method, vec![])?;
                if let Value::String(s) = result {
                    Ok(s)
                } else {
                    Err(format!(
                        "{}::__toString() must return a string value",
                        obj.class_name
                    ))
                }
            } else {
                Err(format!(
                    "Object of class {} could not be converted to string",
                    obj.class_name
                ))
            }
        }
    }
}
```

Use in echo, concatenation, and other string contexts:

```rust
// In echo statement execution
Stmt::Echo(exprs) => {
    for expr in exprs {
        let val = self.evaluate(expr)?;
        let str_val = self.value_to_string(&val)?;
        print!("{}", str_val);
    }
}

// In concatenation
Operator::Concat => {
    let left_str = self.value_to_string(&left)?;
    let right_str = self.value_to_string(&right)?;
    Value::String(format!("{}{}", left_str, right_str))
}
```

### Step 3: Implement __invoke (`src/interpreter/expr_eval/mod.rs`)

Called when object is used as a function:

```rust
fn call_expression(&mut self, callee: &Expr, args: &[Expr]) -> Result<Value, String> {
    let callee_value = self.evaluate(callee)?;
    
    match callee_value {
        Value::Object(obj) => {
            // Check for __invoke magic method
            let class = self.get_class(&obj.class_name)?;
            if let Some(method) = class.get_magic_method("__invoke") {
                let arg_values = args.iter()
                    .map(|a| self.evaluate(a))
                    .collect::<Result<Vec<_>, _>>()?;
                self.call_method(obj, method, arg_values)
            } else {
                Err(format!(
                    "Call to undefined method {}::__invoke()",
                    obj.class_name
                ))
            }
        }
        Value::Closure(closure) => {
            // ... existing closure handling ...
        }
        Value::String(func_name) => {
            self.call_function(&func_name, args)
        }
        _ => Err("Value is not callable".to_string()),
    }
}
```

### Step 4: Implement __get and __set (`src/interpreter/objects/mod.rs`)

Called when accessing undefined properties:

```rust
/// Get property value, using __get if property doesn't exist
fn get_property(&mut self, obj: &Object, prop_name: &str) -> Result<Value, String> {
    // First check if property exists
    if let Some(value) = obj.properties.get(prop_name) {
        return Ok(value.clone());
    }
    
    // Check for __get magic method
    let class = self.get_class(&obj.class_name)?;
    if let Some(method) = class.get_magic_method("__get") {
        return self.call_method(
            obj.clone(),
            method,
            vec![Value::String(prop_name.to_string())],
        );
    }
    
    // Return null for undefined property (PHP behavior)
    Ok(Value::Null)
}

/// Set property value, using __set if property doesn't exist
fn set_property(
    &mut self,
    obj: &mut Object,
    prop_name: &str,
    value: Value,
) -> Result<(), String> {
    // Check if this is a declared property
    if obj.properties.contains_key(prop_name) || self.is_declared_property(&obj.class_name, prop_name) {
        obj.properties.insert(prop_name.to_string(), value);
        return Ok(());
    }
    
    // Check for __set magic method
    let class = self.get_class(&obj.class_name)?;
    if let Some(method) = class.get_magic_method("__set") {
        self.call_method(
            obj.clone(),
            method,
            vec![
                Value::String(prop_name.to_string()),
                value,
            ],
        )?;
        return Ok(());
    }
    
    // Allow dynamic property (with deprecation warning in future)
    obj.properties.insert(prop_name.to_string(), value);
    Ok(())
}
```

### Step 5: Implement __isset and __unset

Called by `isset()` and `unset()` functions:

```rust
/// Check if property is set, using __isset if property doesn't exist
fn property_isset(&mut self, obj: &Object, prop_name: &str) -> Result<bool, String> {
    // First check if property exists
    if let Some(value) = obj.properties.get(prop_name) {
        return Ok(!matches!(value, Value::Null));
    }
    
    // Check for __isset magic method
    let class = self.get_class(&obj.class_name)?;
    if let Some(method) = class.get_magic_method("__isset") {
        let result = self.call_method(
            obj.clone(),
            method,
            vec![Value::String(prop_name.to_string())],
        )?;
        return Ok(result.to_bool());
    }
    
    Ok(false)
}

/// Unset property, using __unset if it's a virtual property
fn property_unset(&mut self, obj: &mut Object, prop_name: &str) -> Result<(), String> {
    // First check if property exists
    if obj.properties.contains_key(prop_name) {
        obj.properties.remove(prop_name);
        return Ok(());
    }
    
    // Check for __unset magic method
    let class = self.get_class(&obj.class_name)?;
    if let Some(method) = class.get_magic_method("__unset") {
        self.call_method(
            obj.clone(),
            method,
            vec![Value::String(prop_name.to_string())],
        )?;
    }
    
    Ok(())
}
```

Update `isset()` and `unset()` built-in functions:

```rust
// In builtins
fn builtin_isset(&mut self, args: &[Expr]) -> Result<Value, String> {
    for arg in args {
        match arg {
            Expr::PropertyAccess { object, property } => {
                let obj_val = self.evaluate(object)?;
                if let Value::Object(obj) = obj_val {
                    if !self.property_isset(&obj, property)? {
                        return Ok(Value::Bool(false));
                    }
                }
            }
            Expr::Variable(name) => {
                if !self.variable_exists(name) {
                    return Ok(Value::Bool(false));
                }
            }
            // ... other cases ...
        }
    }
    Ok(Value::Bool(true))
}
```

### Step 6: Implement __call and __callStatic

Called for undefined method calls:

```rust
/// Call instance method, using __call for undefined methods
fn call_method(
    &mut self,
    obj: Object,
    method_name: &str,
    args: Vec<Value>,
) -> Result<Value, String> {
    let class = self.get_class(&obj.class_name)?;
    
    // First try to find the method
    if let Some(method) = class.get_method(method_name) {
        return self.execute_method(obj, method, args);
    }
    
    // Check for __call magic method
    if let Some(call_method) = class.get_magic_method("__call") {
        let args_array = Value::Array(
            args.into_iter()
                .enumerate()
                .map(|(i, v)| (Value::Integer(i as i64), v))
                .collect()
        );
        return self.execute_method(
            obj,
            call_method,
            vec![
                Value::String(method_name.to_string()),
                args_array,
            ],
        );
    }
    
    Err(format!(
        "Call to undefined method {}::{}()",
        obj.class_name,
        method_name
    ))
}

/// Call static method, using __callStatic for undefined methods
fn call_static_method(
    &mut self,
    class_name: &str,
    method_name: &str,
    args: Vec<Value>,
) -> Result<Value, String> {
    let class = self.get_class(class_name)?;
    
    // First try to find the static method
    if let Some(method) = class.get_static_method(method_name) {
        return self.execute_static_method(class_name, method, args);
    }
    
    // Check for __callStatic magic method
    if let Some(call_static) = class.get_magic_method("__callStatic") {
        let args_array = Value::Array(
            args.into_iter()
                .enumerate()
                .map(|(i, v)| (Value::Integer(i as i64), v))
                .collect()
        );
        return self.execute_static_method(
            class_name,
            call_static,
            vec![
                Value::String(method_name.to_string()),
                args_array,
            ],
        );
    }
    
    Err(format!(
        "Call to undefined method {}::{}()",
        class_name,
        method_name
    ))
}
```

### Step 7: Implement __debugInfo

Used by `var_dump()`:

```rust
fn var_dump_object(&mut self, obj: &Object, indent: usize) -> Result<String, String> {
    let class = self.get_class(&obj.class_name)?;
    
    // Check for __debugInfo
    if let Some(method) = class.get_magic_method("__debugInfo") {
        let info = self.call_method(obj.clone(), method, vec![])?;
        if let Value::Array(arr) = info {
            // Format the debug info array
            return self.format_var_dump(&info, indent);
        }
    }
    
    // Default object dump
    let mut output = format!("object({})#{} ({} properties) {{\n",
        obj.class_name,
        obj.id,
        obj.properties.len()
    );
    
    for (name, value) in &obj.properties {
        let value_dump = self.format_var_dump(value, indent + 2)?;
        output.push_str(&format!(
            "{}[{:?}]=>\n{}{}",
            " ".repeat(indent + 2),
            name,
            " ".repeat(indent + 2),
            value_dump
        ));
    }
    
    output.push_str(&format!("{}}}\n", " ".repeat(indent)));
    Ok(output)
}
```

### Step 8: Add Tests

**tests/classes/magic_tostring.vhpt**
```
--TEST--
__toString magic method
--FILE--
<?php
class Name {
    private $first;
    private $last;
    
    public function __construct($first, $last) {
        $this->first = $first;
        $this->last = $last;
    }
    
    public function __toString(): string {
        return $this->first . " " . $this->last;
    }
}

$name = new Name("John", "Doe");
echo $name;
--EXPECT--
John Doe
```

**tests/classes/magic_invoke.vhpt**
```
--TEST--
__invoke magic method
--FILE--
<?php
class Adder {
    private $base;
    
    public function __construct($base) {
        $this->base = $base;
    }
    
    public function __invoke($n) {
        return $this->base + $n;
    }
}

$add5 = new Adder(5);
echo $add5(10);
--EXPECT--
15
```

**tests/classes/magic_get_set.vhpt**
```
--TEST--
__get and __set magic methods
--FILE--
<?php
class MagicProps {
    private $data = [];
    
    public function __get($name) {
        return $this->data[$name] ?? "undefined";
    }
    
    public function __set($name, $value) {
        $this->data[$name] = $value;
    }
}

$obj = new MagicProps();
$obj->foo = "bar";
echo $obj->foo . "\n";
echo $obj->missing;
--EXPECT--
bar
undefined
```

**tests/classes/magic_isset_unset.vhpt**
```
--TEST--
__isset and __unset magic methods
--FILE--
<?php
class Container {
    private $items = [];
    
    public function __set($name, $value) {
        $this->items[$name] = $value;
    }
    
    public function __get($name) {
        return $this->items[$name] ?? null;
    }
    
    public function __isset($name) {
        return isset($this->items[$name]);
    }
    
    public function __unset($name) {
        unset($this->items[$name]);
    }
}

$c = new Container();
$c->key = "value";
echo isset($c->key) ? "yes" : "no";
echo "\n";
unset($c->key);
echo isset($c->key) ? "yes" : "no";
--EXPECT--
yes
no
```

**tests/classes/magic_call.vhpt**
```
--TEST--
__call magic method
--FILE--
<?php
class Wrapper {
    public function __call($method, $args) {
        return "Called $method with " . count($args) . " args";
    }
}

$w = new Wrapper();
echo $w->unknownMethod(1, 2, 3);
--EXPECT--
Called unknownMethod with 3 args
```

**tests/classes/magic_callstatic.vhpt**
```
--TEST--
__callStatic magic method
--FILE--
<?php
class Factory {
    public static function __callStatic($method, $args) {
        return "Static: $method";
    }
}

echo Factory::anything();
--EXPECT--
Static: anything
```

**tests/classes/magic_tostring_concat.vhpt**
```
--TEST--
__toString used in concatenation
--FILE--
<?php
class Version {
    public function __toString(): string {
        return "1.0.0";
    }
}

$v = new Version();
echo "Version: " . $v;
--EXPECT--
Version: 1.0.0
```

**tests/classes/magic_tostring_error.vhpt**
```
--TEST--
__toString must return string
--FILE--
<?php
class Bad {
    public function __toString(): string {
        return 42;
    }
}

echo new Bad();
--EXPECT_ERROR--
__toString() must return a string value
```

**tests/classes/magic_no_tostring.vhpt**
```
--TEST--
Object without __toString cannot be converted to string
--FILE--
<?php
class NoString {}
echo new NoString();
--EXPECT_ERROR--
could not be converted to string
```

## Magic Method Signatures

| Method | Parameters | Return |
|--------|------------|--------|
| `__toString()` | none | `string` |
| `__invoke(...)` | any | `mixed` |
| `__get(string $name)` | property name | `mixed` |
| `__set(string $name, mixed $value)` | name, value | `void` |
| `__isset(string $name)` | property name | `bool` |
| `__unset(string $name)` | property name | `void` |
| `__call(string $name, array $args)` | method, args | `mixed` |
| `__callStatic(string $name, array $args)` | method, args | `mixed` |
| `__debugInfo()` | none | `array` |

## PHP Compatibility Notes

| Feature | PHP Version |
|---------|-------------|
| Basic magic methods | 5.0 |
| `__invoke` | 5.3 |
| `__callStatic` | 5.3 |
| `__debugInfo` | 5.6 |
| Return type on `__toString` | 7.4 |
| Strict return types | 8.0 |

## Implementation Order

1. **__toString** - Most commonly needed, affects echo/print
2. **__invoke** - Enables callable objects
3. **__get/__set** - Property overloading (very common)
4. **__isset/__unset** - Complete property overloading
5. **__call/__callStatic** - Method overloading
6. **__debugInfo** - Nice-to-have for debugging

## Key Considerations

1. Magic methods are called implicitly, so they should be performant
2. Recursion guard may be needed (calling __get from within __get)
3. Error messages should be clear about what triggered the magic method
4. Visibility rules apply - magic methods are typically public
5. __construct and __clone are already implemented per AGENTS.md
