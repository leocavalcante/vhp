# Plan: Abstract Classes and Methods

## Overview

Abstract classes are classes that cannot be instantiated directly and may contain abstract methods (methods without implementation) that must be implemented by child classes.

**PHP Example:**
```php
<?php
abstract class Shape {
    protected string $color;
    
    public function __construct(string $color) {
        $this->color = $color;
    }
    
    // Concrete method (has implementation)
    public function getColor(): string {
        return $this->color;
    }
    
    // Abstract method (no implementation)
    abstract public function area(): float;
    abstract public function perimeter(): float;
}

class Rectangle extends Shape {
    private float $width;
    private float $height;
    
    public function __construct(string $color, float $w, float $h) {
        parent::__construct($color);
        $this->width = $w;
        $this->height = $h;
    }
    
    public function area(): float {
        return $this->width * $this->height;
    }
    
    public function perimeter(): float {
        return 2 * ($this->width + $this->height);
    }
}

// This works:
$rect = new Rectangle("red", 5, 3);
echo $rect->area(); // 15

// This would error:
// $shape = new Shape("blue"); // Cannot instantiate abstract class
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/token.rs` | Add `Abstract` token |
| `src/ast/stmt.rs` | Add `is_abstract` to Class and Method |
| `src/parser/stmt/mod.rs` | Parse abstract modifier |
| `src/interpreter/objects/mod.rs` | Validate abstract class rules |
| `tests/classes/abstract_*.vhpt` | Test files |

## Implementation Steps

### Step 1: Add Token (`src/token.rs`)

Add the `abstract` keyword token:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ... existing tokens ...
    
    // Keywords
    Abstract,
    
    // ... rest of tokens ...
}
```

### Step 2: Update Lexer (`src/lexer/mod.rs`)

Add `abstract` to keyword recognition:

```rust
fn tokenize_identifier(&mut self) -> TokenKind {
    match ident.to_lowercase().as_str() {
        // ... existing keywords ...
        "abstract" => TokenKind::Abstract,
        // ...
    }
}
```

### Step 3: Update Class AST (`src/ast/stmt.rs`)

Add `is_abstract` flag to `Stmt::Class`:

```rust
Stmt::Class {
    name: String,
    is_abstract: bool,        // NEW: true if abstract class
    is_readonly: bool,
    parent: Option<String>,
    interfaces: Vec<String>,
    traits: Vec<TraitUse>,
    properties: Vec<Property>,
    methods: Vec<Method>,
    attributes: Vec<Attribute>,
},
```

### Step 4: Update Method Structure (`src/ast/stmt.rs`)

Add `is_abstract` flag to `Method`:

```rust
#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_abstract: bool,        // NEW: true if abstract method
    pub params: Vec<FunctionParam>,
    pub return_type: Option<TypeHint>,
    pub body: Vec<Stmt>,          // Empty for abstract methods
    pub attributes: Vec<Attribute>,
}
```

### Step 5: Parse Abstract Modifier (`src/parser/stmt/mod.rs`)

Update class parsing to handle `abstract class`:

```rust
fn parse_class_declaration(&mut self) -> Result<Stmt, String> {
    // Parse attributes
    let attributes = self.parse_attributes()?;
    
    // Check for modifiers before 'class'
    let mut is_abstract = false;
    let mut is_readonly = false;
    let mut is_final = false;
    
    // Parse modifiers in any order
    loop {
        match &self.current_token().kind {
            TokenKind::Abstract => {
                if is_final {
                    return Err("Cannot use 'abstract' with 'final'".to_string());
                }
                is_abstract = true;
                self.advance();
            }
            TokenKind::Final => {
                if is_abstract {
                    return Err("Cannot use 'final' with 'abstract'".to_string());
                }
                is_final = true;
                self.advance();
            }
            TokenKind::Readonly => {
                is_readonly = true;
                self.advance();
            }
            TokenKind::Class => break,
            _ => return Err(format!(
                "Expected 'class', got {:?}",
                self.current_token().kind
            )),
        }
    }
    
    // Now parse 'class' keyword
    self.expect(&TokenKind::Class)?;
    
    // Parse class name
    let name = self.expect_identifier()?;
    
    // ... rest of class parsing (extends, implements, body) ...
    
    Ok(Stmt::Class {
        name,
        is_abstract,
        is_readonly,
        is_final,
        parent,
        interfaces,
        traits,
        properties,
        methods,
        attributes,
    })
}
```

### Step 6: Parse Abstract Methods

Update method parsing inside class body:

```rust
fn parse_class_member(&mut self) -> Result<ClassMember, String> {
    // Parse attributes
    let attributes = self.parse_attributes()?;
    
    // Parse modifiers
    let mut visibility = None;
    let mut is_static = false;
    let mut is_abstract = false;
    let mut is_final = false;
    
    loop {
        match &self.current_token().kind {
            TokenKind::Public => {
                visibility = Some(Visibility::Public);
                self.advance();
            }
            TokenKind::Protected => {
                visibility = Some(Visibility::Protected);
                self.advance();
            }
            TokenKind::Private => {
                visibility = Some(Visibility::Private);
                self.advance();
            }
            TokenKind::Static => {
                is_static = true;
                self.advance();
            }
            TokenKind::Abstract => {
                if is_final {
                    return Err("Cannot use 'abstract' with 'final'".to_string());
                }
                is_abstract = true;
                self.advance();
            }
            TokenKind::Final => {
                if is_abstract {
                    return Err("Cannot use 'final' with 'abstract'".to_string());
                }
                is_final = true;
                self.advance();
            }
            _ => break,
        }
    }
    
    // Now parse function
    if self.check(&TokenKind::Function) {
        self.advance(); // consume 'function'
        let name = self.expect_identifier()?;
        
        // Parse parameters
        self.expect(&TokenKind::LeftParen)?;
        let params = self.parse_function_params()?;
        self.expect(&TokenKind::RightParen)?;
        
        // Parse optional return type
        let return_type = if self.check(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type_hint()?)
        } else {
            None
        };
        
        // Abstract methods end with semicolon, not body
        let body = if is_abstract {
            self.expect(&TokenKind::Semicolon)?;
            Vec::new()
        } else {
            self.expect(&TokenKind::LeftBrace)?;
            let stmts = self.parse_block()?;
            self.expect(&TokenKind::RightBrace)?;
            stmts
        };
        
        return Ok(ClassMember::Method(Method {
            name,
            visibility: visibility.unwrap_or(Visibility::Public),
            is_static,
            is_abstract,
            is_final,
            params,
            return_type,
            body,
            attributes,
        }));
    }
    
    // ... parse property ...
}
```

### Step 7: Validation Rules

Add validation in class parsing or during interpretation:

```rust
fn validate_class(class: &Stmt::Class) -> Result<(), String> {
    if let Stmt::Class { name, is_abstract, methods, .. } = class {
        // Rule 1: Non-abstract class cannot have abstract methods
        if !is_abstract {
            for method in methods {
                if method.is_abstract {
                    return Err(format!(
                        "Class {} contains abstract method {} and must be declared abstract",
                        name, method.name
                    ));
                }
            }
        }
        
        // Rule 2: Abstract methods cannot have body
        for method in methods {
            if method.is_abstract && !method.body.is_empty() {
                return Err(format!(
                    "Abstract method {}::{} cannot contain body",
                    name, method.name
                ));
            }
        }
    }
    Ok(())
}
```

### Step 8: Prevent Abstract Class Instantiation (`src/interpreter/objects/mod.rs`)

When handling `new ClassName()`:

```rust
fn create_instance(&mut self, class_name: &str, args: Vec<Value>) -> Result<Value, String> {
    let class = self.get_class(class_name)?;
    
    // Check if abstract
    if class.is_abstract {
        return Err(format!(
            "Cannot instantiate abstract class {}",
            class_name
        ));
    }
    
    // ... rest of instantiation ...
}
```

### Step 9: Check Abstract Method Implementation

When a class extends an abstract class:

```rust
fn validate_inheritance(&self, child: &Class, parent: &Class) -> Result<(), String> {
    if !child.is_abstract {
        // Non-abstract child must implement all abstract methods from parent
        for parent_method in &parent.methods {
            if parent_method.is_abstract {
                let implemented = child.methods.iter().any(|m| {
                    m.name.eq_ignore_ascii_case(&parent_method.name)
                });
                
                if !implemented {
                    return Err(format!(
                        "Class {} must implement abstract method {}::{}", 
                        child.name, 
                        parent.name,
                        parent_method.name
                    ));
                }
            }
        }
    }
    Ok(())
}
```

### Step 10: Add Tests

**tests/classes/abstract_class_basic.vhpt**
```
--TEST--
Basic abstract class with abstract method
--FILE--
<?php
abstract class Animal {
    abstract public function speak();
}

class Dog extends Animal {
    public function speak() {
        echo "Woof!";
    }
}

$dog = new Dog();
$dog->speak();
--EXPECT--
Woof!
```

**tests/classes/abstract_cannot_instantiate.vhpt**
```
--TEST--
Cannot instantiate abstract class
--FILE--
<?php
abstract class Base {}
new Base();
--EXPECT_ERROR--
Cannot instantiate abstract class Base
```

**tests/classes/abstract_with_concrete.vhpt**
```
--TEST--
Abstract class with concrete methods
--FILE--
<?php
abstract class Shape {
    protected $color;
    
    public function __construct($color) {
        $this->color = $color;
    }
    
    public function getColor() {
        return $this->color;
    }
    
    abstract public function area();
}

class Square extends Shape {
    private $side;
    
    public function __construct($color, $side) {
        parent::__construct($color);
        $this->side = $side;
    }
    
    public function area() {
        return $this->side * $this->side;
    }
}

$sq = new Square("red", 5);
echo $sq->getColor() . "\n";
echo $sq->area();
--EXPECT--
red
25
```

**tests/classes/abstract_must_implement.vhpt**
```
--TEST--
Non-abstract class must implement abstract methods
--FILE--
<?php
abstract class Base {
    abstract public function required();
}

class Child extends Base {
    // Missing implementation of required()
}
--EXPECT_ERROR--
must implement abstract method
```

**tests/classes/abstract_chain.vhpt**
```
--TEST--
Abstract class extending abstract class
--FILE--
<?php
abstract class A {
    abstract public function foo();
}

abstract class B extends A {
    abstract public function bar();
    // B doesn't need to implement foo() because it's also abstract
}

class C extends B {
    public function foo() {
        return "foo";
    }
    public function bar() {
        return "bar";
    }
}

$c = new C();
echo $c->foo() . "\n";
echo $c->bar();
--EXPECT--
foo
bar
```

**tests/classes/abstract_non_abstract_class_error.vhpt**
```
--TEST--
Non-abstract class cannot have abstract methods
--FILE--
<?php
class NotAbstract {
    abstract public function shouldFail();
}
--EXPECT_ERROR--
must be declared abstract
```

**tests/classes/abstract_method_visibility.vhpt**
```
--TEST--
Abstract method with different visibilities
--FILE--
<?php
abstract class Base {
    abstract public function pub();
    abstract protected function prot();
}

class Child extends Base {
    public function pub() {
        return "public";
    }
    
    protected function prot() {
        return "protected";
    }
    
    public function callProt() {
        return $this->prot();
    }
}

$c = new Child();
echo $c->pub() . "\n";
echo $c->callProt();
--EXPECT--
public
protected
```

**tests/classes/abstract_with_constructor.vhpt**
```
--TEST--
Abstract class with constructor
--FILE--
<?php
abstract class Base {
    protected $name;
    
    public function __construct($name) {
        $this->name = $name;
    }
    
    abstract public function greet();
}

class Hello extends Base {
    public function greet() {
        return "Hello, " . $this->name;
    }
}

$h = new Hello("World");
echo $h->greet();
--EXPECT--
Hello, World
```

## Key Rules

1. Abstract classes cannot be instantiated directly
2. Abstract methods have no body (end with semicolon)
3. Abstract methods can only exist in abstract classes
4. Child classes must implement all abstract methods (unless also abstract)
5. Abstract methods can have any visibility (public, protected, private)
6. Cannot combine `abstract` and `final` (contradictory)
7. Abstract methods can specify parameter and return types

## PHP Compatibility Notes

- `abstract` keyword available since PHP 5
- Abstract classes can have constructors
- Abstract methods in child class can have additional default parameters
- Visibility cannot be more restrictive when implementing (LSP)

## Error Messages

Match PHP error format:
- `Cannot instantiate abstract class X`
- `Class X contains abstract method Y and must be declared abstract`
- `Abstract function X::Y() cannot contain body`
- `Class X must implement abstract method Parent::Y`
