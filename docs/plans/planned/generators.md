# Plan: Generators (yield)

## Overview

Generators are functions that can be paused and resumed, allowing iteration without building an entire array in memory. They use the `yield` keyword to produce values one at a time.

**PHP Example:**
```php
<?php
// Basic generator
function numbers() {
    yield 1;
    yield 2;
    yield 3;
}

foreach (numbers() as $n) {
    echo $n . "\n";
}

// Generator with keys
function pairs() {
    yield "a" => 1;
    yield "b" => 2;
}

// Generator delegation (PHP 7.0+)
function all() {
    yield from [1, 2, 3];
    yield from numbers();
}

// Generator return value (PHP 7.0+)
function task() {
    yield 1;
    yield 2;
    return "done";
}

$gen = task();
foreach ($gen as $v) { echo $v; }
echo $gen->getReturn(); // "done"
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/token.rs` | Add `Yield` token |
| `src/ast/expr.rs` | Add `Yield` and `YieldFrom` expressions |
| `src/ast/stmt.rs` | Detect generator functions |
| `src/interpreter/value.rs` | Add `Generator` value type |
| `src/interpreter/mod.rs` | Generator execution engine |
| `tests/generators/*.vhpt` | Test files |

## Implementation Steps

### Step 1: Add Token (`src/token.rs`)

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ... existing tokens ...
    
    Yield,
    
    // ... rest of tokens ...
}
```

### Step 2: Update Lexer (`src/lexer/mod.rs`)

```rust
fn tokenize_identifier(&mut self) -> TokenKind {
    match ident.to_lowercase().as_str() {
        // ... existing keywords ...
        "yield" => TokenKind::Yield,
        // ...
    }
}
```

### Step 3: Add Yield Expressions (`src/ast/expr.rs`)

```rust
#[derive(Debug, Clone)]
pub enum Expr {
    // ... existing variants ...
    
    /// yield expression: yield $value or yield $key => $value
    Yield {
        key: Option<Box<Expr>>,
        value: Option<Box<Expr>>,
    },
    
    /// yield from expression (PHP 7.0+): yield from $iterable
    YieldFrom(Box<Expr>),
}
```

### Step 4: Parse Yield Expressions (`src/parser/expr/mod.rs`)

```rust
fn parse_yield(&mut self) -> Result<Expr, String> {
    self.expect(&TokenKind::Yield)?;
    
    // Check for `yield from`
    if self.check_identifier("from") {
        self.advance(); // consume 'from'
        let iterable = self.parse_expression()?;
        return Ok(Expr::YieldFrom(Box::new(iterable)));
    }
    
    // Check for bare yield (no value)
    if self.is_statement_end() {
        return Ok(Expr::Yield {
            key: None,
            value: None,
        });
    }
    
    // Parse value (and maybe key)
    let first = self.parse_expression()?;
    
    // Check for yield $key => $value
    if self.check(&TokenKind::DoubleArrow) {
        self.advance();
        let value = self.parse_expression()?;
        return Ok(Expr::Yield {
            key: Some(Box::new(first)),
            value: Some(Box::new(value)),
        });
    }
    
    // Just yield $value
    Ok(Expr::Yield {
        key: None,
        value: Some(Box::new(first)),
    })
}
```

Integrate into expression parsing:

```rust
fn parse_primary(&mut self) -> Result<Expr, String> {
    match &self.current_token().kind {
        TokenKind::Yield => self.parse_yield(),
        // ... other cases ...
    }
}
```

### Step 5: Mark Functions as Generators

A function is a generator if it contains `yield`:

```rust
/// Check if function body contains yield
fn is_generator_function(body: &[Stmt]) -> bool {
    struct YieldVisitor {
        found: bool,
    }
    
    impl YieldVisitor {
        fn visit_stmt(&mut self, stmt: &Stmt) {
            match stmt {
                Stmt::Expression(expr) => self.visit_expr(expr),
                Stmt::Return(Some(expr)) => self.visit_expr(expr),
                Stmt::If { condition, then_branch, else_branch, .. } => {
                    self.visit_expr(condition);
                    for s in then_branch { self.visit_stmt(s); }
                    if let Some(branch) = else_branch {
                        for s in branch { self.visit_stmt(s); }
                    }
                }
                // ... visit all statement types ...
                _ => {}
            }
        }
        
        fn visit_expr(&mut self, expr: &Expr) {
            match expr {
                Expr::Yield { .. } | Expr::YieldFrom(_) => {
                    self.found = true;
                }
                // Don't traverse into nested functions/closures
                Expr::ArrowFunction { .. } => {}
                // ... visit child expressions ...
                _ => {}
            }
        }
    }
    
    let mut visitor = YieldVisitor { found: false };
    for stmt in body {
        visitor.visit_stmt(stmt);
        if visitor.found { return true; }
    }
    false
}
```

### Step 6: Add Generator Value Type (`src/interpreter/value.rs`)

```rust
use std::rc::Rc;
use std::cell::RefCell;

/// Generator state
#[derive(Debug, Clone)]
pub struct Generator {
    /// Generator function name (for error messages)
    pub function_name: String,
    /// Function parameters
    pub params: Vec<FunctionParam>,
    /// Function body
    pub body: Vec<Stmt>,
    /// Captured scope variables
    pub scope: HashMap<String, Value>,
    /// Current execution state
    pub state: Rc<RefCell<GeneratorState>>,
}

#[derive(Debug, Clone)]
pub enum GeneratorState {
    /// Not started yet
    Initial,
    /// Suspended at a yield (with instruction pointer and local scope)
    Suspended {
        position: GeneratorPosition,
        locals: HashMap<String, Value>,
    },
    /// Completed normally
    Completed {
        return_value: Value,
    },
    /// Closed/error
    Closed,
}

#[derive(Debug, Clone)]
pub struct GeneratorPosition {
    /// Statement index in function body
    pub stmt_index: usize,
    /// Optional nested position for loops, etc.
    pub nested: Option<Box<GeneratorPosition>>,
}

impl Generator {
    pub fn is_finished(&self) -> bool {
        matches!(
            &*self.state.borrow(),
            GeneratorState::Completed { .. } | GeneratorState::Closed
        )
    }
    
    pub fn get_return(&self) -> Result<Value, String> {
        match &*self.state.borrow() {
            GeneratorState::Completed { return_value } => Ok(return_value.clone()),
            _ => Err("Cannot get return value of a generator that has not returned".to_string()),
        }
    }
}
```

### Step 7: Implement Generator Execution (`src/interpreter/mod.rs`)

```rust
/// Execute generator and return next yielded value
fn generator_next(&mut self, gen: &Generator) -> Result<Option<(Value, Value)>, String> {
    let mut state = gen.state.borrow_mut();
    
    match &*state {
        GeneratorState::Initial => {
            // Start execution
            drop(state);
            self.execute_generator_body(gen, 0)
        }
        GeneratorState::Suspended { position, locals } => {
            // Resume execution
            let pos = position.clone();
            let locals = locals.clone();
            drop(state);
            self.resume_generator(gen, pos, locals)
        }
        GeneratorState::Completed { .. } | GeneratorState::Closed => {
            Ok(None) // Iterator exhausted
        }
    }
}

/// Execute generator body until yield or completion
fn execute_generator_body(
    &mut self,
    gen: &Generator,
    start_index: usize,
) -> Result<Option<(Value, Value)>, String> {
    // Set up generator scope
    self.push_scope();
    
    // Add captured variables
    for (name, value) in &gen.scope {
        self.set_variable(name, value.clone());
    }
    
    // Execute statements
    let mut index = start_index;
    let mut auto_key: i64 = 0;
    
    while index < gen.body.len() {
        match self.execute_stmt_in_generator(&gen.body[index], &mut auto_key)? {
            GeneratorExecResult::Continue => {
                index += 1;
            }
            GeneratorExecResult::Yield { key, value } => {
                // Save state
                let locals = self.current_scope().clone();
                *gen.state.borrow_mut() = GeneratorState::Suspended {
                    position: GeneratorPosition {
                        stmt_index: index + 1,
                        nested: None,
                    },
                    locals,
                };
                self.pop_scope();
                return Ok(Some((key, value)));
            }
            GeneratorExecResult::Return(value) => {
                *gen.state.borrow_mut() = GeneratorState::Completed {
                    return_value: value,
                };
                self.pop_scope();
                return Ok(None);
            }
        }
    }
    
    // Implicit return null
    *gen.state.borrow_mut() = GeneratorState::Completed {
        return_value: Value::Null,
    };
    self.pop_scope();
    Ok(None)
}

enum GeneratorExecResult {
    Continue,
    Yield { key: Value, value: Value },
    Return(Value),
}

fn execute_stmt_in_generator(
    &mut self,
    stmt: &Stmt,
    auto_key: &mut i64,
) -> Result<GeneratorExecResult, String> {
    match stmt {
        Stmt::Expression(expr) => {
            match self.eval_generator_expr(expr, auto_key)? {
                GeneratorExprResult::Value(_) => Ok(GeneratorExecResult::Continue),
                GeneratorExprResult::Yield { key, value } => {
                    Ok(GeneratorExecResult::Yield { key, value })
                }
            }
        }
        Stmt::Return(expr) => {
            let value = if let Some(e) = expr {
                self.evaluate(e)?
            } else {
                Value::Null
            };
            Ok(GeneratorExecResult::Return(value))
        }
        // Handle other statements...
        _ => {
            self.execute_stmt(stmt)?;
            Ok(GeneratorExecResult::Continue)
        }
    }
}

enum GeneratorExprResult {
    Value(Value),
    Yield { key: Value, value: Value },
}

fn eval_generator_expr(
    &mut self,
    expr: &Expr,
    auto_key: &mut i64,
) -> Result<GeneratorExprResult, String> {
    match expr {
        Expr::Yield { key, value } => {
            let yield_key = if let Some(k) = key {
                self.evaluate(k)?
            } else {
                let k = *auto_key;
                *auto_key += 1;
                Value::Integer(k)
            };
            
            let yield_value = if let Some(v) = value {
                self.evaluate(v)?
            } else {
                Value::Null
            };
            
            Ok(GeneratorExprResult::Yield {
                key: yield_key,
                value: yield_value,
            })
        }
        Expr::YieldFrom(iterable) => {
            // Delegate to another iterable
            let iter_value = self.evaluate(iterable)?;
            // This needs special handling - might yield multiple times
            // For simplicity, could iterate and re-yield
            todo!("yield from requires iterating the sub-generator")
        }
        _ => {
            let value = self.evaluate(expr)?;
            Ok(GeneratorExprResult::Value(value))
        }
    }
}
```

### Step 8: Integrate with Foreach

Update foreach to handle generators:

```rust
fn execute_foreach(
    &mut self,
    iterable: &Expr,
    key_var: &Option<String>,
    value_var: &str,
    body: &[Stmt],
) -> Result<ControlFlow, String> {
    let iter_value = self.evaluate(iterable)?;
    
    match iter_value {
        Value::Array(arr) => {
            // ... existing array iteration ...
        }
        Value::Generator(gen) => {
            loop {
                match self.generator_next(&gen)? {
                    Some((key, value)) => {
                        if let Some(kv) = key_var {
                            self.set_variable(kv, key);
                        }
                        self.set_variable(value_var, value);
                        
                        for stmt in body {
                            match self.execute_stmt(stmt)? {
                                ControlFlow::Break => return Ok(ControlFlow::Normal),
                                ControlFlow::Continue => break,
                                ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                                ControlFlow::Normal => {}
                            }
                        }
                    }
                    None => break,
                }
            }
            Ok(ControlFlow::Normal)
        }
        _ => Err("foreach requires array or iterable".to_string()),
    }
}
```

### Step 9: Generator Methods

Generators have these methods:

```rust
// When calling method on generator
fn call_generator_method(
    &mut self,
    gen: &Generator,
    method: &str,
    args: Vec<Value>,
) -> Result<Value, String> {
    match method.to_lowercase().as_str() {
        "current" => {
            // Return current value without advancing
            todo!()
        }
        "key" => {
            // Return current key
            todo!()
        }
        "next" => {
            // Advance and return
            match self.generator_next(gen)? {
                Some((_, value)) => Ok(value),
                None => Ok(Value::Null),
            }
        }
        "rewind" => {
            // Rewind to beginning (only valid before first yield)
            todo!()
        }
        "valid" => {
            Ok(Value::Bool(!gen.is_finished()))
        }
        "getreturn" => {
            gen.get_return()
        }
        "send" => {
            // Send value into generator (received as yield expression result)
            todo!()
        }
        "throw" => {
            // Throw exception into generator
            todo!()
        }
        _ => Err(format!("Call to undefined method Generator::{}", method)),
    }
}
```

### Step 10: Add Tests

**tests/generators/basic_generator.vhpt**
```
--TEST--
Basic generator function
--FILE--
<?php
function numbers() {
    yield 1;
    yield 2;
    yield 3;
}

foreach (numbers() as $n) {
    echo $n . "\n";
}
--EXPECT--
1
2
3
```

**tests/generators/generator_keys.vhpt**
```
--TEST--
Generator with keys
--FILE--
<?php
function pairs() {
    yield "a" => 1;
    yield "b" => 2;
}

foreach (pairs() as $key => $value) {
    echo "$key: $value\n";
}
--EXPECT--
a: 1
b: 2
```

**tests/generators/generator_auto_keys.vhpt**
```
--TEST--
Generator auto-increments keys
--FILE--
<?php
function nums() {
    yield "first";
    yield "second";
}

foreach (nums() as $key => $value) {
    echo "$key: $value\n";
}
--EXPECT--
0: first
1: second
```

**tests/generators/generator_return.vhpt**
```
--TEST--
Generator return value (PHP 7.0+)
--FILE--
<?php
function task() {
    yield 1;
    yield 2;
    return "completed";
}

$gen = task();
foreach ($gen as $v) {
    echo $v . "\n";
}
echo $gen->getReturn();
--EXPECT--
1
2
completed
```

**tests/generators/yield_from_array.vhpt**
```
--TEST--
yield from with array
--FILE--
<?php
function gen() {
    yield 1;
    yield from [2, 3, 4];
    yield 5;
}

foreach (gen() as $v) {
    echo $v . "\n";
}
--EXPECT--
1
2
3
4
5
```

**tests/generators/yield_from_generator.vhpt**
```
--TEST--
yield from with another generator
--FILE--
<?php
function inner() {
    yield "a";
    yield "b";
}

function outer() {
    yield "start";
    yield from inner();
    yield "end";
}

foreach (outer() as $v) {
    echo $v . "\n";
}
--EXPECT--
start
a
b
end
```

**tests/generators/generator_with_loop.vhpt**
```
--TEST--
Generator with loop
--FILE--
<?php
function range_gen($start, $end) {
    for ($i = $start; $i <= $end; $i++) {
        yield $i;
    }
}

foreach (range_gen(1, 5) as $n) {
    echo $n;
}
--EXPECT--
12345
```

**tests/generators/bare_yield.vhpt**
```
--TEST--
Yield without value
--FILE--
<?php
function signals() {
    yield;
    yield;
}

$count = 0;
foreach (signals() as $v) {
    $count++;
}
echo $count;
--EXPECT--
2
```

## PHP Compatibility Notes

| Feature | PHP Version |
|---------|-------------|
| Basic generators | 5.5 |
| `yield` expression (receive values) | 5.5 |
| `yield from` | 7.0 |
| Generator return values | 7.0 |
| `Generator::getReturn()` | 7.0 |

## Implementation Complexity

Generators are complex because they require:

1. **Suspending execution**: Save local state mid-function
2. **Resuming execution**: Restore state and continue
3. **State machine**: Track position within function body
4. **Nested yields**: Handle yields in loops and conditionals

### Simplified Approach

For initial implementation, consider:

1. Transform generator body to a state machine at parse time
2. Or use Rust's async/generators if available
3. Or implement a simple "collect all yields" for basic cases

### State Machine Transformation

Convert:
```php
function gen() {
    yield 1;
    $x = 2;
    yield $x;
}
```

To internally:
```
State 0: yield 1, goto State 1
State 1: $x = 2, yield $x, goto State 2
State 2: return null
```

## Key Considerations

1. **Memory efficiency**: Generators avoid building full arrays
2. **Lazy evaluation**: Values computed on-demand
3. **State preservation**: Local variables persist between yields
4. **Exception handling**: Exceptions can be thrown into generators
5. **Garbage collection**: Generators hold references to scope

## Implementation Order

1. Token and basic parsing
2. Simple generators (no loops in body)
3. Generators with loops
4. Generators with conditionals
5. yield from arrays
6. yield from generators
7. Generator::getReturn()
8. Generator methods (send, throw)
