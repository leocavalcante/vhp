# Plan: Type Declarations (Basic Type Hints)

## Overview

Type declarations allow specifying types for function parameters and return values. This is foundational to PHP's type system and affects how values are validated at runtime.

**PHP Example:**
```php
<?php
// Parameter type hints
function greet(string $name): string {
    return "Hello, $name";
}

// Multiple parameter types
function add(int $a, int $b): int {
    return $a + $b;
}

// Nullable types (PHP 7.1+)
function findUser(?int $id): ?User {
    return $id ? new User() : null;
}

// Union types (PHP 8.0+)
function process(int|string $value): int|string {
    return $value;
}

// void return type (PHP 7.1+)
function logMessage(string $msg): void {
    echo $msg;
}

// mixed type (PHP 8.0+)
function anything(mixed $value): mixed {
    return $value;
}
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/ast/stmt.rs` | Add `TypeHint` struct, update `FunctionParam` |
| `src/ast/expr.rs` | (no changes) |
| `src/parser/stmt/mod.rs` | Parse type hints in parameters and return types |
| `src/interpreter/functions/mod.rs` | Type validation at runtime |
| `src/interpreter/value.rs` | Type checking helpers |
| `tests/types/*.vhpt` | Test files |

## Implementation Steps

### Step 1: Define Type Hint Structure (`src/ast/stmt.rs`)

Add a new struct to represent type hints. Place this near the top of the file:

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

impl TypeHint {
    /// Check if this type hint allows null values
    pub fn is_nullable(&self) -> bool {
        match self {
            TypeHint::Nullable(_) => true,
            TypeHint::Union(types) => types.iter().any(|t| matches!(t, TypeHint::Simple(s) if s == "null")),
            TypeHint::Simple(s) => s == "mixed" || s == "null",
            _ => false,
        }
    }
}
```

### Step 2: Update FunctionParam (`src/ast/stmt.rs`)

Modify the `FunctionParam` struct to include a type hint:

```rust
/// Function parameter
#[derive(Debug, Clone)]
pub struct FunctionParam {
    pub name: String,
    pub type_hint: Option<TypeHint>,  // NEW: Type hint for parameter
    pub default: Option<Expr>,
    pub by_ref: bool,
    pub visibility: Option<Visibility>,
    pub readonly: bool,
    pub attributes: Vec<Attribute>,
}
```

### Step 3: Add Return Type to Function Statement

Update the `Function` variant in `Stmt` enum:

```rust
    Function {
        name: String,
        params: Vec<FunctionParam>,
        return_type: Option<TypeHint>,  // NEW: Return type hint
        body: Vec<Stmt>,
        attributes: Vec<Attribute>,
    },
```

Also update `Method` struct:

```rust
#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub visibility: Visibility,
    pub params: Vec<FunctionParam>,
    pub return_type: Option<TypeHint>,  // NEW: Return type hint
    pub body: Vec<Stmt>,
    pub attributes: Vec<Attribute>,
}
```

And `InterfaceMethodSignature`:

```rust
#[derive(Debug, Clone)]
pub struct InterfaceMethodSignature {
    pub name: String,
    pub params: Vec<FunctionParam>,
    pub return_type: Option<TypeHint>,  // NEW: Return type hint
    pub attributes: Vec<Attribute>,
}
```

### Step 4: Update Parser to Parse Type Hints (`src/parser/stmt/mod.rs`)

Add a function to parse type hints:

```rust
/// Parse a type hint
/// Supports: int, string, ?int, int|string, array, callable, ClassName
fn parse_type_hint(&mut self) -> Result<TypeHint, String> {
    // Check for nullable prefix ?
    let nullable = if self.check(&TokenKind::QuestionMark) {
        self.advance();
        true
    } else {
        false
    };
    
    // Parse the base type
    let base_type = self.parse_single_type()?;
    
    // Check for union | or intersection &
    if self.check_char('|') || self.check(&TokenKind::Or) {
        let mut types = vec![base_type];
        while self.check_char('|') || self.check(&TokenKind::Or) {
            self.advance();
            types.push(self.parse_single_type()?);
        }
        if nullable {
            // ?int|string is not valid syntax, but int|string|null is
            return Err("Cannot use nullable syntax with union types, use |null instead".to_string());
        }
        return Ok(TypeHint::Union(types));
    }
    
    // Check for intersection &
    if self.check_char('&') {
        let mut types = vec![base_type];
        while self.check_char('&') {
            self.advance();
            types.push(self.parse_single_type()?);
        }
        if nullable {
            return Err("Cannot use nullable syntax with intersection types".to_string());
        }
        return Ok(TypeHint::Intersection(types));
    }
    
    // Apply nullable wrapper if needed
    if nullable {
        Ok(TypeHint::Nullable(Box::new(base_type)))
    } else {
        Ok(base_type)
    }
}

/// Parse a single type (without union/intersection)
fn parse_single_type(&mut self) -> Result<TypeHint, String> {
    if let TokenKind::Identifier(name) = &self.current_token().kind {
        let type_name = name.to_lowercase();
        let original_name = name.clone();
        self.advance();
        
        match type_name.as_str() {
            "int" | "integer" => Ok(TypeHint::Simple("int".to_string())),
            "string" => Ok(TypeHint::Simple("string".to_string())),
            "float" | "double" => Ok(TypeHint::Simple("float".to_string())),
            "bool" | "boolean" => Ok(TypeHint::Simple("bool".to_string())),
            "array" => Ok(TypeHint::Simple("array".to_string())),
            "object" => Ok(TypeHint::Simple("object".to_string())),
            "callable" => Ok(TypeHint::Simple("callable".to_string())),
            "iterable" => Ok(TypeHint::Simple("iterable".to_string())),
            "mixed" => Ok(TypeHint::Simple("mixed".to_string())),
            "void" => Ok(TypeHint::Void),
            "never" => Ok(TypeHint::Never),
            "static" => Ok(TypeHint::Static),
            "self" => Ok(TypeHint::SelfType),
            "parent" => Ok(TypeHint::ParentType),
            "null" => Ok(TypeHint::Simple("null".to_string())),
            "false" => Ok(TypeHint::Simple("false".to_string())),
            "true" => Ok(TypeHint::Simple("true".to_string())),
            _ => Ok(TypeHint::Class(original_name)), // Class/interface name
        }
    } else {
        Err(format!("Expected type name, got {:?}", self.current_token().kind))
    }
}
```

### Step 5: Update Parameter Parsing

Modify `parse_function_params()` to handle type hints:

```rust
fn parse_function_param(&mut self) -> Result<FunctionParam, String> {
    // Parse attributes first
    let attributes = self.parse_attributes()?;
    
    // Parse optional visibility (for constructor property promotion)
    let visibility = self.parse_optional_visibility()?;
    
    // Parse optional readonly
    let readonly = self.check(&TokenKind::Readonly);
    if readonly {
        self.advance();
    }
    
    // Parse optional type hint
    let type_hint = if !self.check(&TokenKind::Variable("".to_string())) 
                    && self.is_type_start() {
        Some(self.parse_type_hint()?)
    } else {
        None
    };
    
    // Parse by-reference &
    let by_ref = self.check_char('&');
    if by_ref {
        self.advance();
    }
    
    // Parse variable name
    let name = if let TokenKind::Variable(var_name) = &self.current_token().kind {
        let n = var_name.clone();
        self.advance();
        n
    } else {
        return Err("Expected parameter name".to_string());
    };
    
    // Parse optional default value
    let default = if self.check(&TokenKind::Assign) {
        self.advance();
        Some(self.parse_expression()?)
    } else {
        None
    };
    
    Ok(FunctionParam {
        name,
        type_hint,
        default,
        by_ref,
        visibility,
        readonly,
        attributes,
    })
}

/// Check if current token could start a type hint
fn is_type_start(&self) -> bool {
    match &self.current_token().kind {
        TokenKind::QuestionMark => true, // nullable
        TokenKind::Identifier(name) => {
            let lower = name.to_lowercase();
            matches!(lower.as_str(), 
                "int" | "integer" | "string" | "float" | "double" | 
                "bool" | "boolean" | "array" | "object" | "callable" |
                "iterable" | "mixed" | "void" | "never" | "static" |
                "self" | "parent" | "null" | "false" | "true"
            ) || name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
        }
        _ => false,
    }
}
```

### Step 6: Parse Return Type

Update function parsing to handle return type after `)`:

```rust
// After parsing parameters in parse_function()
self.expect(&TokenKind::RightParen)?;

// Parse optional return type
let return_type = if self.check(&TokenKind::Colon) {
    self.advance(); // consume ':'
    Some(self.parse_type_hint()?)
} else {
    None
};

self.expect(&TokenKind::LeftBrace)?;
// ... parse body
```

### Step 7: Add Type Validation Helpers (`src/interpreter/value.rs`)

Add methods to check value types:

```rust
impl Value {
    /// Check if value matches a type hint
    pub fn matches_type(&self, type_hint: &TypeHint) -> bool {
        match type_hint {
            TypeHint::Simple(name) => self.matches_simple_type(name),
            TypeHint::Nullable(inner) => {
                matches!(self, Value::Null) || self.matches_type(inner)
            }
            TypeHint::Union(types) => {
                types.iter().any(|t| self.matches_type(t))
            }
            TypeHint::Intersection(types) => {
                types.iter().all(|t| self.matches_type(t))
            }
            TypeHint::Class(class_name) => {
                if let Value::Object(obj) = self {
                    obj.is_instance_of(class_name)
                } else {
                    false
                }
            }
            TypeHint::Void => false, // void is for return types only
            TypeHint::Never => false, // never is for return types only
            TypeHint::Static => false, // Requires class context
            TypeHint::SelfType => false, // Requires class context
            TypeHint::ParentType => false, // Requires class context
        }
    }
    
    fn matches_simple_type(&self, type_name: &str) -> bool {
        match (type_name, self) {
            ("int", Value::Integer(_)) => true,
            ("string", Value::String(_)) => true,
            ("float", Value::Float(_)) => true,
            ("float", Value::Integer(_)) => true, // int is compatible with float
            ("bool", Value::Bool(_)) => true,
            ("array", Value::Array(_)) => true,
            ("object", Value::Object(_)) => true,
            ("callable", Value::Callable(_)) => true,
            ("callable", Value::String(_)) => true, // function name
            ("iterable", Value::Array(_)) => true,
            ("mixed", _) => true,
            ("null", Value::Null) => true,
            ("false", Value::Bool(false)) => true,
            ("true", Value::Bool(true)) => true,
            _ => false,
        }
    }
    
    /// Get type name for error messages
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Integer(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
            Value::Callable(_) => "callable",
            // ... other variants
        }
    }
}
```

### Step 8: Add Type Validation at Runtime (`src/interpreter/functions/mod.rs`)

When calling functions, validate parameter types:

```rust
/// Validate argument against type hint
fn validate_argument(
    &self,
    param: &FunctionParam,
    value: &Value,
    func_name: &str,
    arg_position: usize,
) -> Result<(), String> {
    if let Some(ref type_hint) = param.type_hint {
        if !value.matches_type(type_hint) {
            return Err(format!(
                "Argument {} passed to {}() must be of type {}, {} given",
                arg_position + 1,
                func_name,
                self.format_type_hint(type_hint),
                value.type_name()
            ));
        }
    }
    Ok(())
}

/// Validate return value against type hint
fn validate_return(
    &self,
    return_type: &Option<TypeHint>,
    value: &Value,
    func_name: &str,
) -> Result<(), String> {
    if let Some(ref type_hint) = return_type {
        match type_hint {
            TypeHint::Void => {
                if !matches!(value, Value::Null) {
                    return Err(format!(
                        "{}(): Return value must be of type void, {} returned",
                        func_name,
                        value.type_name()
                    ));
                }
            }
            TypeHint::Never => {
                return Err(format!(
                    "{}(): never-returning function must not return",
                    func_name
                ));
            }
            _ => {
                if !value.matches_type(type_hint) {
                    return Err(format!(
                        "{}(): Return value must be of type {}, {} returned",
                        func_name,
                        self.format_type_hint(type_hint),
                        value.type_name()
                    ));
                }
            }
        }
    }
    Ok(())
}

fn format_type_hint(&self, hint: &TypeHint) -> String {
    match hint {
        TypeHint::Simple(s) => s.clone(),
        TypeHint::Nullable(inner) => format!("?{}", self.format_type_hint(inner)),
        TypeHint::Union(types) => types.iter()
            .map(|t| self.format_type_hint(t))
            .collect::<Vec<_>>()
            .join("|"),
        TypeHint::Intersection(types) => types.iter()
            .map(|t| self.format_type_hint(t))
            .collect::<Vec<_>>()
            .join("&"),
        TypeHint::Class(name) => name.clone(),
        TypeHint::Void => "void".to_string(),
        TypeHint::Never => "never".to_string(),
        TypeHint::Static => "static".to_string(),
        TypeHint::SelfType => "self".to_string(),
        TypeHint::ParentType => "parent".to_string(),
    }
}
```

### Step 9: Add Tests (`tests/types/`)

**tests/types/basic_parameter_types.vhpt**
```
--TEST--
Basic parameter type hints
--FILE--
<?php
function greet(string $name): string {
    return "Hello, $name";
}
echo greet("World");
--EXPECT--
Hello, World
```

**tests/types/int_parameter.vhpt**
```
--TEST--
Integer parameter type
--FILE--
<?php
function double(int $n): int {
    return $n * 2;
}
echo double(5);
--EXPECT--
10
```

**tests/types/type_error.vhpt**
```
--TEST--
Type error on wrong parameter type
--FILE--
<?php
function requireInt(int $n) {
    echo $n;
}
requireInt("not an int");
--EXPECT_ERROR--
must be of type int, string given
```

**tests/types/nullable_type.vhpt**
```
--TEST--
Nullable type hint
--FILE--
<?php
function maybeString(?string $s): ?string {
    return $s;
}
echo maybeString(null) ?? "null";
echo "\n";
echo maybeString("hello");
--EXPECT--
null
hello
```

**tests/types/union_types.vhpt**
```
--TEST--
Union types (PHP 8.0+)
--FILE--
<?php
function process(int|string $value): int|string {
    return $value;
}
echo process(42) . "\n";
echo process("hello");
--EXPECT--
42
hello
```

**tests/types/void_return.vhpt**
```
--TEST--
Void return type
--FILE--
<?php
function logIt(string $msg): void {
    echo $msg;
}
logIt("logged");
--EXPECT--
logged
```

**tests/types/void_return_error.vhpt**
```
--TEST--
Void function cannot return value
--FILE--
<?php
function wrong(): void {
    return "something";
}
wrong();
--EXPECT_ERROR--
must be of type void
```

**tests/types/class_type_hint.vhpt**
```
--TEST--
Class type hint
--FILE--
<?php
class User {
    public string $name;
    public function __construct(string $name) {
        $this->name = $name;
    }
}

function greetUser(User $user): string {
    return "Hello, " . $user->name;
}

$user = new User("Alice");
echo greetUser($user);
--EXPECT--
Hello, Alice
```

**tests/types/mixed_type.vhpt**
```
--TEST--
Mixed type accepts anything
--FILE--
<?php
function anything(mixed $x): mixed {
    return $x;
}
echo anything(42) . "\n";
echo anything("str") . "\n";
echo anything(true) ? "true" : "false";
--EXPECT--
42
str
true
```

**tests/types/array_type.vhpt**
```
--TEST--
Array type hint
--FILE--
<?php
function sumArray(array $numbers): int {
    $sum = 0;
    foreach ($numbers as $n) {
        $sum = $sum + $n;
    }
    return $sum;
}
echo sumArray([1, 2, 3, 4]);
--EXPECT--
10
```

### Step 10: Update Documentation

Update:
- `AGENTS.md`: Add type declarations to Phase 7 status
- `docs/features.md`: Document type hints
- `docs/roadmap.md`: Update Phase 7 checklist

## Key Considerations

1. **Coercion Mode**: PHP has strict mode (`declare(strict_types=1)`) vs coercive mode. Start with coercive mode (default PHP behavior).
2. **Type inheritance**: Return types are covariant, parameter types are contravariant.
3. **Self/parent/static**: Only valid in class context.
4. **Void vs null**: `void` means no return statement; `null` is a value.
5. **Never**: Function that always throws or exits (no return).

## PHP Compatibility Notes

| Feature | PHP Version |
|---------|-------------|
| Basic type hints | 7.0 |
| Nullable types `?` | 7.1 |
| `void` return type | 7.1 |
| `object` type | 7.2 |
| Union types `\|` | 8.0 |
| `mixed` type | 8.0 |
| `static` return type | 8.0 |
| `never` return type | 8.1 |
| Intersection types `&` | 8.1 |
| `true`/`false`/`null` standalone | 8.2 |
| DNF types `(A&B)\|C` | 8.2 |

## Reference Implementation

- Parameter parsing in existing `parse_function_params()`
- Type handling similar to `Value` enum matching
- Error messages follow PHP format: `must be of type X, Y given`
