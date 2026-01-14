# Generator Execution Support

## Status: Planned

## Overview

Implement full generator execution with `yield`, `yield from`, generator return values, and the Generator object interface (send, throw, getReturn, valid, current, key, next, rewind).

## Current Status

Generators can be parsed and compiled, but they currently only return a placeholder Generator object. The yield syntax is recognized but not fully executed.

## Background

Generators provide a simple way to iterate through data without needing to build an array in memory. They were introduced in PHP 5.5 and are widely used in:
- Processing large datasets
- Implementing iterators
- Asynchronous programming patterns
- Stream processing

## Requirements

### Core Generator Functionality

1. **Generator Object Creation**
   - Create `Generator` class in runtime
   - Implement Iterator interface (current, key, next, rewind, valid)
   - Implement Generator-specific methods (send, throw, getReturn)

2. **Yield Execution**
   - Pause function execution at yield
   - Return value to caller
   - Resume execution on next iteration
   - Preserve local variable state between yields

3. **Yield with Keys**
   ```php
   yield $key => $value;
   ```
   - Return key-value pairs
   - Support in foreach: `foreach ($gen as $key => $value)`

4. **Yield From Delegation**
   ```php
   yield from $iterable;
   ```
   - Delegate to another generator/array/iterator
   - Forward values from yielded generator
   - Forward return values from delegated generator

5. **Generator Return Values** (PHP 7.0)
   ```php
   function gen() {
       yield 1;
       yield 2;
       return "final";
   }
   $gen = gen();
   $gen->getReturn(); // "final"
   ```

6. **Send Method**
   ```php
   $gen->send($value); // Send value back to generator
   // Inside generator:
   $received = yield;
   ```

7. **Throw Method**
   ```php
   $gen->send($exception); // Throw exception into generator
   // Inside generator:
   try {
       $value = yield;
   } catch (Exception $e) {
       // Handle exception
   }
   ```

### VM Integration

1. **Generator State Management**
   - Add generator state to VM's call frame system
   - Track generator's current position
   - Save/restore execution context

2. **Yield Opcode**
   - Add `YIELD` and `YIELD_FROM` opcodes
   - Implement yield execution in VM loop
   - Handle generator suspension/resumption

3. **Context Preservation**
   - Save local variables on yield
   - Restore variables on resume
   - Track loop contexts across yields

## Implementation Plan

### Phase 1: Generator Object Runtime

**File:** `src/runtime/generators.rs` (new)

```rust
pub struct Generator {
    state: GeneratorState,
    current: Option<Value>,
    key: Option<Value>,
    return_value: Option<Value>,
    // Execution context (saved frame)
}

pub enum GeneratorState {
    Created,
    Running,
    Suspended,
    Closed,
}
```

**Tasks:**
- [ ] Create Generator struct
- [ ] Implement Iterator interface methods
- [ ] Implement send() method
- [ ] Implement throw() method
- [ ] Implement getReturn() method

### Phase 2: Opcodes

**File:** `src/vm/opcode.rs`

Add opcodes:
```rust
YIELD,           // Yield a value (pause generator)
YIELD_FROM,      // Delegate to another iterable
GEN_CREATE,      // Create generator object
```

**Tasks:**
- [ ] Add YIELD opcode
- [ ] Add YIELD_FROM opcode
- [ ] Add GEN_CREATE opcode
- [ ] Document opcodes

### Phase 3: Compiler Support

**File:** `src/vm/compiler/functions.rs` (existing)

Add compilation for yield statements:

```rust
fn compile_yield(&mut self, expr: &Expr) -> Result<(), String> {
    // Compile value expression
    self.compile_expr(expr)?;

    // Emit yield opcode
    self.emit(Opcode::YIELD);

    Ok(())
}
```

**Tasks:**
- [ ] Detect generator functions (contain yield)
- [ ] Compile yield expressions
- [ ] Compile yield from expressions
- [ ] Handle generator return values
- [ ] Mark functions as generators in AST

### Phase 4: VM Execution

**File:** `src/vm/mod.rs` (existing)

Implement yield execution:

```rust
Opcode::YIELD => {
    let value = self.stack.pop();
    // Save execution context
    // Return value to caller
    // Suspend generator
}
```

**Tasks:**
- [ ] Implement YIELD execution
- [ ] Implement YIELD_FROM execution
- [ ] Handle generator resume (next/send)
- [ ] Handle generator throw
- [ ] Track generator state
- [ ] Preserve local variables

### Phase 5: foreach Integration

**File:** `src/vm/compiler/loops.rs` (existing)

Update foreach to work with generators:

**Tasks:**
- [ ] Detect generators in foreach
- [ ] Call generator's next() method
- [ ] Extract key/value from generator
- [ ] Handle generator exhaustion

### Phase 6: Tests

**File:** `tests/generators/` (new directory)

Test coverage:
- Basic yield
- Yield with keys
- Yield from arrays
- Yield from other generators
- Nested generators
- Generator return values
- send() method
- throw() method
- getReturn() method
- Rewind behavior
- foreach with generators

**Example test:**
```
--TEST--
Basic yield
--FILE--
<?php
function gen() {
    yield 1;
    yield 2;
    yield 3;
}

foreach (gen() as $value) {
    echo $value . "\n";
}
--EXPECT--
1
2
3
```

## Dependencies

- Existing AST for function declarations
- Existing call frame system
- Existing foreach implementation

## Testing Strategy

1. **Unit Tests**: Generator object methods (send, throw, getReturn)
2. **Integration Tests**: foreach with generators
3. **Edge Cases**: Empty generators, exceptions, nested yields
4. **Compatibility Tests**: Match PHP 8.x generator behavior

## Success Criteria

- All yield syntax works correctly
- Generators can be iterated with foreach
- send() and throw() methods work as expected
- Generator return values are accessible
- yield from properly delegates
- All tests pass
- Performance is acceptable for typical generator use cases

## Performance Considerations

- Minimize memory overhead for generator state
- Efficiently save/restore execution context
- Optimize yield from delegation
- Avoid unnecessary copying of values

## Open Questions

- Should generators support weak references for cleanup?
- How to handle generators across multiple VM invocations?

## References

- PHP Generator documentation: https://www.php.net/manual/en/language.generators.overview.php
- Generator RFC: https://wiki.php.net/rfc/generators
- PHP 7.0 Generator return values: https://wiki.php.net/rfc/generator-delegation

## Related Plans

- Iterator Interface (future)
- IteratorAggregate (future)
- ArrayAccess interface (future)
