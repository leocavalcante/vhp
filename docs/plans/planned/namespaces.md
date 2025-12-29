# Plan: Namespaces

## Overview

Namespaces provide a way to group related classes, interfaces, functions, and constants. They prevent naming conflicts and organize code logically.

**PHP Example:**
```php
<?php
// File: src/MyApp/Database/Connection.php
namespace MyApp\Database;

class Connection {
    public function connect() {
        return "Connected!";
    }
}

// File: src/MyApp/Http/Connection.php  
namespace MyApp\Http;

class Connection {
    public function connect() {
        return "HTTP Connected!";
    }
}

// File: main.php
namespace MyApp;

use MyApp\Database\Connection as DbConnection;
use MyApp\Http\Connection as HttpConnection;
use function MyApp\Helpers\format_date;
use const MyApp\Config\VERSION;

$db = new DbConnection();
$http = new HttpConnection();

// Fully qualified name (starts with \)
$other = new \OtherPackage\SomeClass();
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/token.rs` | Add `Namespace`, `Use`, `As`, `Backslash` tokens |
| `src/ast/stmt.rs` | Add namespace-related statements |
| `src/parser/stmt/mod.rs` | Parse namespace declarations and use statements |
| `src/interpreter/mod.rs` | Namespace resolution logic |
| `tests/namespaces/*.vhpt` | Test files |

## Implementation Steps

### Step 1: Add Tokens (`src/token.rs`)

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ... existing tokens ...
    
    // Namespace-related
    Namespace,
    Use,
    As,
    Backslash,    // \
    
    // ... rest of tokens ...
}
```

### Step 2: Update Lexer (`src/lexer/mod.rs`)

```rust
fn tokenize_identifier(&mut self) -> TokenKind {
    match ident.to_lowercase().as_str() {
        // ... existing keywords ...
        "namespace" => TokenKind::Namespace,
        "use" => TokenKind::Use,
        "as" => TokenKind::As,
        // ...
    }
}

// Add backslash handling
fn tokenize(&mut self) -> Result<Vec<Token>, String> {
    match c {
        '\\' => {
            self.advance();
            tokens.push(Token::new(TokenKind::Backslash, self.position()));
        }
        // ...
    }
}
```

### Step 3: Add AST Structures (`src/ast/stmt.rs`)

```rust
/// Qualified name (e.g., MyApp\Database\Connection)
#[derive(Debug, Clone, PartialEq)]
pub struct QualifiedName {
    /// Path parts (e.g., ["MyApp", "Database", "Connection"])
    pub parts: Vec<String>,
    /// Whether it starts with \ (fully qualified)
    pub is_fully_qualified: bool,
}

impl QualifiedName {
    pub fn new(parts: Vec<String>, is_fully_qualified: bool) -> Self {
        Self { parts, is_fully_qualified }
    }
    
    pub fn to_string(&self) -> String {
        let prefix = if self.is_fully_qualified { "\\" } else { "" };
        format!("{}{}", prefix, self.parts.join("\\"))
    }
    
    /// Get just the final name (class name, function name, etc.)
    pub fn last(&self) -> Option<&String> {
        self.parts.last()
    }
    
    /// Get namespace portion (everything except last part)
    pub fn namespace(&self) -> Vec<String> {
        if self.parts.len() > 1 {
            self.parts[..self.parts.len()-1].to_vec()
        } else {
            vec![]
        }
    }
}

/// Use statement type
#[derive(Debug, Clone)]
pub enum UseType {
    Class,       // use Foo\Bar;
    Function,    // use function Foo\helper;
    Constant,    // use const Foo\VALUE;
}

/// Single use import
#[derive(Debug, Clone)]
pub struct UseItem {
    pub name: QualifiedName,
    pub alias: Option<String>,  // `as` alias
    pub use_type: UseType,
}

/// Group use statement: use Foo\{Bar, Baz};
#[derive(Debug, Clone)]
pub struct GroupUse {
    pub prefix: QualifiedName,
    pub items: Vec<UseItem>,
}

// Add to Stmt enum
pub enum Stmt {
    // ... existing variants ...
    
    /// Namespace declaration
    Namespace {
        name: Option<QualifiedName>,  // None for global namespace
        body: NamespaceBody,
    },
    
    /// Use statement
    Use(Vec<UseItem>),
    
    /// Group use statement (PHP 7.0+)
    GroupUse(GroupUse),
}

/// Namespace body style
#[derive(Debug, Clone)]
pub enum NamespaceBody {
    /// Braced: namespace Foo { ... }
    Braced(Vec<Stmt>),
    /// Unbraced: namespace Foo; (rest of file)
    Unbraced,
}
```

### Step 4: Update Class/Interface/Trait for Namespaced Names

Allow `QualifiedName` in extends/implements:

```rust
Stmt::Class {
    name: String,
    is_abstract: bool,
    is_final: bool,
    is_readonly: bool,
    parent: Option<QualifiedName>,      // Changed from Option<String>
    interfaces: Vec<QualifiedName>,     // Changed from Vec<String>
    // ...
}
```

### Step 5: Parse Namespace Declaration (`src/parser/stmt/mod.rs`)

```rust
fn parse_namespace(&mut self) -> Result<Stmt, String> {
    self.expect(&TokenKind::Namespace)?;
    
    // Parse namespace name (optional for global namespace block)
    let name = if !self.check(&TokenKind::LeftBrace) && !self.check(&TokenKind::Semicolon) {
        Some(self.parse_qualified_name()?)
    } else {
        None
    };
    
    // Determine body style
    let body = if self.check(&TokenKind::LeftBrace) {
        // Braced namespace: namespace Foo { ... }
        self.advance(); // consume {
        let stmts = self.parse_statements_until(&TokenKind::RightBrace)?;
        self.expect(&TokenKind::RightBrace)?;
        NamespaceBody::Braced(stmts)
    } else {
        // Unbraced namespace: namespace Foo; (rest of file)
        self.expect(&TokenKind::Semicolon)?;
        NamespaceBody::Unbraced
    };
    
    Ok(Stmt::Namespace { name, body })
}

/// Parse qualified name (e.g., Foo\Bar\Baz)
fn parse_qualified_name(&mut self) -> Result<QualifiedName, String> {
    let is_fully_qualified = if self.check(&TokenKind::Backslash) {
        self.advance();
        true
    } else {
        false
    };
    
    let mut parts = vec![];
    
    // First part
    let first = self.expect_identifier()?;
    parts.push(first);
    
    // Additional parts after \
    while self.check(&TokenKind::Backslash) {
        self.advance();
        let part = self.expect_identifier()?;
        parts.push(part);
    }
    
    Ok(QualifiedName::new(parts, is_fully_qualified))
}
```

### Step 6: Parse Use Statements

```rust
fn parse_use(&mut self) -> Result<Stmt, String> {
    self.expect(&TokenKind::Use)?;
    
    // Check for `use function` or `use const`
    let default_type = if self.check(&TokenKind::Function) {
        self.advance();
        UseType::Function
    } else if self.check(&TokenKind::Const) {
        self.advance();
        UseType::Constant
    } else {
        UseType::Class
    };
    
    // Parse the name
    let name = self.parse_qualified_name()?;
    
    // Check for group use: use Foo\{Bar, Baz}
    if self.check(&TokenKind::LeftBrace) {
        return self.parse_group_use(name, default_type);
    }
    
    // Parse single or multiple uses
    let mut items = vec![];
    
    // First item
    let alias = if self.check(&TokenKind::As) {
        self.advance();
        Some(self.expect_identifier()?)
    } else {
        None
    };
    
    items.push(UseItem {
        name,
        alias,
        use_type: default_type.clone(),
    });
    
    // Additional items after comma
    while self.check(&TokenKind::Comma) {
        self.advance();
        let name = self.parse_qualified_name()?;
        let alias = if self.check(&TokenKind::As) {
            self.advance();
            Some(self.expect_identifier()?)
        } else {
            None
        };
        items.push(UseItem {
            name,
            alias,
            use_type: default_type.clone(),
        });
    }
    
    self.expect(&TokenKind::Semicolon)?;
    Ok(Stmt::Use(items))
}

fn parse_group_use(
    &mut self,
    prefix: QualifiedName,
    default_type: UseType,
) -> Result<Stmt, String> {
    self.expect(&TokenKind::LeftBrace)?;
    
    let mut items = vec![];
    
    loop {
        // Check for type modifier
        let use_type = if self.check(&TokenKind::Function) {
            self.advance();
            UseType::Function
        } else if self.check(&TokenKind::Const) {
            self.advance();
            UseType::Constant
        } else {
            default_type.clone()
        };
        
        let name = self.parse_qualified_name()?;
        let alias = if self.check(&TokenKind::As) {
            self.advance();
            Some(self.expect_identifier()?)
        } else {
            None
        };
        
        items.push(UseItem { name, alias, use_type });
        
        if !self.check(&TokenKind::Comma) {
            break;
        }
        self.advance();
    }
    
    self.expect(&TokenKind::RightBrace)?;
    self.expect(&TokenKind::Semicolon)?;
    
    Ok(Stmt::GroupUse(GroupUse { prefix, items }))
}
```

### Step 7: Namespace Resolution Logic (`src/interpreter/mod.rs`)

```rust
/// Namespace context for name resolution
#[derive(Debug, Clone)]
pub struct NamespaceContext {
    /// Current namespace (empty for global)
    pub current: Vec<String>,
    /// Use imports: alias -> fully qualified name
    pub class_imports: HashMap<String, QualifiedName>,
    pub function_imports: HashMap<String, QualifiedName>,
    pub constant_imports: HashMap<String, QualifiedName>,
}

impl NamespaceContext {
    pub fn new() -> Self {
        Self {
            current: vec![],
            class_imports: HashMap::new(),
            function_imports: HashMap::new(),
            constant_imports: HashMap::new(),
        }
    }
    
    /// Resolve a class name to fully qualified
    pub fn resolve_class(&self, name: &QualifiedName) -> String {
        // Already fully qualified
        if name.is_fully_qualified {
            return name.parts.join("\\");
        }
        
        // Single name - check imports first
        if name.parts.len() == 1 {
            let simple_name = &name.parts[0];
            if let Some(imported) = self.class_imports.get(simple_name) {
                return imported.to_string();
            }
        }
        
        // Relative name - prepend current namespace
        let mut full = self.current.clone();
        full.extend(name.parts.clone());
        full.join("\\")
    }
    
    /// Resolve a function name
    pub fn resolve_function(&self, name: &QualifiedName) -> String {
        if name.is_fully_qualified {
            return name.parts.join("\\");
        }
        
        if name.parts.len() == 1 {
            let simple_name = &name.parts[0];
            
            // Check function imports
            if let Some(imported) = self.function_imports.get(simple_name) {
                return imported.to_string();
            }
            
            // For unqualified function names, PHP falls back to global
            // We'll need to check if namespaced version exists first
        }
        
        let mut full = self.current.clone();
        full.extend(name.parts.clone());
        full.join("\\")
    }
    
    /// Add a use import
    pub fn add_import(&mut self, item: &UseItem) {
        let alias = item.alias.clone()
            .unwrap_or_else(|| item.name.last().cloned().unwrap_or_default());
        
        match item.use_type {
            UseType::Class => {
                self.class_imports.insert(alias, item.name.clone());
            }
            UseType::Function => {
                self.function_imports.insert(alias, item.name.clone());
            }
            UseType::Constant => {
                self.constant_imports.insert(alias, item.name.clone());
            }
        }
    }
}
```

### Step 8: Update Class Registry

Store classes with fully qualified names:

```rust
/// Class registry with namespace support
pub struct ClassRegistry {
    /// Classes by fully qualified name (lowercase for case-insensitivity)
    classes: HashMap<String, ClassDef>,
}

impl ClassRegistry {
    pub fn register(&mut self, namespace: &[String], name: &str, class: ClassDef) {
        let fqn = if namespace.is_empty() {
            name.to_lowercase()
        } else {
            format!("{}\\{}", namespace.join("\\"), name).to_lowercase()
        };
        self.classes.insert(fqn, class);
    }
    
    pub fn get(&self, fqn: &str) -> Option<&ClassDef> {
        self.classes.get(&fqn.to_lowercase())
    }
}
```

### Step 9: Add Tests

**tests/namespaces/basic_namespace.vhpt**
```
--TEST--
Basic namespace declaration
--FILE--
<?php
namespace MyApp;

class User {
    public function getName() {
        return "User from MyApp";
    }
}

$user = new User();
echo $user->getName();
--EXPECT--
User from MyApp
```

**tests/namespaces/nested_namespace.vhpt**
```
--TEST--
Nested namespace
--FILE--
<?php
namespace MyApp\Database;

class Connection {
    public function connect() {
        return "Connected to database";
    }
}

$conn = new Connection();
echo $conn->connect();
--EXPECT--
Connected to database
```

**tests/namespaces/use_statement.vhpt**
```
--TEST--
Use statement with alias
--FILE--
<?php
namespace MyApp;

class Logger {
    public static function log($msg) {
        return "Logged: $msg";
    }
}

namespace Main;

use MyApp\Logger as L;

echo L::log("test");
--EXPECT--
Logged: test
```

**tests/namespaces/fully_qualified.vhpt**
```
--TEST--
Fully qualified name with backslash
--FILE--
<?php
namespace MyApp;

class Helper {}

namespace Other;

$h = new \MyApp\Helper();
echo get_class($h);
--EXPECT--
MyApp\Helper
```

**tests/namespaces/braced_namespace.vhpt**
```
--TEST--
Braced namespace syntax
--FILE--
<?php
namespace Foo {
    class A {
        public function name() { return "Foo\\A"; }
    }
}

namespace Bar {
    class A {
        public function name() { return "Bar\\A"; }
    }
}

namespace {
    $foo = new \Foo\A();
    $bar = new \Bar\A();
    echo $foo->name() . "\n";
    echo $bar->name();
}
--EXPECT--
Foo\A
Bar\A
```

**tests/namespaces/use_function.vhpt**
```
--TEST--
Use function statement
--FILE--
<?php
namespace Utils;

function format($text) {
    return strtoupper($text);
}

namespace Main;

use function Utils\format;

echo format("hello");
--EXPECT--
HELLO
```

**tests/namespaces/use_const.vhpt**
```
--TEST--
Use const statement
--FILE--
<?php
namespace Config;

const VERSION = "1.0.0";

namespace Main;

use const Config\VERSION;

echo VERSION;
--EXPECT--
1.0.0
```

**tests/namespaces/group_use.vhpt**
```
--TEST--
Group use statement (PHP 7.0+)
--FILE--
<?php
namespace MyApp\Models {
    class User {
        public function type() { return "User"; }
    }
    class Post {
        public function type() { return "Post"; }
    }
}

namespace Main;

use MyApp\Models\{User, Post};

$u = new User();
$p = new Post();
echo $u->type() . "\n";
echo $p->type();
--EXPECT--
User
Post
```

**tests/namespaces/global_fallback.vhpt**
```
--TEST--
Global namespace fallback for functions
--FILE--
<?php
namespace MyApp;

// strlen should fall back to global \strlen
echo strlen("hello");
--EXPECT--
5
```

## Key Rules

1. **Namespace declaration**: Must be first statement (except declare)
2. **Multiple namespaces**: Use braced syntax for multiple in one file
3. **Global namespace**: Empty namespace or `namespace { }` block
4. **Resolution order**:
   - Fully qualified (`\Foo\Bar`): Direct lookup
   - Qualified (`Foo\Bar`): Prepend current namespace
   - Unqualified (`Bar`): Check imports, then current namespace, then global (for functions/constants)
5. **Case sensitivity**: Namespace and class names are case-insensitive (like PHP)

## PHP Compatibility Notes

| Feature | PHP Version |
|---------|-------------|
| Namespaces | 5.3 |
| Group use | 7.0 |
| Mixed group use | 7.0 |
| Trailing comma in use | 7.2 |

## Implementation Order

1. Tokens and lexer (Namespace, Use, As, Backslash)
2. QualifiedName parsing
3. Basic namespace declarations
4. Simple use statements
5. Namespace resolution for classes
6. Use with aliases
7. Function and constant imports
8. Group use (PHP 7.0)
9. Global fallback for functions/constants

## Error Messages

- `Namespace declaration must be the first statement`
- `Cannot redeclare class X`
- `Cannot use X as Y because the name is already in use`
- `Class 'X' not found`
