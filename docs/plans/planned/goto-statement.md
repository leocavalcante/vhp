# goto Statement

## Status: Planned

## Overview

Implement the `goto` statement and labels for unconditional jumping within the same execution scope.

## Current Status

goto statements are not supported. Using `goto` will result in a parse error.

## Background

The goto statement was introduced in PHP 5.3. While controversial, it's used in:
- Breaking out of deeply nested loops
- State machine implementations
- Error handling without exceptions
- Code obfuscation (unfortunately)

goto jumps to a label within the same file and function scope.

## Requirements

### Syntax

1. **Labels**
   ```php
   label_name:
   // code
   ```

2. **goto Statement**
   ```php
   goto label_name;
   ```

### Rules and Restrictions

1. **Same Scope**
   - goto can only jump within the same function/method
   - Cannot jump into or out of functions, classes, or methods
   - Cannot jump into or out of control structures (if, for, while, foreach, switch)
   - Cannot jump into a try block

2. **Forward Jumps**
   ```php
   goto forward;
   // ... code ...
   forward:
   echo "Jumped here";
   ```

3. **Backward Jumps**
   ```php
   start:
   echo "Looping";
   if ($condition) {
       goto start;
   }
   ```

4. **Allowed Uses**
   - Jump to a label before the goto (forward jump)
   - Jump to a label after the goto (backward jump)
   - Jump within the same scope

5. **Prohibited Uses**
   - Jump into or out of a function
   - Jump into or out of a loop (for, while, foreach)
   - Jump into or out of a switch statement
   - Jump into or out of an if statement
   - Jump into a try block (can jump out)
   - Jump into a finally block
   - Jump across use statements or namespace declarations
   - Jump from one method to another

### Edge Cases

1. **Multiple Labels with Same Name**
   ```php
   label:
   // code
   goto label;
   label:  // Error: label already defined
   ```

2. **Undefined Label**
   ```php
   goto undefined;  // Error: label not found
   ```

3. **Jump Out of Loops**
   ```php
   for ($i = 0; $i < 10; $i++) {
       if ($i == 5) {
           goto outside;
       }
   }
   outside:
   echo "Exited loop";  // Valid
   ```

4. **Jump Into Loops (Invalid)**
   ```php
   goto inside;  // Error: cannot jump into loop
   for ($i = 0; $i < 10; $i++) {
       inside:
       echo "Inside loop";
   }
   ```

5. **Jump with finally Block**
   ```php
   try {
       goto after;
   } finally {
       echo "Finally always executes";
   }
   after:
   echo "After try/finally";  // Valid
   ```

### Compile-time Validation

PHP validates goto statements at compile time:
- Check that target label exists
- Check that jump is within same scope
- Check that jump doesn't violate control structure rules
- Report errors at compile time, not runtime

## Implementation Plan

### Phase 1: AST Extensions

**File:** `ast/stmt.rs` (existing)

```rust
pub enum Stmt {
    // ... existing
    Label {
        name: String,
    },
    Goto {
        label: String,
    },
}
```

**Tasks:**
- [ ] Add Label statement variant
- [ ] Add Goto statement variant
- [ ] Update AST visitor pattern if needed

### Phase 2: Lexer Updates

**File:** `lexer.rs` (existing)

**Tasks:**
- [ ] Add Label token recognition (identifier followed by `:`)
- [ ] Add Goto keyword token
- [ ] Update identifier/keyword handling

```rust
// In lexer.rs
TokenKind::Goto,
TokenKind::Label(String),  // For label names
```

### Phase 3: Parser Updates

**File:** `parser/stmt.rs` (existing)

```rust
fn parse_label(&mut self) -> Result<Stmt, String> {
    let name = self.expect_identifier()?;
    self.expect_colon()?;
    Ok(Stmt::Label { name })
}

fn parse_goto(&mut self) -> Result<Stmt, String> {
    self.expect_identifier()?; // consume 'goto'
    let label = self.expect_identifier()?;
    self.expect_semicolon()?;
    Ok(Stmt::Goto { label })
}
```

**Tasks:**
- [ ] Parse label statements (identifier followed by colon)
- [ ] Parse goto statements
- [ ] Add to statement parsing dispatch

### Phase 4: Label Tracking

**File:** `vm/compiler.rs` or new `vm/compiler/labels.rs`

**Tasks:**
- [ ] Create label tracking system
- [ ] Track label positions (bytecode offset)
- [ ] Validate label uniqueness
- [ ] Store label targets for goto compilation

```rust
pub struct LabelTracker {
    labels: HashMap<String, usize>,  // label name -> bytecode offset
    pending_gotos: Vec<(usize, String)>,  // offset -> target label name
}

impl LabelTracker {
    pub fn define_label(&mut self, name: String, offset: usize) -> Result<(), String> {
        if self.labels.contains_key(&name) {
            return Err(format!("Label '{}' already defined", name));
        }
        self.labels.insert(name, offset);
        Ok(())
    }

    pub fn add_goto(&mut self, offset: usize, label: String) {
        self.pending_gotos.push((offset, label));
    }

    pub fn resolve_gotos(&mut self) -> Result<Vec<(usize, usize)>, String> {
        // Convert pending gotos to (goto_offset, target_offset)
    }
}
```

### Phase 5: Scope Validation

**File:** `vm/compiler.rs` or `vm/compiler/labels.rs`

**Tasks:**
- [ ] Track current scope (function/method/top-level)
- [ ] Validate goto is within same scope as label
- [ ] Validate goto doesn't cross control structure boundaries
- [ ] Implement scope tracking for if/for/while/foreach/switch

```rust
pub enum Scope {
    Global,
    Function(String),
    Method(String, String),  // class_name, method_name
}

pub struct ScopeTracker {
    current_scope: Option<Scope>,
    control_stack: Vec<ControlContext>,  // Track nested control structures
}

pub enum ControlContext {
    For,
    While,
    Foreach,
    Switch,
    If,
    Try,
}
```

### Phase 6: Bytecode Compilation

**File:** `vm/opcode.rs` (existing)

```rust
// Add opcodes
GOTO,           // Unconditional jump to label
LABEL,          // Define a label position
```

**File:** `vm/compiler.rs` or `vm/compiler/stmt.rs`

```rust
fn compile_label(&mut self, name: String) -> Result<(), String> {
    // Record current bytecode position
    let offset = self.bytecode.len();
    self.label_tracker.define_label(name, offset)?;
    self.emit(Opcode::LABEL);
    self.emit_string(&name);
    Ok(())
}

fn compile_goto(&mut self, label: String) -> Result<(), String> {
    // Record goto position for later resolution
    let offset = self.bytecode.len();
    self.emit(Opcode::GOTO);
    // Emit placeholder offset, will be patched
    self.emit_u32(0);
    self.label_tracker.add_goto(offset, label);
    Ok(())
}
```

**Tasks:**
- [ ] Add GOTO opcode
- [ ] Add LABEL opcode
- [ ] Compile label statements
- [ ] Compile goto statements
- [ ] Implement goto resolution (patch jumps)

### Phase 7: VM Execution

**File:** `vm/mod.rs` (existing)

```rust
Opcode::LABEL => {
    let name = self.read_string();
    // Label definition - just skip (used for validation)
}

Opcode::GOTO => {
    let target_offset = self.read_u32() as usize;
    self.ip = target_offset;
}
```

**Tasks:**
- [ ] Implement GOTO execution
- [ ] Implement LABEL execution (no-op)
- [ ] Update instruction pointer
- [ ] Ensure jumps stay within bytecode bounds

### Phase 8: Compile-time Validation

**File:** `vm/compiler.rs` or `vm/compiler/labels.rs`

**Tasks:**
- [ ] Validate all gotos reference existing labels
- [ ] Validate gotos don't cross function boundaries
- [ ] Validate gotos don't cross control structure boundaries
- [ ] Provide clear error messages

```rust
fn validate_goto(&self, goto_offset: usize, label: &str) -> Result<(), String> {
    // Check label exists
    let Some(&target_offset) = self.label_tracker.labels.get(label) else {
        return Err(format!("Undefined label: {}", label));
    };

    // Check scope
    let goto_scope = self.get_scope_at(goto_offset)?;
    let label_scope = self.get_scope_at(target_offset)?;
    if goto_scope != label_scope {
        return Err("goto cannot jump across function/method boundaries".to_string());
    }

    // Check control structure crossing
    if self.crosses_control_structure(goto_offset, target_offset)? {
        return Err("goto cannot jump into/out of control structures".to_string());
    }

    Ok(())
}
```

### Phase 9: Error Messages

**Tasks:**
- [ ] Clear error for undefined label
- [ ] Clear error for duplicate label
- [ ] Clear error for jumping into control structure
- [ ] Clear error for jumping across function boundary
- [ ] Match PHP error message format

### Phase 10: Tests

**File:** `tests/control_flow/goto/` (new directory)

Test coverage:
- Basic forward jump
- Basic backward jump
- Jump out of loop (valid)
- Jump into loop (invalid)
- Jump out of if statement (invalid)
- Jump out of switch statement (invalid)
- Jump out of try-finally (valid, finally executes)
- Jump into try block (invalid)
- Undefined label error
- Duplicate label error
- Jump across function boundary (invalid)
- Multiple goto jumps
- Goto in nested scopes
- Edge cases

**Example tests:**

```
--TEST--
Basic goto forward jump
--FILE--
<?php
goto start;
echo "This won't print";
start:
echo "Hello from start";
--EXPECT--
Hello from start
```

```
--TEST--
Basic goto backward jump (loop)
--FILE--
<?php
$i = 0;
start:
echo $i . "\n";
$i++;
if ($i < 3) goto start;
--EXPECT--
0
1
2
```

```
--TEST--
Cannot jump into loop
--FILE--
<?php
goto inside;
for ($i = 0; $i < 10; $i++) {
    inside:
    echo "Inside";
}
--EXPECT_ERROR--
Cannot jump into loop
```

```
--TEST--
Undefined label error
--FILE--
<?php
goto undefined;
--EXPECT_ERROR--
Undefined label
```

## Implementation Details

### Control Structure Crossing Detection

```rust
fn crosses_control_structure(&self, from: usize, to: usize) -> Result<bool, String> {
    // Find all control structure boundaries between 'from' and 'to'
    // Check if boundary is crossed in invalid direction
    // For example: can't jump INTO a for loop

    let control_boundaries = self.get_control_boundaries_between(from, to)?;

    for boundary in control_boundaries {
        match boundary {
            ControlBoundary::Enter { kind, .. } => {
                if to > from && kind.is_loop_like() {
                    return Ok(true); // Jumping into a loop
                }
            },
            ControlBoundary::Exit { kind, .. } => {
                if to < from && kind.is_jump_out_target() {
                    return Ok(true); // Jumping out of restricted block
                }
            },
        }
    }

    Ok(false)
}
```

### Goto Resolution

```rust
fn resolve_gotos(&mut self) -> Result<(), String> {
    for (goto_offset, label_name) in &self.label_tracker.pending_gotos {
        let Some(&target_offset) = self.label_tracker.labels.get(label_name) else {
            return Err(format!("Undefined label: {}", label_name));
        };

        // Validate goto
        self.validate_goto(*goto_offset, label_name)?;

        // Patch the bytecode with actual target offset
        self.bytecode[*goto_offset + 1] = (target_offset & 0xFF) as u8;
        self.bytecode[*goto_offset + 2] = ((target_offset >> 8) & 0xFF) as u8;
        self.bytecode[*goto_offset + 3] = ((target_offset >> 16) & 0xFF) as u8;
        self.bytecode[*goto_offset + 4] = ((target_offset >> 24) & 0xFF) as u8;
    }

    Ok(())
}
```

## Dependencies

- Existing AST
- Existing parser
- Existing compiler
- Existing VM

## Testing Strategy

1. **Unit Tests**: Label parsing, goto parsing
2. **Compilation Tests**: Label tracking, goto validation
3. **Execution Tests**: Goto jumps at runtime
4. **Error Tests**: Invalid goto usage
5. **Compatibility Tests**: Match PHP 8.x behavior

## Success Criteria

- goto statements parse correctly
- goto statements compile correctly
- goto statements execute correctly
- All validation rules are enforced
- Clear error messages for invalid usage
- All tests pass

## Performance Considerations

- Compile-time validation (no runtime cost)
- Direct jumps (no loop overhead)
- Efficient label lookup
- Minimal bytecode overhead

## Security Considerations

- Prevent infinite loops with goto
- Validate jumps don't exceed bytecode bounds
- Ensure goto can't bypass security checks

## Open Questions

- Should we allow goto in production code (warning)?
- How to handle goto in eval()?

## References

- PHP goto documentation: https://www.php.net/manual/en/control-structures.goto.php
- PHP 5.3 goto RFC: https://wiki.php.net/rfc/goto

## Related Plans

- Exception handling (already implemented)
- Control flow (already implemented)
