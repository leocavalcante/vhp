# Plan: Attributes (PHP 8.0)

## Overview

Attributes (also known as annotations in other languages) were introduced in PHP 8.0 as a structured, native way to add metadata to classes, functions, methods, properties, parameters, and constants. They replace the older docblock-based annotations with a first-class language feature.

Attributes can be used for dependency injection, routing, validation, serialization control, deprecation warnings, and more. They are stored in the AST during compilation and can be retrieved at runtime using Reflection (which VHP doesn't support yet), but they need to be parsed and stored correctly.

**PHP Example:**

```php
// Before (PHP 7.x - docblock comments)
/**
 * @Route("/api/users")
 * @Middleware("auth")
 */
class UserController {
    /**
     * @Validate(max: 100)
     */
    public function create($data) {}
}

// After (PHP 8.0 - native attributes)
#[Route("/api/users")]
#[Middleware("auth")]
class UserController {
    public function create(
        #[Validate(max: 100)]
        #[FromBody]
        $data
    ) {}
}
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/token.rs` | Add `Hash`, `LeftBracket`, `RightBracket` token (already exists) |
| `src/lexer.rs` | Recognize `#[` as attribute start (hash already tokenized) |
| `src/ast/stmt.rs` | Add `Attribute` struct and attribute fields to all statement types |
| `src/ast/expr.rs` | No changes needed (attributes don't appear in expressions) |
| `src/parser/stmt.rs` | Parse `#[...]` syntax and attach to declarations |
| `src/interpreter/mod.rs` | Store attributes in class/function/method definitions (for future reflection) |
| `tests/attributes/*.vhpt` | Create comprehensive test suite |
| `AGENTS.md` | Mark feature as complete |
| `README.md` | Update roadmap and features list |

## Implementation Steps

### Step 1: Add Tokens (`src/token.rs`)

**Status:** Most tokens already exist. Need to verify and potentially add.

**Location:** Check around line 95-110

**Tokens needed:**
- `Hash` (for `#`) - Check if exists or needs to be added
- `LeftBracket` (for `[`) - Already exists (line 102)
- `RightBracket` (for `]`) - Already exists (line 103)

**If `Hash` doesn't exist, add after line 108 (after `Arrow`):**

```rust
    Hash,              // #
```

**Note:** The lexer may already tokenize `#` as the start of a comment. We need to handle the special case `#[` differently from `#` (comment).

### Step 2: Update Lexer (`src/lexer.rs`)

**Location:** Around line 250-450 (in the main tokenization logic)

**Current behavior:** `#` is likely treated as the start of a single-line comment.

**Required change:** Make the lexer recognize `#[` as the start of an attribute, not a comment.

**Find the section that handles `#` character (search for `'#'` in lexer.rs):**

**Replace the current comment handling with:**

```rust
'#' => {
    // Check if this is an attribute (#[) or a comment (#)
    if self.peek(1) == Some('[') {
        // This is an attribute start
        self.advance(); // consume '#'
        tokens.push(Token::new(TokenKind::Hash, line, column));
        // The '[' will be handled in the next iteration
    } else {
        // This is a single-line comment
        while let Some(ch) = self.current() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
        continue;
    }
}
```

**Important:** After adding this, verify that `[` is still tokenized as `LeftBracket`.

### Step 3: Extend AST - Create Attribute Struct (`src/ast/stmt.rs`)

**Location:** After the `Visibility` enum (around line 10)

**Add the `Attribute` and `AttributeArgument` structures:**

```rust
/// Attribute argument (can be positional or named)
#[derive(Debug, Clone)]
pub struct AttributeArgument {
    pub name: Option<String>, // None for positional, Some("name") for named
    pub value: Expr,
}

/// Attribute metadata (PHP 8.0)
#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub arguments: Vec<AttributeArgument>,
}
```

**Explanation:**
- `AttributeArgument` represents each argument passed to an attribute (like `Route("/api")` has one positional argument)
- `Attribute` represents a single attribute with its name and arguments
- Multiple attributes can be attached to a single declaration

### Step 4: Extend AST - Add Attribute Fields (`src/ast/stmt.rs`)

**Location:** Various struct definitions throughout the file

**Changes needed:**

#### 4a. Add to `Property` struct (around line 11-19)

**Current code:**
```rust
#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    #[allow(dead_code)] // Will be used for visibility enforcement
    pub visibility: Visibility,
    pub default: Option<Expr>,
    pub readonly: bool, // PHP 8.1+
}
```

**Updated code:**
```rust
#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    #[allow(dead_code)] // Will be used for visibility enforcement
    pub visibility: Visibility,
    pub default: Option<Expr>,
    pub readonly: bool, // PHP 8.1+
    pub attributes: Vec<Attribute>, // PHP 8.0+
}
```

#### 4b. Add to `Method` struct (around line 21-27)

**Current code:**
```rust
#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub visibility: Visibility,
    pub params: Vec<FunctionParam>,
    pub body: Vec<Stmt>,
}
```

**Updated code:**
```rust
#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub visibility: Visibility,
    pub params: Vec<FunctionParam>,
    pub body: Vec<Stmt>,
    pub attributes: Vec<Attribute>, // PHP 8.0+
}
```

#### 4c. Add to `InterfaceMethodSignature` struct (around line 30-34)

**Current code:**
```rust
#[derive(Debug, Clone)]
pub struct InterfaceMethodSignature {
    pub name: String,
    pub params: Vec<FunctionParam>,
}
```

**Updated code:**
```rust
#[derive(Debug, Clone)]
pub struct InterfaceMethodSignature {
    pub name: String,
    pub params: Vec<FunctionParam>,
    pub attributes: Vec<Attribute>, // PHP 8.0+
}
```

#### 4d. Add to `InterfaceConstant` struct (around line 37-41)

**Current code:**
```rust
#[derive(Debug, Clone)]
pub struct InterfaceConstant {
    pub name: String,
    pub value: Expr,
}
```

**Updated code:**
```rust
#[derive(Debug, Clone)]
pub struct InterfaceConstant {
    pub name: String,
    pub value: Expr,
    pub attributes: Vec<Attribute>, // PHP 8.0+
}
```

#### 4e. Add to `FunctionParam` struct (around line 147-158)

**Current code:**
```rust
#[derive(Debug, Clone)]
pub struct FunctionParam {
    pub name: String,
    pub default: Option<Expr>,
    /// By-reference parameter (will be used when reference semantics are implemented)
    #[allow(dead_code)]
    pub by_ref: bool,
    /// Visibility for constructor property promotion (PHP 8.0)
    pub visibility: Option<Visibility>,
    /// Readonly modifier for constructor property promotion (PHP 8.1)
    pub readonly: bool,
}
```

**Updated code:**
```rust
#[derive(Debug, Clone)]
pub struct FunctionParam {
    pub name: String,
    pub default: Option<Expr>,
    /// By-reference parameter (will be used when reference semantics are implemented)
    #[allow(dead_code)]
    pub by_ref: bool,
    /// Visibility for constructor property promotion (PHP 8.0)
    pub visibility: Option<Visibility>,
    /// Readonly modifier for constructor property promotion (PHP 8.1)
    pub readonly: bool,
    /// Attributes for parameters (PHP 8.0)
    pub attributes: Vec<Attribute>,
}
```

#### 4f. Add to `Stmt` enum variants

**Location:** Around lines 69-137

**Update these statement variants:**

**`Stmt::Function` (around line 110-114):**

**Current:**
```rust
    Function {
        name: String,
        params: Vec<FunctionParam>,
        body: Vec<Stmt>,
    },
```

**Updated:**
```rust
    Function {
        name: String,
        params: Vec<FunctionParam>,
        body: Vec<Stmt>,
        attributes: Vec<Attribute>, // PHP 8.0+
    },
```

**`Stmt::Interface` (around line 116-121):**

**Current:**
```rust
    Interface {
        name: String,
        parents: Vec<String>,
        methods: Vec<InterfaceMethodSignature>,
        constants: Vec<InterfaceConstant>,
    },
```

**Updated:**
```rust
    Interface {
        name: String,
        parents: Vec<String>,
        methods: Vec<InterfaceMethodSignature>,
        constants: Vec<InterfaceConstant>,
        attributes: Vec<Attribute>, // PHP 8.0+
    },
```

**`Stmt::Trait` (around line 122-127):**

**Current:**
```rust
    Trait {
        name: String,
        uses: Vec<String>,
        properties: Vec<Property>,
        methods: Vec<Method>,
    },
```

**Updated:**
```rust
    Trait {
        name: String,
        uses: Vec<String>,
        properties: Vec<Property>,
        methods: Vec<Method>,
        attributes: Vec<Attribute>, // PHP 8.0+
    },
```

**`Stmt::Class` (around line 128-136):**

**Current:**
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

**Updated:**
```rust
    Class {
        name: String,
        readonly: bool, // PHP 8.2+: all properties are implicitly readonly
        parent: Option<String>,
        interfaces: Vec<String>,
        trait_uses: Vec<TraitUse>,
        properties: Vec<Property>,
        methods: Vec<Method>,
        attributes: Vec<Attribute>, // PHP 8.0+
    },
```

### Step 5: Update Parser - Attribute Parsing Function (`src/parser/stmt.rs`)

**Location:** After the helper methods (around line 50-56, before `parse_echo`)

**Add a new method to parse attributes:**

```rust
/// Parse attributes: #[AttributeName(args)] or #[AttributeName]
/// Can have multiple attributes: #[Attr1] #[Attr2(arg)]
fn parse_attributes(&mut self) -> Result<Vec<Attribute>, String> {
    use crate::ast::Attribute;
    use crate::ast::AttributeArgument;

    let mut attributes = Vec::new();

    // Keep parsing while we see #[
    while self.check(&TokenKind::Hash) {
        // Check if next token is [
        let current_pos = *self.pos;
        self.advance(); // consume '#'

        if !self.check(&TokenKind::LeftBracket) {
            // Not an attribute, restore position
            *self.pos = current_pos;
            break;
        }

        self.advance(); // consume '['

        // Parse comma-separated list of attributes within the same #[...]
        loop {
            // Parse attribute name (identifier)
            let name = if let TokenKind::Identifier(name) = &self.current().kind {
                let name = name.clone();
                self.advance();
                name
            } else {
                return Err(format!(
                    "Expected attribute name at line {}, column {}",
                    self.current().line,
                    self.current().column
                ));
            };

            // Parse optional arguments
            let mut arguments = Vec::new();
            if self.check(&TokenKind::LeftParen) {
                self.advance(); // consume '('

                if !self.check(&TokenKind::RightParen) {
                    loop {
                        // Check for named argument (name: value)
                        let mut arg_name = None;
                        if let TokenKind::Identifier(id) = &self.current().kind {
                            // Look ahead for colon
                            let lookahead_pos = *self.pos + 1;
                            if lookahead_pos < self.tokens.len() {
                                if let TokenKind::Colon = self.tokens[lookahead_pos].kind {
                                    // This is a named argument
                                    arg_name = Some(id.clone());
                                    self.advance(); // consume identifier
                                    self.advance(); // consume ':'
                                }
                            }
                        }

                        // Parse argument value
                        let value = self.parse_expression(Precedence::None)?;

                        arguments.push(AttributeArgument {
                            name: arg_name,
                            value,
                        });

                        if !self.check(&TokenKind::Comma) {
                            break;
                        }
                        self.advance(); // consume ','
                    }
                }

                self.consume(TokenKind::RightParen, "Expected ')' after attribute arguments")?;
            }

            attributes.push(Attribute {
                name,
                arguments,
            });

            // Check for comma (multiple attributes in same #[...])
            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance(); // consume ','
        }

        self.consume(TokenKind::RightBracket, "Expected ']' after attribute")?;
    }

    Ok(attributes)
}
```

**Important notes:**
- This function can parse multiple attributes: `#[A] #[B]` or `#[A, B]`
- Supports both positional and named arguments: `#[Route("/api")]` or `#[Route(path: "/api")]`
- Returns empty Vec if no attributes found (not an error)

### Step 6: Update Parser - Integrate Attribute Parsing

Now we need to call `parse_attributes()` before parsing class, function, method, property, and parameter declarations.

#### 6a. Parse attributes before functions (`src/parser/stmt.rs`)

**Location:** Around line 450-456 (in `parse_function`)

**Before:**
```rust
    pub fn parse_function(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'function'

        let name = if let TokenKind::Identifier(name) = &self.current().kind {
```

**After:**
```rust
    pub fn parse_function(&mut self) -> Result<Stmt, String> {
        // Note: attributes should be parsed by the caller (parse_statement)
        // before we get here, but we keep this flexible
        self.advance(); // consume 'function'

        let name = if let TokenKind::Identifier(name) = &self.current().kind {
```

**At the end of parse_function (around line 545):**

**Before:**
```rust
        Ok(Stmt::Function { name, params, body })
```

**After:**
```rust
        Ok(Stmt::Function {
            name,
            params,
            body,
            attributes: Vec::new(), // Will be set by parse_statement
        })
```

**Note:** We'll modify `parse_statement` to parse attributes first, then pass them to the statement creation.

#### 6b. Parse attributes before classes (`src/parser/stmt.rs`)

**Location:** Around line 1135 (in `parse_class`)

**At the end of parse_class (around line 1280-1289):**

**Before:**
```rust
        Ok(Stmt::Class {
            name,
            readonly,
            parent,
            interfaces,
            trait_uses,
            properties,
            methods,
        })
```

**After:**
```rust
        Ok(Stmt::Class {
            name,
            readonly,
            parent,
            interfaces,
            trait_uses,
            properties,
            methods,
            attributes: Vec::new(), // Will be set by parse_statement
        })
```

#### 6c. Parse attributes before properties (`src/parser/stmt.rs`)

**Location:** In `parse_class` body where properties are parsed (around line 1210-1260)

**Find the section that parses properties (around line 1249-1253):**

**Before:**
```rust
            } else if self.check(&TokenKind::Variable(String::new())) {
                // Parse property with readonly modifier
                let mut prop = self.parse_property(visibility)?;
                prop.readonly = readonly;
                properties.push(prop);
```

**After:**
```rust
            } else if self.check(&TokenKind::Variable(String::new())) {
                // Parse property with readonly modifier
                let mut prop = self.parse_property(visibility)?;
                prop.readonly = readonly;
                // Note: attributes for properties would be parsed before visibility
                // in the outer loop, but we'll handle it in the class parsing loop
                properties.push(prop);
```

**Now modify the class parsing loop to handle attributes:**

**Location:** Around line 1210 (start of property/method loop in parse_class)

**Before:**
```rust
        while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
            // Check for readonly modifier (can appear before visibility)
            let readonly_first = if self.check(&TokenKind::Readonly) {
```

**After:**
```rust
        while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
            // Parse attributes that may precede property or method
            let attributes = self.parse_attributes()?;

            // Check for readonly modifier (can appear before visibility)
            let readonly_first = if self.check(&TokenKind::Readonly) {
```

**Then when creating properties and methods, pass attributes:**

**For properties (around line 1249-1253):**

**Before:**
```rust
                let mut prop = self.parse_property(visibility)?;
                prop.readonly = readonly;
                properties.push(prop);
```

**After:**
```rust
                let mut prop = self.parse_property(visibility)?;
                prop.readonly = readonly;
                prop.attributes = attributes;
                properties.push(prop);
```

**For methods (around line 1248):**

**Before:**
```rust
            if self.check(&TokenKind::Function) {
                methods.push(self.parse_method(visibility)?);
```

**After:**
```rust
            if self.check(&TokenKind::Function) {
                let mut method = self.parse_method(visibility)?;
                method.attributes = attributes;
                methods.push(method);
```

#### 6d. Update parse_property to initialize attributes

**Location:** Find `parse_property` function (around line 570-599)

**At the end, when returning Property:**

**Before:**
```rust
    Ok(Property {
        name,
        visibility,
        default,
        readonly,
    })
```

**After:**
```rust
    Ok(Property {
        name,
        visibility,
        default,
        readonly,
        attributes: Vec::new(), // Will be set by caller
    })
```

#### 6e. Update parse_method to initialize attributes

**Location:** Find `parse_method` function (search for "fn parse_method")

**At the end, when returning Method:**

**Before:**
```rust
    Ok(Method {
        name,
        visibility,
        params,
        body,
    })
```

**After:**
```rust
    Ok(Method {
        name,
        visibility,
        params,
        body,
        attributes: Vec::new(), // Will be set by caller
    })
```

#### 6f. Parse attributes before parameters

**Location:** In function/method parameter parsing

Parameters can have attributes too: `function foo(#[Sensitive] $password) {}`

**Find where parameters are parsed** (in `parse_function` around line 468-530 or in a separate `parse_param` function)

**Before each parameter, parse attributes:**

**Example pattern (around line 470-525):**

```rust
        if !self.check(&TokenKind::RightParen) {
            loop {
                // Parse attributes for this parameter
                let param_attributes = self.parse_attributes()?;

                // Skip type hints (not supported yet)
                if let TokenKind::Identifier(type_name) = &self.current().kind {
                    // ... existing type hint skipping code ...
                }

                // ... rest of parameter parsing ...

                params.push(FunctionParam {
                    name: param_name,
                    default,
                    by_ref,
                    visibility: None,
                    readonly: false,
                    attributes: param_attributes,
                });
```

#### 6g. Update parse_statement to parse attributes

**Location:** Around line 1308-1340 (in `parse_statement`)

**Before the main match statement:**

**Before:**
```rust
    pub fn parse_statement(&mut self) -> Result<Option<Stmt>, String> {
        let token = self.current().clone();
        match token.kind {
```

**After:**
```rust
    pub fn parse_statement(&mut self) -> Result<Option<Stmt>, String> {
        // Parse any attributes that may precede declarations
        let attributes = self.parse_attributes()?;

        let token = self.current().clone();
        match token.kind {
```

**Then for Function and Class statements, attach the attributes:**

**For Function (around line 1328):**

**Before:**
```rust
            TokenKind::Function => Ok(Some(self.parse_function()?)),
```

**After:**
```rust
            TokenKind::Function => {
                let mut func = self.parse_function()?;
                if let Stmt::Function { attributes: ref mut attrs, .. } = func {
                    *attrs = attributes;
                }
                Ok(Some(func))
            }
```

**For Class (around line 1329-1333):**

**Before:**
```rust
            TokenKind::Class => Ok(Some(self.parse_class()?)),
            TokenKind::Readonly => {
                // readonly can be used before class keyword (PHP 8.2)
                // Just let parse_class handle it since it looks for readonly before the class keyword
                Ok(Some(self.parse_class()?))
            }
```

**After:**
```rust
            TokenKind::Class => {
                let mut class = self.parse_class()?;
                if let Stmt::Class { attributes: ref mut attrs, .. } = class {
                    *attrs = attributes;
                }
                Ok(Some(class))
            }
            TokenKind::Readonly => {
                // readonly can be used before class keyword (PHP 8.2)
                let mut class = self.parse_class()?;
                if let Stmt::Class { attributes: ref mut attrs, .. } = class {
                    *attrs = attributes;
                }
                Ok(Some(class))
            }
```

**Similarly for Interface and Trait:**

**For Interface (around line 1335):**

```rust
            TokenKind::Interface => {
                let mut iface = self.parse_interface()?;
                if let Stmt::Interface { attributes: ref mut attrs, .. } = iface {
                    *attrs = attributes;
                }
                Ok(Some(iface))
            }
```

**For Trait (around line 1336):**

```rust
            TokenKind::Trait => {
                let mut trait_stmt = self.parse_trait()?;
                if let Stmt::Trait { attributes: ref mut attrs, .. } = trait_stmt {
                    *attrs = attributes;
                }
                Ok(Some(trait_stmt))
            }
```

### Step 7: Update Interpreter (`src/interpreter/mod.rs`)

The interpreter needs to store attributes in class, function, and method definitions for potential future use (like reflection or runtime inspection).

#### 7a. Update ClassDefinition

**Location:** Around line 35-45

**Before:**
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
    #[allow(dead_code)] // Will be used for reflection
    pub attributes: Vec<crate::ast::Attribute>,
}
```

#### 7b. Update UserFunction

**Location:** Around line 28-32

**Before:**
```rust
pub struct UserFunction {
    pub params: Vec<FunctionParam>,
    pub body: Vec<Stmt>,
}
```

**After:**
```rust
pub struct UserFunction {
    pub params: Vec<FunctionParam>,
    pub body: Vec<Stmt>,
    #[allow(dead_code)] // Will be used for reflection
    pub attributes: Vec<crate::ast::Attribute>,
}
```

#### 7c. Update InterfaceDefinition

**Location:** Around line 47-57

**Before:**
```rust
pub struct InterfaceDefinition {
    #[allow(dead_code)] // Will be used for interface validation
    pub name: String,
    #[allow(dead_code)] // Will be used for interface inheritance
    pub parents: Vec<String>,
    pub methods: Vec<(String, Vec<FunctionParam>)>, // (name, params)
    #[allow(dead_code)] // Will be used for interface constants
    pub constants: HashMap<String, Value>,
}
```

**After:**
```rust
pub struct InterfaceDefinition {
    #[allow(dead_code)] // Will be used for interface validation
    pub name: String,
    #[allow(dead_code)] // Will be used for interface inheritance
    pub parents: Vec<String>,
    pub methods: Vec<(String, Vec<FunctionParam>)>, // (name, params)
    #[allow(dead_code)] // Will be used for interface constants
    pub constants: HashMap<String, Value>,
    #[allow(dead_code)] // Will be used for reflection
    pub attributes: Vec<crate::ast::Attribute>,
}
```

#### 7d. Update TraitDefinition

**Location:** Around line 59-69

**Before:**
```rust
pub struct TraitDefinition {
    #[allow(dead_code)] // Will be used for trait validation
    pub name: String,
    #[allow(dead_code)] // Will be used for trait composition
    pub uses: Vec<String>,
    pub properties: Vec<Property>,
    pub methods: HashMap<String, UserFunction>,
    pub method_visibility: HashMap<String, Visibility>,
}
```

**After:**
```rust
pub struct TraitDefinition {
    #[allow(dead_code)] // Will be used for trait validation
    pub name: String,
    #[allow(dead_code)] // Will be used for trait composition
    pub uses: Vec<String>,
    pub properties: Vec<Property>,
    pub methods: HashMap<String, UserFunction>,
    pub method_visibility: HashMap<String, Visibility>,
    #[allow(dead_code)] // Will be used for reflection
    pub attributes: Vec<crate::ast::Attribute>,
}
```

#### 7e. Update statement execution to store attributes

**Location:** Search for where `Stmt::Function`, `Stmt::Class`, `Stmt::Interface`, and `Stmt::Trait` are handled in `execute_statement`

**For Stmt::Function (search for "Stmt::Function"):**

When creating `UserFunction`, add attributes field:

**Before:**
```rust
                let user_func = UserFunction {
                    params: params.clone(),
                    body: body.clone(),
                };
```

**After:**
```rust
                let user_func = UserFunction {
                    params: params.clone(),
                    body: body.clone(),
                    attributes: attributes.clone(),
                };
```

**For Stmt::Class (search for "Stmt::Class"):**

When creating `ClassDefinition`, add attributes field:

**Before:**
```rust
                let class_def = ClassDefinition {
                    name: name.clone(),
                    readonly: *readonly,
                    parent: parent.clone(),
                    properties: all_properties,
                    methods: methods_map,
                    method_visibility: visibility_map,
                };
```

**After:**
```rust
                let class_def = ClassDefinition {
                    name: name.clone(),
                    readonly: *readonly,
                    parent: parent.clone(),
                    properties: all_properties,
                    methods: methods_map,
                    method_visibility: visibility_map,
                    attributes: attributes.clone(),
                };
```

**For Stmt::Interface (search for "Stmt::Interface"):**

When creating `InterfaceDefinition`, add attributes field:

**Before:**
```rust
                let interface_def = InterfaceDefinition {
                    name: name.clone(),
                    parents: parents.clone(),
                    methods: method_sigs,
                    constants: const_map,
                };
```

**After:**
```rust
                let interface_def = InterfaceDefinition {
                    name: name.clone(),
                    parents: parents.clone(),
                    methods: method_sigs,
                    constants: const_map,
                    attributes: attributes.clone(),
                };
```

**For Stmt::Trait (search for "Stmt::Trait"):**

When creating `TraitDefinition`, add attributes field:

**Before:**
```rust
                let trait_def = TraitDefinition {
                    name: name.clone(),
                    uses: uses.clone(),
                    properties: properties.clone(),
                    methods: methods_map,
                    method_visibility: visibility_map,
                };
```

**After:**
```rust
                let trait_def = TraitDefinition {
                    name: name.clone(),
                    uses: uses.clone(),
                    properties: properties.clone(),
                    methods: methods_map,
                    method_visibility: visibility_map,
                    attributes: attributes.clone(),
                };
```

**Important:** In all these locations, make sure the destructured match patterns include `attributes`:

**Example for Stmt::Function:**

**Before:**
```rust
            Stmt::Function { name, params, body } => {
```

**After:**
```rust
            Stmt::Function { name, params, body, attributes } => {
```

### Step 8: Add Tests (`tests/attributes/`)

Create a new directory `tests/attributes/` and add comprehensive test files.

#### Test 1: `tests/attributes/class_single_attribute.vhpt`

```php
--TEST--
Attributes - single attribute on class
--FILE--
<?php
#[Route("/api/users")]
class UserController {
    public function index() {
        echo "Users list";
    }
}

$controller = new UserController();
$controller->index();
--EXPECT--
Users list
```

#### Test 2: `tests/attributes/class_multiple_attributes.vhpt`

```php
--TEST--
Attributes - multiple attributes on class
--FILE--
<?php
#[Route("/api/users")]
#[Middleware("auth")]
#[Cache(ttl: 300)]
class UserController {
    public function index() {
        echo "Authenticated users list";
    }
}

$controller = new UserController();
$controller->index();
--EXPECT--
Authenticated users list
```

#### Test 3: `tests/attributes/class_multiple_attributes_single_block.vhpt`

```php
--TEST--
Attributes - multiple attributes in single block
--FILE--
<?php
#[Route("/api/users"), Middleware("auth"), Cache(ttl: 300)]
class UserController {
    public function index() {
        echo "Cached users";
    }
}

$controller = new UserController();
$controller->index();
--EXPECT--
Cached users
```

#### Test 4: `tests/attributes/method_attribute.vhpt`

```php
--TEST--
Attributes - attribute on method
--FILE--
<?php
class UserController {
    #[Route("/api/users")]
    public function index() {
        echo "Users";
    }

    #[Route("/api/users/create")]
    #[ValidateRequest]
    public function create() {
        echo "Create user";
    }
}

$controller = new UserController();
$controller->index();
echo "\n";
$controller->create();
--EXPECT--
Users
Create user
```

#### Test 5: `tests/attributes/property_attribute.vhpt`

```php
--TEST--
Attributes - attribute on property
--FILE--
<?php
class User {
    #[NotNull]
    #[MaxLength(100)]
    public string $name;

    #[Email]
    public string $email;

    public function __construct($name, $email) {
        $this->name = $name;
        $this->email = $email;
    }
}

$user = new User("John Doe", "john@example.com");
echo $user->name;
echo "\n";
echo $user->email;
--EXPECT--
John Doe
john@example.com
```

#### Test 6: `tests/attributes/parameter_attribute.vhpt`

```php
--TEST--
Attributes - attribute on parameter
--FILE--
<?php
function createUser(
    #[NotBlank] $name,
    #[Email] #[Unique] $email
) {
    echo "User: " . $name . " (" . $email . ")";
}

createUser("Alice", "alice@example.com");
--EXPECT--
User: Alice (alice@example.com)
```

#### Test 7: `tests/attributes/function_attribute.vhpt`

```php
--TEST--
Attributes - attribute on function
--FILE--
<?php
#[Route("/api/health")]
#[Cache(ttl: 60)]
function healthCheck() {
    echo "OK";
}

healthCheck();
--EXPECT--
OK
```

#### Test 8: `tests/attributes/attribute_with_arguments_positional.vhpt`

```php
--TEST--
Attributes - positional arguments
--FILE--
<?php
#[Route("/api/users", "GET")]
class UserController {
    public function index() {
        echo "GET users";
    }
}

$controller = new UserController();
$controller->index();
--EXPECT--
GET users
```

#### Test 9: `tests/attributes/attribute_with_arguments_named.vhpt`

```php
--TEST--
Attributes - named arguments
--FILE--
<?php
#[Route(path: "/api/users", method: "POST")]
#[RateLimit(requests: 100, period: 60)]
class UserController {
    public function create() {
        echo "POST user";
    }
}

$controller = new UserController();
$controller->create();
--EXPECT--
POST user
```

#### Test 10: `tests/attributes/attribute_with_mixed_arguments.vhpt`

```php
--TEST--
Attributes - mixed positional and named arguments
--FILE--
<?php
#[Route("/api/users", method: "GET", cache: 300)]
class UserController {
    public function index() {
        echo "Cached GET";
    }
}

$controller = new UserController();
$controller->index();
--EXPECT--
Cached GET
```

#### Test 11: `tests/attributes/interface_attribute.vhpt`

```php
--TEST--
Attributes - attribute on interface
--FILE--
<?php
#[Deprecated]
#[Replaced(by: "UserServiceV2")]
interface UserService {
    public function getUsers();
}

class UserServiceImpl implements UserService {
    public function getUsers() {
        echo "Legacy users";
    }
}

$service = new UserServiceImpl();
$service->getUsers();
--EXPECT--
Legacy users
```

#### Test 12: `tests/attributes/trait_attribute.vhpt`

```php
--TEST--
Attributes - attribute on trait
--FILE--
<?php
#[Timestampable]
trait Timestamps {
    public $created_at;
    public $updated_at;
}

class Post {
    use Timestamps;

    public function __construct() {
        $this->created_at = "2023-01-01";
        echo $this->created_at;
    }
}

$post = new Post();
--EXPECT--
2023-01-01
```

#### Test 13: `tests/attributes/constructor_promotion_with_attribute.vhpt`

```php
--TEST--
Attributes - with constructor property promotion
--FILE--
<?php
class User {
    public function __construct(
        #[NotNull] #[MaxLength(100)]
        public string $name,
        #[Email]
        public string $email
    ) {}
}

$user = new User("Jane", "jane@example.com");
echo $user->name;
echo "\n";
echo $user->email;
--EXPECT--
Jane
jane@example.com
```

#### Test 14: `tests/attributes/attribute_no_arguments.vhpt`

```php
--TEST--
Attributes - without arguments
--FILE--
<?php
#[Deprecated]
class OldClass {
    #[Internal]
    public function legacy() {
        echo "Legacy method";
    }
}

$obj = new OldClass();
$obj->legacy();
--EXPECT--
Legacy method
```

#### Test 15: `tests/attributes/attribute_complex_values.vhpt`

```php
--TEST--
Attributes - complex argument values
--FILE--
<?php
#[Route("/api/users", methods: ["GET", "POST"], cache: 300)]
class UserController {
    public function handle() {
        echo "Complex attributes";
    }
}

$controller = new UserController();
$controller->handle();
--EXPECT--
Complex attributes
```

### Step 9: Update Documentation

#### 9a. Update `AGENTS.md`

**Location:** Line 403 (Phase 6 roadmap)

**Change:**
```markdown
### Phase 6: Modern PHP 8.x Features (In Progress)
- [x] Match Expressions (PHP 8.0)
- [x] Named Arguments (PHP 8.0)
- [x] Attributes (PHP 8.0)
- [ ] Enums (PHP 8.1)
- [ ] Fibers (PHP 8.1)
- [ ] Pipe Operator (PHP 8.5)
```

**Also update:** Around line 195+ (Current Features list)

**Add a new section after Match Expressions:**

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
```

#### 9b. Update `README.md`

Add to the features section:

```markdown
### Attributes (PHP 8.0)
- [x] Attribute syntax with `#[...]`
- [x] Positional and named arguments
- [x] Multiple attributes on declarations
- [x] Attributes on classes, functions, methods, properties, and parameters
```

#### 9c. Create or update `docs/features.md`

Add a comprehensive section:

```markdown
## Attributes (PHP 8.0)

Attributes provide a structured way to add metadata to declarations:

### Basic Syntax

```php
#[Route("/api/users")]
class UserController {
    #[Authorize("admin")]
    public function delete(#[FromBody] $data) {
        // ...
    }
}
```

### Multiple Attributes

```php
// Multiple blocks
#[Route("/api")]
#[Middleware("auth")]
class Controller {}

// Single block
#[Route("/api"), Middleware("auth")]
class Controller {}
```

### Arguments

```php
// Positional
#[Route("/users", "GET")]

// Named
#[Route(path: "/users", method: "GET")]

// Mixed
#[Route("/users", method: "GET")]
```

### Supported Locations

- Classes
- Interfaces
- Traits
- Methods (including interface methods)
- Properties
- Functions
- Parameters (including constructor promoted properties)
- Constants (interface constants)

Note: While VHP parses and stores attributes, reflection APIs to read them at runtime are not yet implemented.
```

## Key Considerations

### PHP Compatibility

1. **Syntax**: PHP 8.0 uses `#[...]` for attributes (different from `@` in some languages)
2. **Multiple attributes**: Can be specified as `#[A] #[B]` or `#[A, B]`
3. **Arguments**: Support both positional and named arguments (like function calls)
4. **Nesting**: Attribute arguments can contain complex expressions (arrays, constants, etc.)
5. **Locations**: Attributes can appear before classes, functions, methods, properties, parameters, and constants

### Edge Cases to Handle

1. **Comment vs Attribute**: `#` alone is a comment, but `#[` starts an attribute
2. **Whitespace**: Whitespace between `#` and `[` is NOT allowed - `# [Attr]` is invalid
3. **Empty attributes**: `#[]` is invalid, must have at least one attribute name
4. **Reserved names**: Some attribute names like `Attribute` itself have special meaning in PHP reflection (we don't need to handle this yet)
5. **Duplicate attributes**: PHP allows the same attribute multiple times (for now, we just store them)

### Implementation Notes

1. **Storage only**: For now, attributes are parsed and stored but not actively used
2. **Future enhancement**: When reflection is implemented, attributes can be read at runtime
3. **Validation**: We don't validate attribute names or arguments (PHP validates them at runtime via the Attribute class)
4. **Performance**: Attribute parsing should not significantly impact parse time

### Error Messages

Clear error messages for:
- Missing `[` after `#`
- Missing attribute name
- Malformed argument list
- Missing closing `]`

Examples:
- `"Expected '[' after '#' for attribute at line X, column Y"`
- `"Expected attribute name at line X, column Y"`
- `"Expected ']' after attribute at line X, column Y"`

## Test Cases Summary

The test suite verifies:

1. Single attribute on class
2. Multiple attributes on class (separate blocks)
3. Multiple attributes in single block
4. Attributes on methods
5. Attributes on properties
6. Attributes on parameters
7. Attributes on functions
8. Positional arguments
9. Named arguments
10. Mixed positional and named arguments
11. Attributes on interfaces
12. Attributes on traits
13. Attributes with constructor promotion
14. Attributes without arguments
15. Attributes with complex values (arrays, etc.)

## Reference Implementation

For implementation patterns, reference these existing features:

| Pattern | Reference | What to Learn |
|---------|-----------|---------------|
| Token addition | `src/token.rs` | How to add new tokens |
| Lexer lookahead | `src/lexer.rs` (for `?>` or `===`) | How to check next character |
| AST struct | `src/ast/stmt.rs` Property/Method | How to define data structures |
| Parser helper | `parse_visibility()` | How to parse optional modifiers |
| Named arguments | `src/ast/expr.rs` Argument | How named args work |
| Multiple items | `parse_interfaces()` | Comma-separated parsing |
| Statement decoration | Visibility on properties | How to attach metadata to declarations |

## Implementation Checklist

- [ ] Add `Hash` token (if not exists) to `TokenKind`
- [ ] Update lexer to distinguish `#[` from `#` comment
- [ ] Create `Attribute` and `AttributeArgument` structs
- [ ] Add `attributes: Vec<Attribute>` to all relevant AST nodes
- [ ] Implement `parse_attributes()` function
- [ ] Integrate attribute parsing before all declarations
- [ ] Update all `Stmt` constructors to include attributes field
- [ ] Update interpreter structs to store attributes
- [ ] Update statement execution to copy attributes to definitions
- [ ] Add 15 comprehensive test files
- [ ] Update AGENTS.md documentation
- [ ] Update README.md
- [ ] Update or create docs/features.md
- [ ] Run full test suite to ensure no regressions
- [ ] Verify clippy passes with no warnings

## Next Steps After Completion

After implementing attributes, the logical next features in Phase 6 are:

1. **Enums (PHP 8.1)**: Type-safe enumerations with optional backing values
2. **Fibers (PHP 8.1)**: Lightweight cooperative multitasking primitives
3. **Pipe Operator (PHP 8.5)**: Functional piping operator for cleaner code

Attributes are foundational for many frameworks and libraries, making them a high-priority feature.
