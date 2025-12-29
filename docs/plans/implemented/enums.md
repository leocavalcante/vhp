# Plan: Enums (PHP 8.1)

## Overview

Enums (Enumerations) were introduced in PHP 8.1 as a way to define a type that has a fixed set of possible values. PHP supports two types of enums:

1. **Pure enums**: Cases without scalar values
2. **Backed enums**: Cases backed by `string` or `int` values

Enums can have methods, implement interfaces, and provide built-in methods for reflection and validation. They provide type safety and make code more expressive and self-documenting.

**PHP Example:**

```php
// Pure enum (no backing values)
enum Status {
    case Pending;
    case Active;
    case Archived;
}

$status = Status::Active;
echo $status->name; // "Active"

// Backed enum (with int backing)
enum Priority: int {
    case Low = 1;
    case Medium = 5;
    case High = 10;
}

$priority = Priority::High;
echo $priority->name;  // "High"
echo $priority->value; // 10

// Backed enum (with string backing)
enum Color: string {
    case Red = 'red';
    case Green = 'green';
    case Blue = 'blue';
}

$color = Color::Red;
echo $color->value; // "red"

// Built-in methods
$cases = Priority::cases();  // Returns array of all cases
$medium = Priority::from(5); // Get case by value (throws on invalid)
$low = Priority::tryFrom(1); // Get case by value (returns null on invalid)
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/token.rs` | Add `Enum` token (line 43 after `Readonly`) |
| `src/lexer.rs` | Add "enum" keyword recognition (line ~170-200) |
| `src/ast/stmt.rs` | Add `EnumDeclaration` and `EnumCase` structs, add `Stmt::Enum` variant |
| `src/ast/expr.rs` | Add `Expr::EnumCase` for accessing cases like `Status::Active` |
| `src/parser/stmt.rs` | Add `parse_enum()` method |
| `src/parser/expr.rs` | Handle enum case access in `::` operator parsing |
| `src/interpreter/mod.rs` | Add enum storage, case access, built-in methods |
| `src/interpreter/value.rs` | Add `Value::EnumCase` variant |
| `tests/enums/*.vhpt` | Create comprehensive test suite (15+ tests) |
| `AGENTS.md` | Update roadmap marking enums as complete |
| `README.md` | Update features list |

## Implementation Steps

### Step 1: Add Tokens (`src/token.rs`)

**Location:** Line 43, after `Readonly` token

**Add:**

```rust
    Enum,         // enum
```

**Note:** `Case` token already exists (line 22) for switch-case statements. We'll reuse it for enum cases.

### Step 2: Update Lexer (`src/lexer.rs`)

**Location:** Around line 170-200 in the keyword matching section

**Find the keyword match block that looks like:**

```rust
match ident.to_lowercase().as_str() {
    "echo" => TokenKind::Echo,
    "true" => TokenKind::True,
    // ... other keywords ...
    "readonly" => TokenKind::Readonly,
```

**Add after the "readonly" line:**

```rust
    "enum" => TokenKind::Enum,
```

**Verification:** The `Case` keyword should already be recognized (for switch-case statements). Verify it exists in the match block.

### Step 3: Extend AST - Add Enum Structures (`src/ast/stmt.rs`)

**Location 1:** After `InterfaceConstant` struct (around line 67)

**Add:**

```rust
/// Enum case definition
#[derive(Debug, Clone)]
pub struct EnumCase {
    pub name: String,
    pub value: Option<Expr>, // Some(expr) for backed enums, None for pure enums
}

/// Enum backing type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnumBackingType {
    None,   // Pure enum
    Int,    // Backed by integers
    String, // Backed by strings
}
```

**Location 2:** In the `Stmt` enum (around line 165, after `Stmt::Class`)

**Add:**

```rust
    Enum {
        name: String,
        backing_type: EnumBackingType,
        cases: Vec<EnumCase>,
        methods: Vec<Method>, // Enums can have methods
        attributes: Vec<Attribute>, // PHP 8.0+
    },
```

### Step 4: Extend AST - Add Enum Case Expression (`src/ast/expr.rs`)

**Location:** In the `Expr` enum (around line 126, after `Expr::Match`)

**Add:**

```rust
    // Enum case access: EnumName::CASE
    EnumCase {
        enum_name: String,
        case_name: String,
    },
```

### Step 5: Update Parser - Parse Enum Declarations (`src/parser/stmt.rs`)

**Location 1:** In `parse_statement()` method (around line 200-250)

**Find the match block with `TokenKind::Class =>` and add before or after it:**

```rust
            TokenKind::Enum => {
                let stmt = self.parse_enum()?;
                Ok(Some(stmt))
            }
```

**Location 2:** Add new method at the end of `impl<'a> StmtParser<'a>` (around line 1000+)

**Add:**

```rust
    /// Parse enum declaration: enum Name: type { case Value; case Value = expr; ... }
    fn parse_enum(&mut self) -> Result<Stmt, String> {
        // Parse attributes (already consumed by caller)
        let attributes = Vec::new(); // Will be passed from caller when attributes are parsed before enum

        self.consume(TokenKind::Enum, "Expected 'enum' keyword")?;

        // Parse enum name
        let name = if let TokenKind::Identifier(id) = &self.current().kind {
            let name = id.clone();
            self.advance();
            name
        } else {
            return Err(format!(
                "Expected enum name at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        };

        // Check for backing type (: int or : string)
        let backing_type = if self.check(&TokenKind::Colon) {
            self.advance(); // consume ':'

            if let TokenKind::Identifier(type_name) = &self.current().kind {
                let type_lower = type_name.to_lowercase();
                let backing = match type_lower.as_str() {
                    "int" => EnumBackingType::Int,
                    "string" => EnumBackingType::String,
                    _ => {
                        return Err(format!(
                            "Invalid enum backing type '{}'. Only 'int' and 'string' are supported at line {}, column {}",
                            type_name,
                            self.current().line,
                            self.current().column
                        ));
                    }
                };
                self.advance(); // consume type name
                backing
            } else {
                return Err(format!(
                    "Expected backing type (int or string) at line {}, column {}",
                    self.current().line,
                    self.current().column
                ));
            }
        } else {
            EnumBackingType::None
        };

        self.consume(TokenKind::LeftBrace, "Expected '{' after enum name")?;

        let mut cases = Vec::new();
        let mut methods = Vec::new();

        // Parse cases and methods
        while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
            // Check for case or method
            if self.check(&TokenKind::Case) {
                // Parse enum case
                self.advance(); // consume 'case'

                let case_name = if let TokenKind::Identifier(id) = &self.current().kind {
                    let name = id.clone();
                    self.advance();
                    name
                } else {
                    return Err(format!(
                        "Expected case name at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ));
                };

                // Check for value assignment (backed enums)
                let value = if self.check(&TokenKind::Assign) {
                    if backing_type == EnumBackingType::None {
                        return Err(format!(
                            "Pure enum cannot have case values at line {}, column {}",
                            self.current().line,
                            self.current().column
                        ));
                    }
                    self.advance(); // consume '='
                    Some(self.parse_expression(Precedence::Lowest)?)
                } else {
                    if backing_type != EnumBackingType::None {
                        return Err(format!(
                            "Backed enum must have case values at line {}, column {}",
                            self.current().line,
                            self.current().column
                        ));
                    }
                    None
                };

                self.consume(TokenKind::Semicolon, "Expected ';' after case declaration")?;

                cases.push(EnumCase {
                    name: case_name,
                    value,
                });
            } else if self.check(&TokenKind::Public) || self.check(&TokenKind::Private) || self.check(&TokenKind::Protected) {
                // Parse method (enums can have methods)
                let visibility = match &self.current().kind {
                    TokenKind::Public => Visibility::Public,
                    TokenKind::Private => Visibility::Private,
                    TokenKind::Protected => Visibility::Protected,
                    _ => unreachable!(),
                };
                self.advance();

                self.consume(TokenKind::Function, "Expected 'function' after visibility modifier in enum")?;

                let method_name = if let TokenKind::Identifier(id) = &self.current().kind {
                    let name = id.clone();
                    self.advance();
                    name
                } else {
                    return Err(format!(
                        "Expected method name at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ));
                };

                self.consume(TokenKind::LeftParen, "Expected '(' after method name")?;

                let mut params = Vec::new();
                if !self.check(&TokenKind::RightParen) {
                    loop {
                        // Parse parameter
                        let param_name = if let TokenKind::Variable(var) = &self.current().kind {
                            let name = var.clone();
                            self.advance();
                            name
                        } else {
                            return Err(format!(
                                "Expected parameter name at line {}, column {}",
                                self.current().line,
                                self.current().column
                            ));
                        };

                        let default = if self.check(&TokenKind::Assign) {
                            self.advance();
                            Some(self.parse_expression(Precedence::Lowest)?)
                        } else {
                            None
                        };

                        params.push(FunctionParam {
                            name: param_name,
                            default,
                            by_ref: false,
                            visibility: None,
                            readonly: false,
                            attributes: Vec::new(),
                        });

                        if !self.check(&TokenKind::Comma) {
                            break;
                        }
                        self.advance(); // consume ','
                    }
                }

                self.consume(TokenKind::RightParen, "Expected ')' after parameters")?;
                self.consume(TokenKind::LeftBrace, "Expected '{' before method body")?;

                let body = self.parse_block()?;

                methods.push(Method {
                    name: method_name,
                    visibility,
                    params,
                    body,
                    attributes: Vec::new(),
                });
            } else {
                return Err(format!(
                    "Expected 'case' or method declaration in enum at line {}, column {}",
                    self.current().line,
                    self.current().column
                ));
            }
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after enum body")?;

        if cases.is_empty() {
            return Err(format!(
                "Enum '{}' must have at least one case",
                name
            ));
        }

        Ok(Stmt::Enum {
            name,
            backing_type,
            cases,
            methods,
            attributes,
        })
    }
```

**Note:** The `parse_block()` helper method should already exist for parsing statement blocks. If not, you can reuse the pattern from `parse_class()`.

### Step 6: Update Parser - Handle Enum Case Access (`src/parser/expr.rs`)

**Location:** In the `::` operator handling section (around line 430-480)

**Find the section that handles `TokenKind::DoubleColon` for static method calls:**

```rust
TokenKind::DoubleColon => {
    self.advance(); // consume '::'

    // Current code handles method calls...
}
```

**Modify to check for enum case access (identifier without parentheses) vs method call:**

The existing code should handle this automatically if it checks for method calls with `(`. Enum case access will be an identifier without `(`. Update the logic to return `Expr::EnumCase` when no `(` follows:

```rust
TokenKind::DoubleColon => {
    self.advance(); // consume '::'

    let method_or_case = if let TokenKind::Identifier(id) = &self.current().kind {
        let name = id.clone();
        self.advance();
        name
    } else {
        return Err(format!(
            "Expected method or case name after '::' at line {}, column {}",
            self.current().line,
            self.current().column
        ));
    };

    // Check if this is a method call (with parentheses) or case access
    if self.check(&TokenKind::LeftParen) {
        self.advance(); // consume '('
        let mut args = Vec::new();

        // Parse arguments (existing code)...

        expr = Expr::StaticMethodCall {
            class_name,
            method: method_or_case,
            args,
        };
    } else {
        // This is an enum case access (no parentheses)
        expr = Expr::EnumCase {
            enum_name: class_name,
            case_name: method_or_case,
        };
    }
}
```

**Important:** Make sure the class_name variable is available from parsing the left side of `::`. This pattern should already exist for static method calls.

### Step 7: Update Interpreter - Add Enum Storage (`src/interpreter/mod.rs`)

**Location 1:** After `TraitDefinition` struct (around line 77)

**Add:**

```rust
/// Enum definition stored in the interpreter
#[derive(Debug, Clone)]
pub struct EnumDefinition {
    pub name: String,
    pub backing_type: crate::ast::stmt::EnumBackingType,
    pub cases: Vec<(String, Option<Value>)>, // (case_name, optional_value)
    pub methods: HashMap<String, UserFunction>,
    pub method_visibility: HashMap<String, Visibility>,
    #[allow(dead_code)] // Will be used for reflection
    pub attributes: Vec<crate::ast::Attribute>,
}
```

**Location 2:** In the `Interpreter` struct (around line 88)

**Add field:**

```rust
    enums: HashMap<String, EnumDefinition>,
```

**Location 3:** In `Interpreter::new()` method (around line 102)

**Add initialization:**

```rust
            enums: HashMap::new(),
```

### Step 8: Update Interpreter - Handle Enum Statements (`src/interpreter/mod.rs`)

**Location:** In `execute_stmt()` method, find the match block with statement types (around line 1500-1800)

**Add after `Stmt::Trait { ... }` handling:**

```rust
            Stmt::Enum {
                name,
                backing_type,
                cases,
                methods,
                attributes,
            } => {
                // Validate cases
                let mut case_values: HashMap<String, Value> = HashMap::new();
                let mut case_list: Vec<(String, Option<Value>)> = Vec::new();

                for case in cases {
                    // Check for duplicate case names
                    if case_values.contains_key(&case.name) {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!("Duplicate case name '{}' in enum '{}'", case.name, name),
                        ));
                    }

                    // Evaluate case value for backed enums
                    let value = if let Some(ref value_expr) = case.value {
                        let val = self.eval_expr(value_expr).map_err(|e| {
                            io::Error::new(io::ErrorKind::Other, e)
                        })?;

                        // Validate backing type matches
                        match backing_type {
                            crate::ast::stmt::EnumBackingType::Int => {
                                if !matches!(val, Value::Integer(_)) {
                                    return Err(io::Error::new(
                                        io::ErrorKind::Other,
                                        format!(
                                            "Enum case '{}::{}' must have int backing value",
                                            name, case.name
                                        ),
                                    ));
                                }
                            }
                            crate::ast::stmt::EnumBackingType::String => {
                                if !matches!(val, Value::String(_)) {
                                    return Err(io::Error::new(
                                        io::ErrorKind::Other,
                                        format!(
                                            "Enum case '{}::{}' must have string backing value",
                                            name, case.name
                                        ),
                                    ));
                                }
                            }
                            crate::ast::stmt::EnumBackingType::None => {
                                return Err(io::Error::new(
                                    io::ErrorKind::Other,
                                    "Pure enum cannot have case values".to_string(),
                                ));
                            }
                        }

                        // Check for duplicate values
                        for (_, existing_val) in &case_list {
                            if let Some(existing) = existing_val {
                                if self.values_identical(existing, &val) {
                                    return Err(io::Error::new(
                                        io::ErrorKind::Other,
                                        format!(
                                            "Duplicate case value in backed enum '{}'",
                                            name
                                        ),
                                    ));
                                }
                            }
                        }

                        Some(val)
                    } else {
                        None
                    };

                    case_values.insert(case.name.clone(), value.clone().unwrap_or(Value::Null));
                    case_list.push((case.name.clone(), value));
                }

                // Store methods
                let mut method_map = HashMap::new();
                let mut visibility_map = HashMap::new();

                for method in methods {
                    let method_name_lower = method.name.to_lowercase();
                    method_map.insert(
                        method_name_lower.clone(),
                        UserFunction {
                            params: method.params.clone(),
                            body: method.body.clone(),
                            attributes: method.attributes.clone(),
                        },
                    );
                    visibility_map.insert(method_name_lower, method.visibility);
                }

                // Store enum definition
                let enum_def = EnumDefinition {
                    name: name.clone(),
                    backing_type: *backing_type,
                    cases: case_list,
                    methods: method_map,
                    method_visibility: visibility_map,
                    attributes: attributes.clone(),
                };

                self.enums.insert(name.to_lowercase(), enum_def);
                Ok(ControlFlow::None)
            }
```

### Step 9: Update Value Type (`src/interpreter/value.rs`)

**Location:** In the `Value` enum (around line 10-30)

**Add variant:**

```rust
    EnumCase {
        enum_name: String,
        case_name: String,
        backing_value: Option<Box<Value>>, // Some(value) for backed enums, None for pure
    },
```

**Location:** In `impl Value` block, update relevant methods:

**Update `get_type()` method (around line 50-80):**

Add match arm:

```rust
            Value::EnumCase { enum_name, case_name, .. } => {
                format!("{}::{}", enum_name, case_name)
            }
```

**Update `to_string()` method (around line 100-150):**

Add match arm:

```rust
            Value::EnumCase { enum_name, case_name, .. } => {
                format!("{}::{}", enum_name, case_name)
            }
```

**Update `to_bool()` method (around line 200-230):**

Add match arm:

```rust
            Value::EnumCase { .. } => true, // Enum cases are always truthy
```

### Step 10: Update Interpreter - Handle Enum Case Access (`src/interpreter/mod.rs`)

**Location:** In `eval_expr()` method, in the match block (around line 187, after `Expr::Match`)

**Add:**

```rust
            Expr::EnumCase { enum_name, case_name } => {
                self.eval_enum_case(enum_name, case_name)
            }
```

**Location:** Add new method in the impl block (around line 1300+)

**Add:**

```rust
    /// Evaluate enum case access (EnumName::CASE)
    fn eval_enum_case(&self, enum_name: &str, case_name: &str) -> Result<Value, String> {
        let enum_name_lower = enum_name.to_lowercase();

        // Look up enum definition
        let enum_def = self.enums.get(&enum_name_lower)
            .ok_or_else(|| format!("Undefined enum '{}'", enum_name))?;

        // Find the case
        for (name, value) in &enum_def.cases {
            if name == case_name {
                return Ok(Value::EnumCase {
                    enum_name: enum_def.name.clone(),
                    case_name: name.clone(),
                    backing_value: value.as_ref().map(|v| Box::new(v.clone())),
                });
            }
        }

        Err(format!("Undefined case '{}' for enum '{}'", case_name, enum_name))
    }
```

### Step 11: Update Interpreter - Handle Built-in Enum Methods (`src/interpreter/mod.rs`)

**Location:** In `eval_static_method_call()` method (around line 1250)

**Before the "Look up method in hierarchy" section, add enum method handling:**

```rust
        // Check if this is an enum (handle built-in enum methods)
        if let Some(enum_def) = self.enums.get(&target_class.to_lowercase()) {
            let method_lower = method.to_lowercase();

            return match method_lower.as_str() {
                "cases" => {
                    // Return array of all enum cases
                    if !args.is_empty() {
                        return Err("cases() takes no arguments".to_string());
                    }

                    let cases: Vec<(ArrayKey, Value)> = enum_def
                        .cases
                        .iter()
                        .enumerate()
                        .map(|(i, (name, value))| {
                            (
                                ArrayKey::Integer(i as i64),
                                Value::EnumCase {
                                    enum_name: enum_def.name.clone(),
                                    case_name: name.clone(),
                                    backing_value: value.as_ref().map(|v| Box::new(v.clone())),
                                },
                            )
                        })
                        .collect();

                    Ok(Value::Array(cases))
                }
                "from" => {
                    // Get case by backing value (throws on invalid)
                    if args.len() != 1 {
                        return Err("from() expects exactly 1 argument".to_string());
                    }

                    if enum_def.backing_type == crate::ast::stmt::EnumBackingType::None {
                        return Err(format!(
                            "Pure enum '{}' cannot use from() method",
                            enum_def.name
                        ));
                    }

                    let search_value = self.eval_expr(&args[0].value)?;

                    for (name, value) in &enum_def.cases {
                        if let Some(val) = value {
                            if self.values_identical(val, &search_value) {
                                return Ok(Value::EnumCase {
                                    enum_name: enum_def.name.clone(),
                                    case_name: name.clone(),
                                    backing_value: Some(Box::new(val.clone())),
                                });
                            }
                        }
                    }

                    Err(format!(
                        "Value '{}' is not a valid backing value for enum '{}'",
                        search_value.to_string(),
                        enum_def.name
                    ))
                }
                "tryfrom" => {
                    // Get case by backing value (returns null on invalid)
                    if args.len() != 1 {
                        return Err("tryFrom() expects exactly 1 argument".to_string());
                    }

                    if enum_def.backing_type == crate::ast::stmt::EnumBackingType::None {
                        return Err(format!(
                            "Pure enum '{}' cannot use tryFrom() method",
                            enum_def.name
                        ));
                    }

                    let search_value = self.eval_expr(&args[0].value)?;

                    for (name, value) in &enum_def.cases {
                        if let Some(val) = value {
                            if self.values_identical(val, &search_value) {
                                return Ok(Value::EnumCase {
                                    enum_name: enum_def.name.clone(),
                                    case_name: name.clone(),
                                    backing_value: Some(Box::new(val.clone())),
                                });
                            }
                        }
                    }

                    Ok(Value::Null)
                }
                _ => {
                    // Check for user-defined method
                    if let Some(func) = enum_def.methods.get(&method_lower) {
                        // Call enum method (enums don't have instance state)
                        self.call_user_function_with_arguments(func, args)
                    } else {
                        Err(format!(
                            "Call to undefined method {}::{}()",
                            enum_def.name, method
                        ))
                    }
                }
            };
        }
```

### Step 12: Update Interpreter - Handle Enum Case Property Access (`src/interpreter/mod.rs`)

**Location:** In `eval_property_access()` method (around line 1050)

**Add handling for enum case properties (`name` and `value`):**

At the beginning of the method, before object handling:

```rust
    fn eval_property_access(&mut self, object: &Expr, property: &str) -> Result<Value, String> {
        let obj_value = self.eval_expr(object)?;

        // Handle enum case properties
        if let Value::EnumCase {
            enum_name,
            case_name,
            backing_value,
        } = obj_value
        {
            match property {
                "name" => return Ok(Value::String(case_name)),
                "value" => {
                    if let Some(val) = backing_value {
                        return Ok(*val);
                    } else {
                        return Err(format!(
                            "Pure enum case {}::{} does not have a 'value' property",
                            enum_name, case_name
                        ));
                    }
                }
                _ => {
                    return Err(format!(
                        "Enum case {}::{} does not have property '{}'",
                        enum_name, case_name, property
                    ));
                }
            }
        }

        // Existing object property access code...
```

### Step 13: Update Interpreter - Add Helper Method (`src/interpreter/mod.rs`)

**Location:** Add at the end of the impl block (around line 1800+)

**Add:**

```rust
    /// Check if two values are identical (===)
    fn values_identical(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            _ => false,
        }
    }
```

### Step 14: Add Tests (`tests/enums/`)

Create a new directory and add comprehensive test files:

#### Test 1: `tests/enums/pure_enum_basic.vhpt`

```php
--TEST--
Pure enum basic declaration and case access
--FILE--
<?php

enum Status {
    case Pending;
    case Active;
    case Archived;
}

$status = Status::Active;
echo $status->name;

--EXPECT--
Active
```

#### Test 2: `tests/enums/backed_enum_int.vhpt`

```php
--TEST--
Backed enum with int values
--FILE--
<?php

enum Priority: int {
    case Low = 1;
    case Medium = 5;
    case High = 10;
}

$p = Priority::High;
echo $p->name . "\n";
echo $p->value;

--EXPECT--
High
10
```

#### Test 3: `tests/enums/backed_enum_string.vhpt`

```php
--TEST--
Backed enum with string values
--FILE--
<?php

enum Color: string {
    case Red = 'red';
    case Green = 'green';
    case Blue = 'blue';
}

$c = Color::Red;
echo $c->name . "\n";
echo $c->value;

--EXPECT--
Red
red
```

#### Test 4: `tests/enums/enum_cases_method.vhpt`

```php
--TEST--
Enum cases() method returns all cases
--FILE--
<?php

enum Status {
    case Pending;
    case Active;
    case Archived;
}

$cases = Status::cases();
echo count($cases) . "\n";
echo $cases[0]->name . "\n";
echo $cases[1]->name . "\n";
echo $cases[2]->name;

--EXPECT--
3
Pending
Active
Archived
```

#### Test 5: `tests/enums/enum_from_method.vhpt`

```php
--TEST--
Enum from() method retrieves case by value
--FILE--
<?php

enum Priority: int {
    case Low = 1;
    case Medium = 5;
    case High = 10;
}

$p = Priority::from(5);
echo $p->name;

--EXPECT--
Medium
```

#### Test 6: `tests/enums/enum_from_invalid.vhpt`

```php
--TEST--
Enum from() method throws error on invalid value
--FILE--
<?php

enum Priority: int {
    case Low = 1;
    case Medium = 5;
    case High = 10;
}

$p = Priority::from(99);

--EXPECT_ERROR--
Value '99' is not a valid backing value
```

#### Test 7: `tests/enums/enum_tryfrom_method.vhpt`

```php
--TEST--
Enum tryFrom() method returns null on invalid value
--FILE--
<?php

enum Priority: int {
    case Low = 1;
    case Medium = 5;
    case High = 10;
}

$p1 = Priority::tryFrom(5);
echo $p1->name . "\n";

$p2 = Priority::tryFrom(99);
var_dump($p2);

--EXPECT--
Medium
NULL
```

#### Test 8: `tests/enums/enum_comparison_identical.vhpt`

```php
--TEST--
Enum case comparison with identical operator
--FILE--
<?php

enum Status {
    case Pending;
    case Active;
}

$s1 = Status::Active;
$s2 = Status::Active;
$s3 = Status::Pending;

var_dump($s1 === $s2);
var_dump($s1 === $s3);

--EXPECT--
bool(true)
bool(false)
```

#### Test 9: `tests/enums/enum_in_array.vhpt`

```php
--TEST--
Enum cases can be stored in arrays
--FILE--
<?php

enum Status {
    case Pending;
    case Active;
    case Archived;
}

$statuses = [Status::Pending, Status::Active];
echo $statuses[0]->name . "\n";
echo $statuses[1]->name;

--EXPECT--
Pending
Active
```

#### Test 10: `tests/enums/pure_enum_no_value_error.vhpt`

```php
--TEST--
Pure enum cases cannot access value property
--FILE--
<?php

enum Status {
    case Pending;
}

$s = Status::Pending;
echo $s->value;

--EXPECT_ERROR--
Pure enum case Status::Pending does not have a 'value' property
```

#### Test 11: `tests/enums/backed_enum_duplicate_value_error.vhpt`

```php
--TEST--
Backed enum cannot have duplicate values
--FILE--
<?php

enum Priority: int {
    case Low = 1;
    case Medium = 1;
}

--EXPECT_ERROR--
Duplicate case value in backed enum
```

#### Test 12: `tests/enums/enum_undefined_case_error.vhpt`

```php
--TEST--
Accessing undefined enum case throws error
--FILE--
<?php

enum Status {
    case Pending;
}

$s = Status::InvalidCase;

--EXPECT_ERROR--
Undefined case 'InvalidCase' for enum 'Status'
```

#### Test 13: `tests/enums/backed_enum_wrong_type_error.vhpt`

```php
--TEST--
Backed enum must have correct backing type
--FILE--
<?php

enum Priority: int {
    case Low = "low";
}

--EXPECT_ERROR--
Enum case 'Priority::Low' must have int backing value
```

#### Test 14: `tests/enums/pure_enum_with_value_error.vhpt`

```php
--TEST--
Pure enum cannot have case values
--FILE--
<?php

enum Status {
    case Pending = 1;
}

--EXPECT_ERROR--
Pure enum cannot have case values
```

#### Test 15: `tests/enums/backed_enum_without_value_error.vhpt`

```php
--TEST--
Backed enum must have case values
--FILE--
<?php

enum Priority: int {
    case Low;
}

--EXPECT_ERROR--
Backed enum must have case values
```

#### Test 16: `tests/enums/enum_switch_case.vhpt`

```php
--TEST--
Enum cases in switch statement
--FILE--
<?php

enum Status {
    case Pending;
    case Active;
    case Archived;
}

$status = Status::Active;

switch ($status->name) {
    case "Pending":
        echo "Waiting";
        break;
    case "Active":
        echo "Running";
        break;
    case "Archived":
        echo "Done";
        break;
}

--EXPECT--
Running
```

### Step 15: Update Documentation

#### Update `AGENTS.md`

**Location:** Around line 419, in the Phase 6 section

**Change:**

```markdown
- [ ] Enums (PHP 8.1)
```

**To:**

```markdown
- [x] Enums (PHP 8.1)
```

**Also update the Current Features section (around line 200):**

Add after attributes:

```markdown
### Enums (PHP 8.1)
- [x] Pure enums (cases without values)
- [x] Backed enums (int and string)
- [x] Enum case access (`EnumName::CASE`)
- [x] Case properties (`->name`, `->value`)
- [x] Built-in methods (`cases()`, `from()`, `tryFrom()`)
- [x] Case-sensitive case names
- [x] Enum methods (with visibility modifiers)
```

#### Update `README.md`

Add enums to the features list and update the roadmap table to mark enums as complete.

#### Update `docs/roadmap.md`

Mark enums as complete in the roadmap.

## Key Considerations

### PHP Compatibility

1. **Case Sensitivity**
   - Enum **names** are case-insensitive (like classes): `Status` === `status`
   - Case **names** are case-sensitive: `Status::Active` !== `Status::ACTIVE`

2. **Backing Type Validation**
   - Pure enums: No backing type, no values allowed
   - Backed enums: Must specify `int` or `string`, all cases must have values
   - Backing values must be constant expressions (no variables)
   - Duplicate backing values are not allowed

3. **Built-in Methods**
   - `cases()`: Returns array of all enum cases (works for all enums)
   - `from($value)`: Returns case by backing value, throws error if not found (backed enums only)
   - `tryFrom($value)`: Returns case by backing value or null (backed enums only)

4. **Properties**
   - All cases have `->name` property (string)
   - Backed enum cases have `->value` property (int or string)
   - Pure enum cases do NOT have `->value` property

5. **Comparison**
   - Enum cases are compared by identity: `Status::Active === Status::Active` is `true`
   - Different cases are never equal: `Status::Active === Status::Pending` is `false`
   - Implementation: Compare enum_name and case_name for equality

### Edge Cases to Handle

1. **Empty Enums**: Enums must have at least one case (parser should error)
2. **Duplicate Case Names**: Error during parsing/storage
3. **Duplicate Backing Values**: Error during enum registration
4. **Wrong Backing Type**: String value for int-backed enum (error)
5. **Pure Enum with Values**: Error during parsing
6. **Backed Enum without Values**: Error during parsing
7. **from() on Invalid Value**: Throw error with descriptive message
8. **tryFrom() on Invalid Value**: Return null (no error)
9. **from()/tryFrom() on Pure Enum**: Error - method not available
10. **Accessing ->value on Pure Enum**: Error - property doesn't exist

### Interaction with Existing Features

1. **Type Coercion**: Enum cases don't coerce to other types automatically
2. **String Context**: When converted to string, should display "EnumName::CaseName"
3. **Boolean Context**: Enum cases are always truthy
4. **Arrays**: Enum cases can be array keys and values
5. **Switch Statements**: Use `$enum->name` for switch comparison
6. **Variables**: Enum cases can be assigned to variables
7. **Function Parameters**: Enum cases can be passed to functions
8. **Class Properties**: Enum cases can be stored in object properties

### Error Message Requirements

- "Undefined enum 'EnumName'"
- "Undefined case 'CaseName' for enum 'EnumName'"
- "Enum 'Name' must have at least one case"
- "Duplicate case name 'CaseName' in enum 'EnumName'"
- "Duplicate case value in backed enum 'EnumName'"
- "Enum case 'EnumName::CaseName' must have int backing value"
- "Enum case 'EnumName::CaseName' must have string backing value"
- "Pure enum cannot have case values"
- "Backed enum must have case values"
- "Pure enum 'EnumName' cannot use from() method"
- "Pure enum 'EnumName' cannot use tryFrom() method"
- "Value 'X' is not a valid backing value for enum 'EnumName'"
- "Pure enum case EnumName::CaseName does not have a 'value' property"
- "Enum case EnumName::CaseName does not have property 'PropName'"
- "Invalid enum backing type 'TypeName'. Only 'int' and 'string' are supported"

## Test Cases

Test suite should cover:

### Pure Enums
- ✅ Basic declaration and case access
- ✅ Multiple cases
- ✅ Case name property access
- ✅ Error when accessing value property
- ✅ Error when declaring with values

### Backed Enums (int)
- ✅ Declaration with int values
- ✅ Case name and value properties
- ✅ from() method success
- ✅ from() method failure
- ✅ tryFrom() success and failure
- ✅ Error when wrong type (string instead of int)
- ✅ Error when declaring without values
- ✅ Error on duplicate values

### Backed Enums (string)
- ✅ Declaration with string values
- ✅ Case name and value properties
- ✅ from() and tryFrom() methods

### Built-in Methods
- ✅ cases() returns all cases as array
- ✅ from() throws on invalid value
- ✅ tryFrom() returns null on invalid value
- ✅ Error when using from()/tryFrom() on pure enums

### Integration
- ✅ Enum cases in arrays
- ✅ Enum cases in variables
- ✅ Enum cases in switch statements (using ->name)
- ✅ Comparison with === operator
- ✅ Undefined enum error
- ✅ Undefined case error

### Validation
- ✅ Empty enum error
- ✅ Duplicate case names
- ✅ Duplicate backing values
- ✅ Invalid backing type

## Reference Implementation

### Similar Patterns in Existing Code

1. **Class/Interface/Trait Storage**: Look at how `ClassDefinition`, `InterfaceDefinition`, and `TraitDefinition` are structured in `src/interpreter/mod.rs` (lines 36-77)

2. **Static Method Calls**: Review `eval_static_method_call()` in `src/interpreter/mod.rs` (line 1214) for pattern on handling `::` operator

3. **Property Access**: Review `eval_property_access()` in `src/interpreter/mod.rs` (line 1038) for accessing object properties

4. **Case-Insensitive Lookups**: See how class names are lowercased for storage and lookup throughout the interpreter

5. **Visibility Modifiers**: See how `Method` struct handles visibility in `src/ast/stmt.rs` (line 41)

6. **Token and Lexer Updates**: Reference how `readonly` keyword was added (token.rs line 43, lexer.rs keyword matching)

### PHP 8.1 Official Docs

- PHP RFC: https://wiki.php.net/rfc/enumerations
- PHP Manual: https://www.php.net/manual/en/language.enumerations.php

## Implementation Priority

**Phase 1: Core Enum Support** (MVP)
1. Pure enums with case access
2. Backed enums (int and string)
3. `->name` and `->value` properties
4. `cases()` method

**Phase 2: Advanced Features**
5. `from()` and `tryFrom()` methods
6. Enum comparison (===)
7. Comprehensive error handling

**Phase 3: Optional Features** (Future)
8. Enum methods
9. Interface implementation
10. Serialization support

**Recommended Approach**: Implement Phase 1 and Phase 2 fully. Phase 3 can be added later as needed.

## Notes

- VHP uses only Rust std library - no external dependencies
- Follow existing patterns for case-insensitive name lookups
- Enum cases should be immutable once created
- The `case` keyword already exists for switch-case, so it will be reused for enum cases
- Enums don't support inheritance or static properties (per PHP spec)
- For v1 implementation, focus on pure enums + backed enums + built-in methods
- Enum methods can be added in a later iteration if needed
