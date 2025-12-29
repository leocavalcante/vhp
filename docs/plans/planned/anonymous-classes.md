# Plan: Anonymous Classes

## Overview

Anonymous classes are classes without a name, defined inline at instantiation. They're useful for creating simple one-off objects, especially for implementing interfaces.

**PHP Example:**
```php
<?php
interface Logger {
    public function log(string $message): void;
}

// Anonymous class implementing an interface
$logger = new class implements Logger {
    public function log(string $message): void {
        echo "[LOG] $message\n";
    }
};

$logger->log("Hello");

// Anonymous class extending a class
class Base {
    protected $value;
    public function __construct($value) {
        $this->value = $value;
    }
}

$obj = new class(42) extends Base {
    public function getValue() {
        return $this->value;
    }
};

echo $obj->getValue(); // 42

// Anonymous class with constructor arguments
$greeting = "Hello";
$greeter = new class($greeting) {
    private $msg;
    public function __construct(string $msg) {
        $this->msg = $msg;
    }
    public function greet() {
        return $this->msg;
    }
};
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/ast/expr.rs` | Add `NewAnonymousClass` expression |
| `src/parser/expr/mod.rs` | Parse anonymous class syntax |
| `src/interpreter/objects/mod.rs` | Handle anonymous class instantiation |
| `tests/classes/anonymous_*.vhpt` | Test files |

Note: No new tokens needed - uses existing `new`, `class`, `extends`, `implements` tokens.

## Implementation Steps

### Step 1: Add Anonymous Class Expression (`src/ast/expr.rs`)

```rust
#[derive(Debug, Clone)]
pub enum Expr {
    // ... existing variants ...
    
    /// Anonymous class: new class(...) extends/implements { ... }
    NewAnonymousClass {
        /// Constructor arguments passed to new class(...)
        constructor_args: Vec<Expr>,
        /// Parent class (if extends)
        parent: Option<String>,
        /// Implemented interfaces
        interfaces: Vec<String>,
        /// Trait uses
        traits: Vec<TraitUse>,
        /// Class properties
        properties: Vec<Property>,
        /// Class methods
        methods: Vec<Method>,
        /// Class constants
        constants: Vec<ClassConstant>,
    },
}
```

### Step 2: Parse Anonymous Class (`src/parser/expr/mod.rs`)

Modify `new` expression parsing to handle `new class`:

```rust
fn parse_new(&mut self) -> Result<Expr, String> {
    self.expect(&TokenKind::New)?;
    
    // Check for anonymous class: new class ...
    if self.check(&TokenKind::Class) {
        return self.parse_anonymous_class();
    }
    
    // Regular new expression: new ClassName(...)
    let class_name = self.expect_identifier()?;
    
    let args = if self.check(&TokenKind::LeftParen) {
        self.advance();
        let args = self.parse_argument_list()?;
        self.expect(&TokenKind::RightParen)?;
        args
    } else {
        vec![]
    };
    
    Ok(Expr::New {
        class_name,
        args,
    })
}

fn parse_anonymous_class(&mut self) -> Result<Expr, String> {
    self.expect(&TokenKind::Class)?;
    
    // Parse optional constructor arguments: new class(arg1, arg2)
    let constructor_args = if self.check(&TokenKind::LeftParen) {
        self.advance();
        let args = self.parse_argument_list()?;
        self.expect(&TokenKind::RightParen)?;
        args
    } else {
        vec![]
    };
    
    // Parse optional extends
    let parent = if self.check(&TokenKind::Extends) {
        self.advance();
        Some(self.expect_identifier()?)
    } else {
        None
    };
    
    // Parse optional implements
    let interfaces = if self.check(&TokenKind::Implements) {
        self.advance();
        self.parse_identifier_list()?
    } else {
        vec![]
    };
    
    // Parse class body
    self.expect(&TokenKind::LeftBrace)?;
    
    let mut traits = vec![];
    let mut properties = vec![];
    let mut methods = vec![];
    let mut constants = vec![];
    
    while !self.check(&TokenKind::RightBrace) {
        let member = self.parse_class_member()?;
        match member {
            ClassMember::TraitUse(t) => traits.push(t),
            ClassMember::Property(p) => properties.push(p),
            ClassMember::Method(m) => methods.push(m),
            ClassMember::Constant(c) => constants.push(c),
        }
    }
    
    self.expect(&TokenKind::RightBrace)?;
    
    Ok(Expr::NewAnonymousClass {
        constructor_args,
        parent,
        interfaces,
        traits,
        properties,
        methods,
        constants,
    })
}
```

### Step 3: Generate Anonymous Class Names

Anonymous classes need internal names for debugging and `get_class()`:

```rust
impl Interpreter {
    /// Counter for generating unique anonymous class names
    anonymous_class_counter: u64,
    
    /// Generate a unique anonymous class name
    fn generate_anonymous_class_name(&mut self) -> String {
        self.anonymous_class_counter += 1;
        // PHP format: class@anonymous/path/to/file.php:line$id
        // Simplified: class@anonymous$id
        format!("class@anonymous${}", self.anonymous_class_counter)
    }
}
```

### Step 4: Evaluate Anonymous Class (`src/interpreter/expr_eval/mod.rs`)

```rust
fn eval_new_anonymous_class(
    &mut self,
    constructor_args: &[Expr],
    parent: &Option<String>,
    interfaces: &[String],
    traits: &[TraitUse],
    properties: &[Property],
    methods: &[Method],
    constants: &[ClassConstant],
) -> Result<Value, String> {
    // Generate unique class name
    let class_name = self.generate_anonymous_class_name();
    
    // Create class definition
    let class_def = ClassDef {
        name: class_name.clone(),
        is_abstract: false,
        is_final: true,  // Anonymous classes are implicitly final
        is_readonly: false,
        parent: parent.clone(),
        interfaces: interfaces.to_vec(),
        traits: traits.to_vec(),
        properties: properties.to_vec(),
        methods: methods.to_vec(),
        constants: constants.to_vec(),
    };
    
    // Register class temporarily
    self.register_class(&class_def)?;
    
    // Validate inheritance if parent specified
    if let Some(parent_name) = parent {
        let parent_class = self.get_class(parent_name)?;
        if parent_class.is_final {
            return Err(format!(
                "Anonymous class cannot extend final class {}",
                parent_name
            ));
        }
    }
    
    // Validate interface implementation
    for iface in interfaces {
        self.validate_interface_implementation(&class_def, iface)?;
    }
    
    // Evaluate constructor arguments
    let args: Vec<Value> = constructor_args
        .iter()
        .map(|arg| self.evaluate(arg))
        .collect::<Result<_, _>>()?;
    
    // Create instance
    let mut instance = Object::new(&class_name);
    
    // Initialize properties
    for prop in properties {
        if let Some(ref default) = prop.default {
            let value = self.evaluate(default)?;
            instance.set_property(&prop.name, value);
        }
    }
    
    // Inherit parent properties
    if let Some(parent_name) = parent {
        self.inherit_properties(&mut instance, parent_name)?;
    }
    
    // Call constructor if present
    if let Some(constructor) = class_def.get_constructor() {
        self.call_constructor(&mut instance, constructor, args)?;
    } else if !args.is_empty() {
        return Err("Anonymous class has no constructor but arguments were passed".to_string());
    }
    
    Ok(Value::Object(Rc::new(RefCell::new(instance))))
}
```

### Step 5: Handle get_class() for Anonymous Classes

```rust
fn builtin_get_class(&mut self, args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Object(obj)) => {
            let obj_ref = obj.borrow();
            // Return the internal name (includes class@anonymous)
            Ok(Value::String(obj_ref.class_name.clone()))
        }
        _ => Err("get_class() expects an object".to_string()),
    }
}
```

### Step 6: Anonymous Classes are Implicitly Final

In validation:

```rust
fn validate_class_inheritance(&self, child: &str, parent: &str) -> Result<(), String> {
    let parent_class = self.get_class(parent)?;
    
    // Anonymous classes can't be extended (implicitly final)
    if parent_class.name.starts_with("class@anonymous") {
        return Err("Cannot extend anonymous class".to_string());
    }
    
    if parent_class.is_final {
        return Err(format!("Cannot extend final class {}", parent));
    }
    
    Ok(())
}
```

### Step 7: Add Tests

**tests/classes/anonymous_basic.vhpt**
```
--TEST--
Basic anonymous class
--FILE--
<?php
$obj = new class {
    public function greet() {
        return "Hello!";
    }
};

echo $obj->greet();
--EXPECT--
Hello!
```

**tests/classes/anonymous_with_constructor.vhpt**
```
--TEST--
Anonymous class with constructor arguments
--FILE--
<?php
$obj = new class("World") {
    private $name;
    
    public function __construct($name) {
        $this->name = $name;
    }
    
    public function greet() {
        return "Hello, " . $this->name;
    }
};

echo $obj->greet();
--EXPECT--
Hello, World
```

**tests/classes/anonymous_extends.vhpt**
```
--TEST--
Anonymous class extending another class
--FILE--
<?php
class Base {
    protected $value;
    
    public function __construct($value) {
        $this->value = $value;
    }
}

$obj = new class(42) extends Base {
    public function getValue() {
        return $this->value;
    }
};

echo $obj->getValue();
--EXPECT--
42
```

**tests/classes/anonymous_implements.vhpt**
```
--TEST--
Anonymous class implementing interface
--FILE--
<?php
interface Printable {
    public function toString();
}

$obj = new class implements Printable {
    public function toString() {
        return "I am printable";
    }
};

echo $obj->toString();
--EXPECT--
I am printable
```

**tests/classes/anonymous_extends_implements.vhpt**
```
--TEST--
Anonymous class with extends and implements
--FILE--
<?php
class Counter {
    protected $count = 0;
}

interface Incrementable {
    public function increment();
}

$obj = new class extends Counter implements Incrementable {
    public function increment() {
        return ++$this->count;
    }
};

echo $obj->increment() . "\n";
echo $obj->increment() . "\n";
echo $obj->increment();
--EXPECT--
1
2
3
```

**tests/classes/anonymous_get_class.vhpt**
```
--TEST--
get_class on anonymous class returns special name
--FILE--
<?php
$obj = new class {};
$name = get_class($obj);
echo strpos($name, "class@anonymous") !== false ? "yes" : "no";
--EXPECT--
yes
```

**tests/classes/anonymous_with_trait.vhpt**
```
--TEST--
Anonymous class using trait
--FILE--
<?php
trait Greetable {
    public function greet() {
        return "Hello from trait!";
    }
}

$obj = new class {
    use Greetable;
};

echo $obj->greet();
--EXPECT--
Hello from trait!
```

**tests/classes/anonymous_implicitly_final.vhpt**
```
--TEST--
Anonymous classes cannot be referenced for inheritance
--DESCRIPTION--
This is more of a practical test - since anonymous classes have no name,
they can't be referenced in an extends clause.
--FILE--
<?php
$obj = new class {
    public function test() { return "test"; }
};
// We can't write: class X extends <anonymous> {}
// So this just tests that the anonymous class works
echo $obj->test();
--EXPECT--
test
```

**tests/classes/anonymous_multiple.vhpt**
```
--TEST--
Multiple anonymous classes are independent
--FILE--
<?php
$a = new class {
    public $value = "A";
};

$b = new class {
    public $value = "B";
};

echo $a->value . "\n";
echo $b->value . "\n";
echo get_class($a) === get_class($b) ? "same" : "different";
--EXPECT--
A
B
different
```

**tests/classes/anonymous_nested_in_method.vhpt**
```
--TEST--
Anonymous class created inside method
--FILE--
<?php
class Factory {
    public function createGreeter($name) {
        return new class($name) {
            private $name;
            public function __construct($name) {
                $this->name = $name;
            }
            public function greet() {
                return "Hello, " . $this->name;
            }
        };
    }
}

$factory = new Factory();
$greeter = $factory->createGreeter("PHP");
echo $greeter->greet();
--EXPECT--
Hello, PHP
```

## PHP Compatibility Notes

| Feature | PHP Version |
|---------|-------------|
| Anonymous classes | 7.0 |

## Key Rules

1. **Implicit final**: Anonymous classes cannot be extended
2. **Naming**: Internal name is `class@anonymous<filepath>:<line>$<id>`
3. **No standalone definition**: Must use `new class ...` syntax
4. **Full class features**: Supports extends, implements, traits, properties, methods
5. **Constructor arguments**: Passed in `new class(args)` syntax
6. **Scope**: Can reference outer scope variables if passed to constructor

## Implementation Order

1. AST structure for anonymous class expression
2. Parse `new class` syntax
3. Basic anonymous class evaluation
4. Anonymous class with constructor
5. Anonymous class extends
6. Anonymous class implements
7. Anonymous class with traits
8. get_class() support

## Error Messages

- `Anonymous class cannot extend final class X`
- `Anonymous class has no constructor but arguments were passed`
- `Cannot extend anonymous class`

## Considerations

1. **Memory**: Each instantiation creates a new class definition
2. **Comparison**: Each anonymous class is a unique type
3. **Reflection**: Should support get_class(), instanceof
4. **Serialization**: Anonymous classes cannot be serialized (PHP limitation)
