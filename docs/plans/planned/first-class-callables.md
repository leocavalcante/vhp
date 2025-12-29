# Plan: First-Class Callables

## Overview

PHP 8.1 introduced first-class callable syntax, allowing you to create closures from existing functions/methods using the `...` syntax. This is also used for the pipe operator.

**PHP Example:**
```php
<?php
// Create closure from function name
$len = strlen(...);
echo $len("hello");  // 5

// Create closure from method
class Formatter {
    public function format(string $s): string {
        return strtoupper($s);
    }
    
    public static function static_format(string $s): string {
        return strtolower($s);
    }
}

$formatter = new Formatter();

// Instance method
$format = $formatter->format(...);
echo $format("hello");  // HELLO

// Static method
$staticFormat = Formatter::static_format(...);
echo $staticFormat("HELLO");  // hello

// Used with pipe operator
$result = "  hello  " 
    |> trim(...)
    |> strtoupper(...);
echo $result;  // HELLO
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/ast/expr.rs` | Add `Closure` from callable expressions |
| `src/parser/expr/mod.rs` | Parse first-class callable syntax |
| `src/interpreter/expr_eval/mod.rs` | Create closures from callables |
| `tests/functions/first_class_*.vhpt` | Test files |

Note: Uses existing `Ellipsis` token from variadic functions plan.

## Implementation Steps

### Step 1: Add Callable Closure Expression (`src/ast/expr.rs`)

```rust
#[derive(Debug, Clone)]
pub enum Expr {
    // ... existing variants ...
    
    /// First-class callable: functionName(...)
    CallableFromFunction(String),
    
    /// First-class callable from method: $obj->method(...)
    CallableFromMethod {
        object: Box<Expr>,
        method: String,
    },
    
    /// First-class callable from static method: Class::method(...)
    CallableFromStaticMethod {
        class: String,
        method: String,
    },
}
```

### Step 2: Parse First-Class Callable Syntax (`src/parser/expr/mod.rs`)

Modify function call parsing to detect `func(...)` syntax:

```rust
fn parse_function_call(&mut self, name: String) -> Result<Expr, String> {
    self.expect(&TokenKind::LeftParen)?;
    
    // Check for first-class callable: func(...)
    if self.check(&TokenKind::Ellipsis) {
        self.advance();
        self.expect(&TokenKind::RightParen)?;
        return Ok(Expr::CallableFromFunction(name));
    }
    
    // Regular function call with arguments
    let args = self.parse_argument_list()?;
    self.expect(&TokenKind::RightParen)?;
    
    Ok(Expr::FunctionCall { name, args })
}
```

### Step 3: Parse Method First-Class Callable

Modify method call parsing:

```rust
fn parse_method_call(&mut self, object: Expr) -> Result<Expr, String> {
    self.expect(&TokenKind::Arrow)?;  // ->
    let method = self.expect_identifier()?;
    
    self.expect(&TokenKind::LeftParen)?;
    
    // Check for first-class callable: $obj->method(...)
    if self.check(&TokenKind::Ellipsis) {
        self.advance();
        self.expect(&TokenKind::RightParen)?;
        return Ok(Expr::CallableFromMethod {
            object: Box::new(object),
            method,
        });
    }
    
    let args = self.parse_argument_list()?;
    self.expect(&TokenKind::RightParen)?;
    
    Ok(Expr::MethodCall {
        object: Box::new(object),
        method,
        args,
    })
}
```

### Step 4: Parse Static Method First-Class Callable

Modify static method call parsing:

```rust
fn parse_static_call(&mut self, class: String) -> Result<Expr, String> {
    self.expect(&TokenKind::DoubleColon)?;  // ::
    let method = self.expect_identifier()?;
    
    self.expect(&TokenKind::LeftParen)?;
    
    // Check for first-class callable: Class::method(...)
    if self.check(&TokenKind::Ellipsis) {
        self.advance();
        self.expect(&TokenKind::RightParen)?;
        return Ok(Expr::CallableFromStaticMethod {
            class,
            method,
        });
    }
    
    let args = self.parse_argument_list()?;
    self.expect(&TokenKind::RightParen)?;
    
    Ok(Expr::StaticMethodCall {
        class,
        method,
        args,
    })
}
```

### Step 5: Evaluate First-Class Callables (`src/interpreter/expr_eval/mod.rs`)

Create closures from the callables:

```rust
fn eval_first_class_callable(&mut self, expr: &Expr) -> Result<Value, String> {
    match expr {
        Expr::CallableFromFunction(name) => {
            // Check if function exists
            if !self.function_exists(name) && !self.builtin_exists(name) {
                return Err(format!("Function {} does not exist", name));
            }
            
            // Create a closure that calls the function
            let closure = Closure {
                params: self.get_function_params(name)?,
                return_type: self.get_function_return_type(name),
                body: ClosureBody::FunctionRef(name.clone()),
                captured_vars: HashMap::new(),
            };
            
            Ok(Value::Closure(Rc::new(closure)))
        }
        
        Expr::CallableFromMethod { object, method } => {
            let obj_value = self.evaluate(object)?;
            
            // Verify method exists
            if let Value::Object(ref obj) = obj_value {
                let class = self.get_class(&obj.borrow().class_name)?;
                if !class.has_method(method) {
                    return Err(format!(
                        "Method {}::{} does not exist",
                        obj.borrow().class_name,
                        method
                    ));
                }
            } else {
                return Err("Cannot create callable from non-object".to_string());
            }
            
            // Create closure bound to this object and method
            let closure = Closure {
                params: vec![], // Will be filled from method signature
                return_type: None,
                body: ClosureBody::MethodRef {
                    object: obj_value,
                    method: method.clone(),
                },
                captured_vars: HashMap::new(),
            };
            
            Ok(Value::Closure(Rc::new(closure)))
        }
        
        Expr::CallableFromStaticMethod { class, method } => {
            // Verify static method exists
            let class_def = self.get_class(class)?;
            if !class_def.has_static_method(method) {
                return Err(format!(
                    "Static method {}::{} does not exist",
                    class, method
                ));
            }
            
            let closure = Closure {
                params: vec![],
                return_type: None,
                body: ClosureBody::StaticMethodRef {
                    class: class.clone(),
                    method: method.clone(),
                },
                captured_vars: HashMap::new(),
            };
            
            Ok(Value::Closure(Rc::new(closure)))
        }
        
        _ => Err("Not a first-class callable expression".to_string()),
    }
}
```

### Step 6: Update ClosureBody Enum

Extend the closure body to support callable references:

```rust
#[derive(Debug, Clone)]
pub enum ClosureBody {
    /// Regular closure with statements
    Block(Vec<Stmt>),
    /// Arrow function with expression
    Expression(Expr),
    /// Reference to named function
    FunctionRef(String),
    /// Reference to instance method
    MethodRef {
        object: Value,
        method: String,
    },
    /// Reference to static method
    StaticMethodRef {
        class: String,
        method: String,
    },
}
```

### Step 7: Update Closure Calling

When calling a closure, check its body type:

```rust
fn call_closure(
    &mut self,
    closure: &Closure,
    args: Vec<Value>,
) -> Result<Value, String> {
    match &closure.body {
        ClosureBody::FunctionRef(name) => {
            // Forward to function call
            self.call_function_by_name(name, args)
        }
        
        ClosureBody::MethodRef { object, method } => {
            // Forward to method call
            if let Value::Object(obj) = object {
                self.call_instance_method(obj.clone(), method, args)
            } else {
                Err("Invalid method reference".to_string())
            }
        }
        
        ClosureBody::StaticMethodRef { class, method } => {
            // Forward to static method call
            self.call_static_method(class, method, args)
        }
        
        ClosureBody::Block(stmts) => {
            // Execute closure body
            self.execute_closure_block(closure, stmts, args)
        }
        
        ClosureBody::Expression(expr) => {
            // Execute arrow function
            self.execute_arrow_closure(closure, expr, args)
        }
    }
}
```

### Step 8: Integration with Pipe Operator

The existing pipe operator implementation should already work with first-class callables since they produce `Closure` values. Update pipe operator evaluation to handle the `...` syntax:

```rust
fn eval_pipe(&mut self, left: &Expr, right: &Expr) -> Result<Value, String> {
    let left_value = self.evaluate(left)?;
    
    // Check if right side is a first-class callable call
    match right {
        // e.g., $x |> trim(...)
        Expr::CallableFromFunction(name) => {
            // Call function with piped value as first arg
            self.call_function_by_name(name, vec![left_value])
        }
        
        // e.g., $x |> strtoupper(..., additional_args)
        // Actually this would be Expr::FunctionCall with Spread
        Expr::FunctionCall { name, args } => {
            // Prepend piped value to args
            let mut all_args = vec![left_value];
            for arg in args {
                all_args.push(self.evaluate(arg)?);
            }
            self.call_function_by_name(name, all_args)
        }
        
        _ => {
            // Evaluate right side as callable and invoke
            let callable = self.evaluate(right)?;
            self.call_value(callable, vec![left_value])
        }
    }
}
```

### Step 9: Add Tests

**tests/functions/first_class_function.vhpt**
```
--TEST--
First-class callable from function
--FILE--
<?php
$len = strlen(...);
echo $len("hello");
--EXPECT--
5
```

**tests/functions/first_class_user_function.vhpt**
```
--TEST--
First-class callable from user function
--FILE--
<?php
function double($n) {
    return $n * 2;
}

$fn = double(...);
echo $fn(21);
--EXPECT--
42
```

**tests/functions/first_class_method.vhpt**
```
--TEST--
First-class callable from instance method
--FILE--
<?php
class Formatter {
    private $prefix;
    
    public function __construct($prefix) {
        $this->prefix = $prefix;
    }
    
    public function format($s) {
        return $this->prefix . $s;
    }
}

$f = new Formatter("Hello, ");
$greet = $f->format(...);
echo $greet("World");
--EXPECT--
Hello, World
```

**tests/functions/first_class_static.vhpt**
```
--TEST--
First-class callable from static method
--FILE--
<?php
class Utils {
    public static function upper($s) {
        return strtoupper($s);
    }
}

$upper = Utils::upper(...);
echo $upper("hello");
--EXPECT--
HELLO
```

**tests/functions/first_class_with_pipe.vhpt**
```
--TEST--
First-class callable with pipe operator
--FILE--
<?php
$result = "  hello  " 
    |> trim(...)
    |> strtoupper(...);
echo $result;
--EXPECT--
HELLO
```

**tests/functions/first_class_stored.vhpt**
```
--TEST--
First-class callable stored in variable
--FILE--
<?php
$funcs = [
    'trim' => trim(...),
    'upper' => strtoupper(...),
];

$text = "  hello  ";
$text = $funcs['trim']($text);
$text = $funcs['upper']($text);
echo $text;
--EXPECT--
HELLO
```

**tests/functions/first_class_passed.vhpt**
```
--TEST--
First-class callable passed to function
--FILE--
<?php
function apply($value, $func) {
    return $func($value);
}

echo apply("hello", strtoupper(...));
--EXPECT--
HELLO
```

**tests/functions/first_class_nonexistent_error.vhpt**
```
--TEST--
Error on non-existent function
--FILE--
<?php
$fn = nonexistent_function(...);
--EXPECT_ERROR--
does not exist
```

**tests/functions/first_class_method_preserves_this.vhpt**
```
--TEST--
First-class method preserves $this context
--FILE--
<?php
class Counter {
    private $count = 0;
    
    public function increment() {
        return ++$this->count;
    }
}

$counter = new Counter();
$inc = $counter->increment(...);

echo $inc() . "\n";
echo $inc() . "\n";
echo $inc();
--EXPECT--
1
2
3
```

## PHP Compatibility Notes

| Feature | PHP Version |
|---------|-------------|
| First-class callables | 8.1 |
| Used with pipe operator | 8.5 |

## Key Rules

1. **Syntax**: `functionName(...)` creates a closure
2. **Validation**: Function/method must exist at creation time
3. **Context preserved**: `$this` is bound for instance methods
4. **Static preserved**: Static methods maintain static context
5. **Can be stored**: Result is a regular `Closure` object

## Implementation Order

1. Parser changes for `func(...)`
2. Parser changes for `$obj->method(...)`
3. Parser changes for `Class::method(...)`
4. ClosureBody variants
5. Closure creation from callables
6. Closure invocation
7. Pipe operator integration
8. Tests

## Error Messages

- `Function X does not exist`
- `Method X::Y does not exist`
- `Cannot create callable from non-object`

## Relationship with Other Features

- **Depends on**: Closures/arrow functions (for Closure value type)
- **Enhances**: Pipe operator (provides clean syntax)
- **Used with**: array_map, array_filter, etc.
