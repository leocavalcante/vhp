# Plan: Attribute Reflection API (PHP 8.0)

## Overview

The Attribute Reflection API allows PHP code to retrieve and inspect attributes that were attached to classes, methods, properties, functions, and parameters at runtime. This is the second part of the attributes feature - the first part (syntax parsing and AST storage) is already implemented in VHP.

PHP 8.0 introduced reflection methods to retrieve attributes via `ReflectionClass`, `ReflectionMethod`, `ReflectionProperty`, `ReflectionFunction`, and `ReflectionParameter`. Since VHP doesn't have a full Reflection API yet, this plan will implement simplified built-in functions that provide attribute retrieval capabilities.

**PHP Example:**

```php
// Before (without reflection API)
#[Route("/api/users")]
class UserController {
    #[ValidateRequest]
    public function create() {}
}
// Attributes are stored but cannot be accessed at runtime

// After (with reflection API)
#[Route("/api/users")]
class UserController {
    #[ValidateRequest]
    public function create() {}
}

// Get attributes at runtime
$class_attrs = get_class_attributes(UserController::class);
var_dump($class_attrs);
// Array with attribute metadata

$method_attrs = get_method_attributes(UserController::class, 'create');
var_dump($method_attrs);
// Array with method attribute metadata
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/interpreter/builtins/mod.rs` | Export new reflection module |
| `src/interpreter/builtins/reflection.rs` | NEW - Implement reflection functions |
| `src/interpreter/mod.rs` | Register reflection built-in functions |
| `src/interpreter/value.rs` | No changes needed (can use Array to represent attributes) |
| `tests/attributes/reflection_*.vhpt` | NEW - Test files for reflection API |
| `AGENTS.md` | Mark reflection API as complete |
| `README.md` | Update features list |
| `docs/features.md` | Add reflection API documentation |

## Implementation Steps

### Step 1: Create Reflection Built-ins Module (`src/interpreter/builtins/reflection.rs`)

**Location:** Create new file

**File contents:**

```rust
//! Reflection built-in functions for attributes

use crate::interpreter::value::{ArrayKey, Value};
use crate::ast::{Attribute, AttributeArgument, Expr};

/// Convert an Attribute AST node to a runtime Value (associative array)
/// Format: ["name" => "AttributeName", "arguments" => [...]]
pub fn attribute_to_value(attr: &Attribute) -> Value {
    let mut result = Vec::new();

    // Add attribute name
    result.push((
        ArrayKey::String("name".to_string()),
        Value::String(attr.name.clone()),
    ));

    // Add arguments as an array
    let mut args_array = Vec::new();
    for arg in &attr.arguments {
        args_array.push((
            ArrayKey::Integer(args_array.len() as i64),
            argument_to_value(arg),
        ));
    }

    result.push((
        ArrayKey::String("arguments".to_string()),
        Value::Array(args_array),
    ));

    Value::Array(result)
}

/// Convert an AttributeArgument to a runtime Value (associative array)
/// Format: ["name" => "param_name" or null, "value" => <evaluated_value>]
fn argument_to_value(arg: &AttributeArgument) -> Value {
    let mut result = Vec::new();

    // Add argument name (null for positional arguments)
    result.push((
        ArrayKey::String("name".to_string()),
        match &arg.name {
            Some(name) => Value::String(name.clone()),
            None => Value::Null,
        },
    ));

    // Add argument value (simplified - convert literal expressions to values)
    result.push((
        ArrayKey::String("value".to_string()),
        expr_to_simple_value(&arg.value),
    ));

    Value::Array(result)
}

/// Convert a simple expression (literals) to a Value
/// For more complex expressions, we'd need full expression evaluation
fn expr_to_simple_value(expr: &Expr) -> Value {
    match expr {
        Expr::Null => Value::Null,
        Expr::Bool(b) => Value::Bool(*b),
        Expr::Integer(n) => Value::Integer(*n),
        Expr::Float(f) => Value::Float(*f),
        Expr::String(s) => Value::String(s.clone()),
        Expr::Array(elements) => {
            // Convert array elements
            let mut arr = Vec::new();
            for (idx, elem) in elements.iter().enumerate() {
                let key = if let Some(key_expr) = &elem.key {
                    // Has explicit key
                    ArrayKey::from_value(&expr_to_simple_value(key_expr))
                } else {
                    // Auto-incrementing numeric key
                    ArrayKey::Integer(idx as i64)
                };
                let value = expr_to_simple_value(&elem.value);
                arr.push((key, value));
            }
            Value::Array(arr)
        }
        // For other expressions, return a placeholder string
        _ => Value::String(format!("<expression>")),
    }
}

/// get_class_attributes - Get attributes for a class
/// Usage: get_class_attributes(string $class_name): array
pub fn get_class_attributes(
    args: &[Value],
    classes: &std::collections::HashMap<String, crate::interpreter::ClassDefinition>,
) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("get_class_attributes() expects exactly 1 parameter".to_string());
    }

    let class_name = args[0].to_string_val();
    let class_name_lower = class_name.to_lowercase();

    if let Some(class_def) = classes.get(&class_name_lower) {
        let mut attrs = Vec::new();
        for (idx, attr) in class_def.attributes.iter().enumerate() {
            attrs.push((
                ArrayKey::Integer(idx as i64),
                attribute_to_value(attr),
            ));
        }
        Ok(Value::Array(attrs))
    } else {
        Err(format!("Class '{}' not found", class_name))
    }
}

/// get_method_attributes - Get attributes for a class method
/// Usage: get_method_attributes(string $class_name, string $method_name): array
pub fn get_method_attributes(
    args: &[Value],
    classes: &std::collections::HashMap<String, crate::interpreter::ClassDefinition>,
) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("get_method_attributes() expects exactly 2 parameters".to_string());
    }

    let class_name = args[0].to_string_val();
    let method_name = args[1].to_string_val();
    let class_name_lower = class_name.to_lowercase();

    if let Some(class_def) = classes.get(&class_name_lower) {
        let method_name_lower = method_name.to_lowercase();
        if let Some(method) = class_def.methods.get(&method_name_lower) {
            let mut attrs = Vec::new();
            for (idx, attr) in method.attributes.iter().enumerate() {
                attrs.push((
                    ArrayKey::Integer(idx as i64),
                    attribute_to_value(attr),
                ));
            }
            Ok(Value::Array(attrs))
        } else {
            Err(format!("Method '{}::{}' not found", class_name, method_name))
        }
    } else {
        Err(format!("Class '{}' not found", class_name))
    }
}

/// get_property_attributes - Get attributes for a class property
/// Usage: get_property_attributes(string $class_name, string $property_name): array
pub fn get_property_attributes(
    args: &[Value],
    classes: &std::collections::HashMap<String, crate::interpreter::ClassDefinition>,
) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("get_property_attributes() expects exactly 2 parameters".to_string());
    }

    let class_name = args[0].to_string_val();
    let property_name = args[1].to_string_val();
    let class_name_lower = class_name.to_lowercase();

    if let Some(class_def) = classes.get(&class_name_lower) {
        // Find property by name (properties use '$name' format)
        let prop_name_with_dollar = if !property_name.starts_with('$') {
            format!("${}", property_name)
        } else {
            property_name.clone()
        };

        if let Some(prop) = class_def.properties.iter().find(|p| p.name == prop_name_with_dollar) {
            let mut attrs = Vec::new();
            for (idx, attr) in prop.attributes.iter().enumerate() {
                attrs.push((
                    ArrayKey::Integer(idx as i64),
                    attribute_to_value(attr),
                ));
            }
            Ok(Value::Array(attrs))
        } else {
            Err(format!("Property '{}::{}' not found", class_name, property_name))
        }
    } else {
        Err(format!("Class '{}' not found", class_name))
    }
}

/// get_function_attributes - Get attributes for a function
/// Usage: get_function_attributes(string $function_name): array
pub fn get_function_attributes(
    args: &[Value],
    functions: &std::collections::HashMap<String, crate::interpreter::UserFunction>,
) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("get_function_attributes() expects exactly 1 parameter".to_string());
    }

    let function_name = args[0].to_string_val();
    let function_name_lower = function_name.to_lowercase();

    if let Some(func) = functions.get(&function_name_lower) {
        let mut attrs = Vec::new();
        for (idx, attr) in func.attributes.iter().enumerate() {
            attrs.push((
                ArrayKey::Integer(idx as i64),
                attribute_to_value(attr),
            ));
        }
        Ok(Value::Array(attrs))
    } else {
        Err(format!("Function '{}' not found", function_name))
    }
}

/// get_parameter_attributes - Get attributes for a function parameter
/// Usage: get_parameter_attributes(string $function_name, string $parameter_name): array
pub fn get_parameter_attributes(
    args: &[Value],
    functions: &std::collections::HashMap<String, crate::interpreter::UserFunction>,
) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("get_parameter_attributes() expects exactly 2 parameters".to_string());
    }

    let function_name = args[0].to_string_val();
    let parameter_name = args[1].to_string_val();
    let function_name_lower = function_name.to_lowercase();

    if let Some(func) = functions.get(&function_name_lower) {
        // Find parameter by name (parameters use '$name' format)
        let param_name_with_dollar = if !parameter_name.starts_with('$') {
            format!("${}", parameter_name)
        } else {
            parameter_name.clone()
        };

        if let Some(param) = func.params.iter().find(|p| p.name == param_name_with_dollar) {
            let mut attrs = Vec::new();
            for (idx, attr) in param.attributes.iter().enumerate() {
                attrs.push((
                    ArrayKey::Integer(idx as i64),
                    attribute_to_value(attr),
                ));
            }
            Ok(Value::Array(attrs))
        } else {
            Err(format!("Parameter '{}' in function '{}' not found", parameter_name, function_name))
        }
    } else {
        Err(format!("Function '{}' not found", function_name))
    }
}

/// get_method_parameter_attributes - Get attributes for a method parameter
/// Usage: get_method_parameter_attributes(string $class_name, string $method_name, string $parameter_name): array
pub fn get_method_parameter_attributes(
    args: &[Value],
    classes: &std::collections::HashMap<String, crate::interpreter::ClassDefinition>,
) -> Result<Value, String> {
    if args.len() != 3 {
        return Err("get_method_parameter_attributes() expects exactly 3 parameters".to_string());
    }

    let class_name = args[0].to_string_val();
    let method_name = args[1].to_string_val();
    let parameter_name = args[2].to_string_val();
    let class_name_lower = class_name.to_lowercase();

    if let Some(class_def) = classes.get(&class_name_lower) {
        let method_name_lower = method_name.to_lowercase();
        if let Some(method) = class_def.methods.get(&method_name_lower) {
            // Find parameter by name
            let param_name_with_dollar = if !parameter_name.starts_with('$') {
                format!("${}", parameter_name)
            } else {
                parameter_name.clone()
            };

            if let Some(param) = method.params.iter().find(|p| p.name == param_name_with_dollar) {
                let mut attrs = Vec::new();
                for (idx, attr) in param.attributes.iter().enumerate() {
                    attrs.push((
                        ArrayKey::Integer(idx as i64),
                        attribute_to_value(attr),
                    ));
                }
                Ok(Value::Array(attrs))
            } else {
                Err(format!("Parameter '{}' in method '{}::{}' not found", parameter_name, class_name, method_name))
            }
        } else {
            Err(format!("Method '{}::{}' not found", class_name, method_name))
        }
    } else {
        Err(format!("Class '{}' not found", class_name))
    }
}

/// get_interface_attributes - Get attributes for an interface
/// Usage: get_interface_attributes(string $interface_name): array
pub fn get_interface_attributes(
    args: &[Value],
    interfaces: &std::collections::HashMap<String, crate::interpreter::InterfaceDefinition>,
) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("get_interface_attributes() expects exactly 1 parameter".to_string());
    }

    let interface_name = args[0].to_string_val();
    let interface_name_lower = interface_name.to_lowercase();

    if let Some(interface_def) = interfaces.get(&interface_name_lower) {
        let mut attrs = Vec::new();
        for (idx, attr) in interface_def.attributes.iter().enumerate() {
            attrs.push((
                ArrayKey::Integer(idx as i64),
                attribute_to_value(attr),
            ));
        }
        Ok(Value::Array(attrs))
    } else {
        Err(format!("Interface '{}' not found", interface_name))
    }
}

/// get_trait_attributes - Get attributes for a trait
/// Usage: get_trait_attributes(string $trait_name): array
pub fn get_trait_attributes(
    args: &[Value],
    traits: &std::collections::HashMap<String, crate::interpreter::TraitDefinition>,
) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("get_trait_attributes() expects exactly 1 parameter".to_string());
    }

    let trait_name = args[0].to_string_val();
    let trait_name_lower = trait_name.to_lowercase();

    if let Some(trait_def) = traits.get(&trait_name_lower) {
        let mut attrs = Vec::new();
        for (idx, attr) in trait_def.attributes.iter().enumerate() {
            attrs.push((
                ArrayKey::Integer(idx as i64),
                attribute_to_value(attr),
            ));
        }
        Ok(Value::Array(attrs))
    } else {
        Err(format!("Trait '{}' not found", trait_name))
    }
}
```

### Step 2: Export Reflection Module (`src/interpreter/builtins/mod.rs`)

**Location:** Around line 5-10 (after existing module declarations)

**Current code:**
```rust
mod array;
mod math;
mod output;
mod string;
mod types;

pub use array::*;
pub use math::*;
pub use output::*;
pub use string::*;
pub use types::*;
```

**Updated code:**
```rust
mod array;
mod math;
mod output;
mod reflection;
mod string;
mod types;

pub use array::*;
pub use math::*;
pub use output::*;
pub use reflection::*;
pub use string::*;
pub use types::*;
```

### Step 3: Register Reflection Functions in Interpreter (`src/interpreter/mod.rs`)

**Location:** In the `call_function` method, around line 250-500 (where built-in functions are registered)

Find the section that handles built-in function calls (search for `"strlen"` or `"intval"` to locate the built-in function dispatch).

**Add these cases to the function name matching:**

```rust
            // Reflection functions (PHP 8.0 attributes)
            "get_class_attributes" => {
                builtins::get_class_attributes(&evaled_args, &self.classes)
            }
            "get_method_attributes" => {
                builtins::get_method_attributes(&evaled_args, &self.classes)
            }
            "get_property_attributes" => {
                builtins::get_property_attributes(&evaled_args, &self.classes)
            }
            "get_function_attributes" => {
                builtins::get_function_attributes(&evaled_args, &self.functions)
            }
            "get_parameter_attributes" => {
                builtins::get_parameter_attributes(&evaled_args, &self.functions)
            }
            "get_method_parameter_attributes" => {
                builtins::get_method_parameter_attributes(&evaled_args, &self.classes)
            }
            "get_interface_attributes" => {
                builtins::get_interface_attributes(&evaled_args, &self.interfaces)
            }
            "get_trait_attributes" => {
                builtins::get_trait_attributes(&evaled_args, &self.traits)
            }
```

**Note:** Since these functions need access to the interpreter's state (classes, functions, etc.), they need special handling. You may need to adjust the function signatures to pass references to these collections.

**Alternative approach (if the above doesn't work due to lifetime issues):**

Implement these functions directly in the interpreter instead of in the builtins module. In that case, add them as private methods in `impl<W: Write> Interpreter<W>` and call them from the match statement.

### Step 4: Handle ArrayElement Conversion (`src/interpreter/builtins/reflection.rs`)

**Note:** The reflection module needs access to `ArrayElement` from the AST. Make sure the import is correct:

**At the top of reflection.rs:**
```rust
use crate::ast::{Attribute, AttributeArgument, Expr, ArrayElement};
```

### Step 5: Add Test Cases

Create test files in `tests/attributes/` to verify the reflection API works correctly.

#### Test 1: `tests/attributes/reflection_class_attributes.vhpt`

```php
--TEST--
Attribute Reflection - get_class_attributes()
--FILE--
<?php
#[Route("/api/users")]
#[Middleware("auth")]
class UserController {}

$attrs = get_class_attributes("UserController");
echo count($attrs);
echo "\n";
echo $attrs[0]["name"];
echo "\n";
echo $attrs[1]["name"];
--EXPECT--
2
Route
Middleware
```

#### Test 2: `tests/attributes/reflection_method_attributes.vhpt`

```php
--TEST--
Attribute Reflection - get_method_attributes()
--FILE--
<?php
class UserController {
    #[Route("/api/users")]
    #[ValidateRequest]
    public function index() {}
}

$attrs = get_method_attributes("UserController", "index");
echo count($attrs);
echo "\n";
echo $attrs[0]["name"];
echo "\n";
echo $attrs[1]["name"];
--EXPECT--
2
Route
ValidateRequest
```

#### Test 3: `tests/attributes/reflection_property_attributes.vhpt`

```php
--TEST--
Attribute Reflection - get_property_attributes()
--FILE--
<?php
class User {
    #[MaxLength(100)]
    #[NotNull]
    public $name;
}

$attrs = get_property_attributes("User", "name");
echo count($attrs);
echo "\n";
echo $attrs[0]["name"];
echo "\n";
echo $attrs[1]["name"];
--EXPECT--
2
MaxLength
NotNull
```

#### Test 4: `tests/attributes/reflection_function_attributes.vhpt`

```php
--TEST--
Attribute Reflection - get_function_attributes()
--FILE--
<?php
#[Route("/health")]
#[Cache(ttl: 60)]
function healthCheck() {
    return "OK";
}

$attrs = get_function_attributes("healthCheck");
echo count($attrs);
echo "\n";
echo $attrs[0]["name"];
echo "\n";
echo $attrs[1]["name"];
--EXPECT--
2
Route
Cache
```

#### Test 5: `tests/attributes/reflection_parameter_attributes.vhpt`

```php
--TEST--
Attribute Reflection - get_parameter_attributes()
--FILE--
<?php
function createUser(#[NotBlank] #[Email] $email) {}

$attrs = get_parameter_attributes("createUser", "email");
echo count($attrs);
echo "\n";
echo $attrs[0]["name"];
echo "\n";
echo $attrs[1]["name"];
--EXPECT--
2
NotBlank
Email
```

#### Test 6: `tests/attributes/reflection_method_parameter_attributes.vhpt`

```php
--TEST--
Attribute Reflection - get_method_parameter_attributes()
--FILE--
<?php
class UserController {
    public function create(#[FromBody] #[Validate] $data) {}
}

$attrs = get_method_parameter_attributes("UserController", "create", "data");
echo count($attrs);
echo "\n";
echo $attrs[0]["name"];
echo "\n";
echo $attrs[1]["name"];
--EXPECT--
2
FromBody
Validate
```

#### Test 7: `tests/attributes/reflection_attribute_with_arguments.vhpt`

```php
--TEST--
Attribute Reflection - attribute with positional arguments
--FILE--
<?php
#[Route("/api/users", "GET")]
class UserController {}

$attrs = get_class_attributes("UserController");
$args = $attrs[0]["arguments"];
echo count($args);
echo "\n";
echo $args[0]["value"];
echo "\n";
echo $args[1]["value"];
--EXPECT--
2
/api/users
GET
```

#### Test 8: `tests/attributes/reflection_attribute_with_named_arguments.vhpt`

```php
--TEST--
Attribute Reflection - attribute with named arguments
--FILE--
<?php
#[Route(path: "/api/users", method: "POST")]
class UserController {}

$attrs = get_class_attributes("UserController");
$args = $attrs[0]["arguments"];
echo count($args);
echo "\n";
echo $args[0]["name"];
echo "\n";
echo $args[0]["value"];
echo "\n";
echo $args[1]["name"];
echo "\n";
echo $args[1]["value"];
--EXPECT--
2
path
/api/users
method
POST
```

#### Test 9: `tests/attributes/reflection_no_attributes.vhpt`

```php
--TEST--
Attribute Reflection - class with no attributes
--FILE--
<?php
class PlainClass {}

$attrs = get_class_attributes("PlainClass");
echo count($attrs);
--EXPECT--
0
```

#### Test 10: `tests/attributes/reflection_class_not_found.vhpt`

```php
--TEST--
Attribute Reflection - class not found error
--FILE--
<?php
$attrs = get_class_attributes("NonExistentClass");
--EXPECT_ERROR--
Class 'NonExistentClass' not found
```

#### Test 11: `tests/attributes/reflection_interface_attributes.vhpt`

```php
--TEST--
Attribute Reflection - get_interface_attributes()
--FILE--
<?php
#[Deprecated]
#[Replaced(by: "UserServiceV2")]
interface UserService {}

$attrs = get_interface_attributes("UserService");
echo count($attrs);
echo "\n";
echo $attrs[0]["name"];
echo "\n";
echo $attrs[1]["name"];
--EXPECT--
2
Deprecated
Replaced
```

#### Test 12: `tests/attributes/reflection_trait_attributes.vhpt`

```php
--TEST--
Attribute Reflection - get_trait_attributes()
--FILE--
<?php
#[Timestampable]
trait Timestamps {}

$attrs = get_trait_attributes("Timestamps");
echo count($attrs);
echo "\n";
echo $attrs[0]["name"];
--EXPECT--
1
Timestampable
```

#### Test 13: `tests/attributes/reflection_complex_attribute_values.vhpt`

```php
--TEST--
Attribute Reflection - complex attribute values (arrays)
--FILE--
<?php
#[Route("/api", methods: ["GET", "POST"])]
class ApiController {}

$attrs = get_class_attributes("ApiController");
$args = $attrs[0]["arguments"];
echo $args[0]["value"];
echo "\n";
echo $args[1]["name"];
echo "\n";
echo count($args[1]["value"]);
--EXPECT--
/api
methods
2
```

#### Test 14: `tests/attributes/reflection_constructor_promotion_attributes.vhpt`

```php
--TEST--
Attribute Reflection - constructor promoted property attributes
--FILE--
<?php
class User {
    public function __construct(
        #[MaxLength(100)]
        public $name
    ) {}
}

$attrs = get_property_attributes("User", "name");
echo count($attrs);
echo "\n";
echo $attrs[0]["name"];
--EXPECT--
1
MaxLength
```

### Step 6: Update Documentation

#### 6a. Update `AGENTS.md`

**Location:** Line 216 (in the Attributes section)

**Change:**
```markdown
### Attributes (PHP 8.0)
- [x] Basic attribute syntax: `#[AttributeName]`
- [x] Attributes with positional arguments: `#[Route("/path")]`
- [x] Attributes with named arguments: `#[Route(path: "/path")]`
- [x] Multiple attributes: `#[Attr1] #[Attr2]` or `#[Attr1, Attr2]`
- [x] Attributes on classes, interfaces, traits
- [x] Attributes on methods, properties, functions
- [x] Attributes on parameters (including constructor promotion)
- [x] Attributes on interface methods and constants
- [x] Attributes parsing and storage in AST
- [x] Attribute reflection API (retrieving attributes at runtime)
```

**Also update the Built-in Functions count around line 160:**

**Change:**
```markdown
### Built-in Functions (71)
- [x] **String** (23): `strlen`, `substr`, `strtoupper`, `strtolower`, `trim`, `ltrim`, `rtrim`, `str_repeat`, `str_replace`, `strpos`, `strrev`, `ucfirst`, `lcfirst`, `ucwords`, `str_starts_with`, `str_ends_with`, `str_contains`, `str_pad`, `explode`, `implode`/`join`, `sprintf`, `chr`, `ord`
- [x] **Math** (9): `abs`, `ceil`, `floor`, `round`, `max`, `min`, `pow`, `sqrt`, `rand`/`mt_rand`
- [x] **Array** (13): `count`/`sizeof`, `array_push`, `array_pop`, `array_shift`, `array_unshift`, `array_keys`, `array_values`, `in_array`, `array_search`, `array_reverse`, `array_merge`, `array_key_exists`, `range`
- [x] **Type** (14): `intval`, `floatval`/`doubleval`, `strval`, `boolval`, `gettype`, `is_null`, `is_bool`, `is_int`/`is_integer`/`is_long`, `is_float`/`is_double`/`is_real`, `is_string`, `is_array`, `is_numeric`, `isset`, `empty`
- [x] **Output** (4): `print`, `var_dump`, `print_r`, `printf`
- [x] **Reflection** (8): `get_class_attributes`, `get_method_attributes`, `get_property_attributes`, `get_function_attributes`, `get_parameter_attributes`, `get_method_parameter_attributes`, `get_interface_attributes`, `get_trait_attributes`
```

**Update the roadmap (line 417):**

```markdown
### Phase 6: Modern PHP 8.x Features (In Progress)
- [x] Match Expressions (PHP 8.0)
- [x] Named Arguments (PHP 8.0)
- [x] Attributes (PHP 8.0) - Full support including reflection API
- [ ] Enums (PHP 8.1)
- [ ] Fibers (PHP 8.1)
- [ ] Pipe Operator (PHP 8.5)
```

#### 6b. Update `README.md`

Add reflection functions to the built-in functions list and update the attributes feature description to mention reflection support.

#### 6c. Update `docs/features.md`

**Add to the Attributes section:**

```markdown
### Runtime Reflection

VHP provides built-in functions to retrieve attributes at runtime:

```php
#[Route("/api/users")]
class UserController {
    #[ValidateRequest]
    public function create(#[FromBody] $data) {}
}

// Get class attributes
$class_attrs = get_class_attributes("UserController");
// Returns: [["name" => "Route", "arguments" => [["name" => null, "value" => "/api/users"]]]]

// Get method attributes
$method_attrs = get_method_attributes("UserController", "create");

// Get parameter attributes
$param_attrs = get_method_parameter_attributes("UserController", "create", "data");
```

**Available Reflection Functions:**

| Function | Description |
|----------|-------------|
| `get_class_attributes($class)` | Get all attributes for a class |
| `get_method_attributes($class, $method)` | Get all attributes for a method |
| `get_property_attributes($class, $property)` | Get all attributes for a property |
| `get_function_attributes($function)` | Get all attributes for a function |
| `get_parameter_attributes($function, $param)` | Get all attributes for a function parameter |
| `get_method_parameter_attributes($class, $method, $param)` | Get all attributes for a method parameter |
| `get_interface_attributes($interface)` | Get all attributes for an interface |
| `get_trait_attributes($trait)` | Get all attributes for a trait |

**Return Format:**

Each function returns an array of attributes. Each attribute is an associative array:

```php
[
    "name" => "AttributeName",
    "arguments" => [
        [
            "name" => "param_name",  // null for positional args
            "value" => "param_value"
        ]
    ]
]
```
```

## Key Considerations

### PHP Compatibility

1. **Standard Reflection API**: PHP 8.0 uses `ReflectionClass::getAttributes()`, `ReflectionMethod::getAttributes()`, etc. VHP provides simplified built-in functions instead since it doesn't have a full Reflection API yet.

2. **Attribute Instances**: In real PHP, calling `$attr->newInstance()` would instantiate the attribute class. VHP just returns the raw metadata as arrays.

3. **ReflectionAttribute**: PHP returns `ReflectionAttribute` objects, not arrays. VHP's simplified approach returns associative arrays.

4. **Filtering**: PHP's `getAttributes($name, $flags)` supports filtering by attribute name and flags. VHP's initial implementation returns all attributes without filtering (can be added later).

### Implementation Notes

1. **Lifetime Issues**: The reflection functions need access to interpreter state (classes, functions, etc.). This may cause lifetime/borrowing issues in Rust. Solutions:
   - Pass references to collections as parameters
   - Implement functions directly in the interpreter rather than in the builtins module
   - Clone data where necessary

2. **Expression Evaluation**: Attribute arguments can be complex expressions. The initial implementation converts only literal values (strings, numbers, bools, simple arrays). For complex expressions, more work is needed.

3. **Performance**: Reflection is typically used sparingly (e.g., at framework startup). The current implementation clones data, which is acceptable for this use case.

4. **Error Handling**: Functions should return clear errors when classes/methods/properties are not found.

### Edge Cases to Handle

1. **Case Insensitivity**: Class and function names are case-insensitive in PHP (and VHP). Reflection functions must normalize names.

2. **Property Names**: Properties include the `$` prefix in the AST. Reflection functions should accept names with or without `$`.

3. **Constructor Promotion**: Promoted parameters become properties. Make sure their attributes are accessible via `get_property_attributes()`.

4. **Trait Methods**: When a trait is used by a class, its method attributes should be accessible via the class's method attributes.

5. **Empty Attributes**: Classes/methods with no attributes should return empty arrays, not errors.

6. **Interface Methods**: Attributes on interface method signatures should be accessible (future enhancement if needed).

### Testing Strategy

Tests should cover:
1. Basic retrieval of attributes from classes, methods, properties, functions, parameters
2. Multiple attributes on the same declaration
3. Attributes with no arguments
4. Attributes with positional arguments
5. Attributes with named arguments
6. Attributes with mixed arguments
7. Complex attribute values (arrays, nested structures)
8. Empty attribute lists (no attributes)
9. Error cases (class/method not found)
10. Case-insensitive name lookup
11. Constructor promotion with attributes
12. Interface and trait attributes

## Open Questions/Decisions

### 1. Should we implement full ReflectionClass/ReflectionMethod APIs?

**Decision**: No, not yet. Start with simple built-in functions that return arrays. Full Reflection API can be added later as a separate feature.

**Rationale**:
- Simpler to implement
- Sufficient for most use cases (routing, validation, dependency injection)
- Can be extended incrementally

### 2. How to handle complex expression evaluation in attribute arguments?

**Decision**: Start with literal values only (strings, numbers, bools, simple arrays). Return `"<expression>"` placeholder for complex expressions.

**Rationale**:
- Most attributes use simple literals
- Full expression evaluation would require access to interpreter context
- Can be enhanced later if needed

### 3. Should attribute names be case-sensitive?

**Decision**: Yes, attribute names should be case-sensitive (following PHP behavior).

**Rationale**:
- PHP class names are case-insensitive, but attribute usage in metadata systems typically expects case-sensitive names
- Matches PHP 8.0 behavior

### 4. How to represent attributes in the return value?

**Decision**: Use associative arrays with `"name"` and `"arguments"` keys. Each argument has `"name"` (null for positional) and `"value"`.

**Rationale**:
- Easy to work with in PHP code
- Matches PHP's array-based APIs
- Can be converted to objects later if needed

### 5. Should we support filtering attributes by name?

**Decision**: Not in the initial implementation. Can be added later as an optional parameter.

**Rationale**:
- Simpler to implement initially
- Users can filter the returned array themselves
- Can be added backward-compatibly later

## Reference Implementation

For implementation patterns, reference these existing features:

| Pattern | Reference | What to Learn |
|---------|-----------|---------------|
| Built-in function module | `src/interpreter/builtins/string.rs` | How to structure a built-ins module |
| Accessing interpreter state | `call_function` in `interpreter/mod.rs` | How to pass context to built-ins |
| Array construction | `array.rs` built-ins | How to build PHP arrays from Rust |
| Error handling | Any built-in function | Error message patterns |
| Case-insensitive lookup | Class/function calls | How VHP handles case insensitivity |

## Implementation Checklist

- [ ] Create `src/interpreter/builtins/reflection.rs` with all 8 functions
- [ ] Export reflection module in `src/interpreter/builtins/mod.rs`
- [ ] Register all 8 reflection functions in `src/interpreter/mod.rs`
- [ ] Add helper functions to convert `Attribute` to `Value`
- [ ] Add helper functions to convert `AttributeArgument` to `Value`
- [ ] Add helper functions to convert simple `Expr` to `Value`
- [ ] Handle imports for `ArrayElement` if needed
- [ ] Add 14 test files for reflection API
- [ ] Update `AGENTS.md` documentation
- [ ] Update `README.md` features and built-ins list
- [ ] Update `docs/features.md` with reflection API documentation
- [ ] Run full test suite to ensure no regressions
- [ ] Verify clippy passes with no warnings

## Next Steps After Completion

After implementing the Attribute Reflection API, the next logical features in Phase 6 are:

1. **Enums (PHP 8.1)**: Type-safe enumerations with optional backing values
2. **Fibers (PHP 8.1)**: Lightweight cooperative multitasking primitives
3. **Pipe Operator (PHP 8.5)**: Functional piping operator for cleaner code

The Attribute Reflection API completes the attributes feature, making it fully usable for real-world applications like routing, validation, dependency injection, and metadata-driven frameworks.
