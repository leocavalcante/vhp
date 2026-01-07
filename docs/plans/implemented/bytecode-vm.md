# VHP Bytecode VM Implementation Plan

**Status:** Planned
**Priority:** High
**Estimated Effort:** 14 weeks
**Performance Target:** 2-5x speedup overall, 10x on recursive functions

---

## Executive Summary

Implement a bytecode VM to replace the current tree-walking interpreter, targeting significant performance improvements while maintaining 100% compatibility with all 490 existing tests.

**Current State:**
- Tree-walking interpreter: ~15,125 lines of Rust code
- Performance: 5-8x faster than PHP on simple operations, 45x slower on recursion
- Strong test coverage: 490 comprehensive tests
- Well-structured AST with clear separation of concerns

**Target State:**
- Hybrid interpreter with bytecode VM for hot paths
- Performance: 2-5x overall improvement (especially recursive functions)
- Zero test breakage during incremental rollout
- Maintainable, well-documented bytecode instruction set

---

## 1. Bytecode Instruction Set Design

**Stack-based VM with ~70 core opcodes**

### 1.1 Core Instruction Categories

#### Literals & Constants (8 opcodes)
```rust
pub enum Opcode {
    PushNull,           // Push Value::Null
    PushTrue,           // Push Value::Bool(true)
    PushFalse,          // Push Value::Bool(false)
    PushInt(i64),       // Push integer literal
    PushFloat(f64),     // Push float literal
    PushString(u32),    // Push string from constant pool (index)
    LoadConst(u32),     // Load any constant from pool
    LoadGlobal(u32),    // Load global constant by name index
}
```

#### Variables (6 opcodes)
```rust
    LoadVar(u32),        // Load variable by name index
    StoreVar(u32),       // Store top of stack to variable
    LoadFast(u16),       // Load local variable by slot (fast path)
    StoreFast(u16),      // Store to local variable slot
    LoadGlobalVar(u32),  // Load from global scope
    StoreGlobalVar(u32), // Store to global scope
```

#### Arithmetic & Logic (12 opcodes)
```rust
    Add, Sub, Mul, Div, Mod, Pow,  // Arithmetic
    Concat,                         // String concatenation
    Eq, Identical, Lt, Gt,          // Comparison
    Spaceship,                      // <=>
```

#### Control Flow (10 opcodes)
```rust
    Jump(u32),           // Unconditional jump
    JumpIfFalse(u32),    // Conditional jump
    JumpIfTrue(u32),
    JumpIfNull(u32),     // For ??=
    Call(u16),           // Function call
    CallBuiltin(u32, u8), // Built-in function
    Return,
    ForLoop(u32),        // Optimized for loop
    BreakLoop,
    ContinueLoop,
```

#### Arrays (8 opcodes)
```rust
    NewArray(u16),       // Create array
    ArrayPush,           // Push to array
    ArraySet,            // $arr[$key] = $val
    ArrayGet,            // $arr[$key]
    ArrayAppend,         // $arr[] = $val
    ArrayUnpack,         // Spread: ...$arr
    ArrayCount,          // Optimized count()
    ArrayMerge,          // Optimized array_merge()
```

#### Objects & OOP (12 opcodes)
```rust
    NewObject(u32),          // new ClassName
    LoadProperty(u32),       // $obj->prop
    StoreProperty(u32),
    LoadStaticProp(u32, u32),
    StoreStaticProp(u32, u32),
    CallMethod(u32, u8),     // $obj->method()
    CallStaticMethod(u32, u32, u8),
    LoadThis,
    InstanceOf(u32),
    Clone,
    CloneWith(u16),
    Throw,
```

#### Stack & Utility (8 opcodes)
```rust
    Pop, Dup, Swap,          // Stack manipulation
    Cast(TypeHint),          // Type casting
    TypeCheck(TypeHint),     // Type validation
    Match(u32),              // Match expression
    Yield,                   // Generator (future)
    Nop,
```

**Total: ~70 opcodes** covering 95% of common PHP operations.

---

## 2. VM Architecture

### 2.1 Core VM Structure

```rust
pub struct VM {
    // Execution state
    stack: Vec<Value>,              // Operand stack
    frames: Vec<CallFrame>,         // Call frame stack
    ip: usize,                      // Instruction pointer

    // Runtime data
    constants: Vec<Value>,          // Constant pool
    strings: Vec<String>,           // String pool
    globals: HashMap<String, Value>,

    // Limits
    max_stack_size: usize,          // Stack overflow protection
    max_call_depth: usize,          // Recursion limit

    // Shared state
    classes: HashMap<String, ClassDefinition>,
    functions: HashMap<String, CompiledFunction>,

    // Output
    output: Box<dyn Write>,
}
```

### 2.2 Call Frame Structure

```rust
pub struct CallFrame {
    function: FunctionRef,
    bytecode: Arc<[u8]>,
    ip: usize,
    stack_base: usize,
    locals: Vec<Value>,              // Fixed slots
    local_names: Arc<Vec<String>>,
    saved_globals: Option<HashMap<String, Value>>,
    this: Option<ObjectInstance>,
    called_class: Option<String>,
}
```

### 2.3 Execution Loop

```rust
impl VM {
    pub fn execute(&mut self) -> Result<Value, VMError> {
        loop {
            let opcode = self.fetch_opcode()?;
            match opcode {
                Opcode::PushInt(n) => self.stack.push(Value::Integer(n)),
                Opcode::Add => {
                    let right = self.stack.pop().ok_or(StackUnderflow)?;
                    let left = self.stack.pop().ok_or(StackUnderflow)?;
                    self.stack.push(self.add_values(left, right)?);
                }
                Opcode::Call(arg_count) => {
                    let func = self.stack.pop().ok_or(StackUnderflow)?;
                    let args = self.pop_n(arg_count)?;
                    self.call_function(func, args)?;
                }
                Opcode::Return => {
                    let val = self.stack.pop().unwrap_or(Value::Null);
                    self.pop_frame()?;
                    if self.frames.is_empty() { return Ok(val); }
                    self.stack.push(val);
                }
                // ... 65 more opcodes
            }
        }
    }
}
```

---

## 3. Compiler Design

### 3.1 Compiler Structure

```rust
pub struct Compiler {
    bytecode: Vec<u8>,
    constants: Vec<Value>,
    strings: Vec<String>,
    locals: Vec<LocalVar>,
    scopes: Vec<Scope>,
    loop_stack: Vec<LoopContext>,
    label_counter: usize,
    patches: Vec<JumpPatch>,
}
```

### 3.2 Compilation Pipeline

```rust
impl Compiler {
    pub fn compile_function(&mut self, func: &UserFunction)
        -> Result<CompiledFunction, CompileError> {
        // 1. Analyze locals
        self.analyze_locals(&func.params, &func.body)?;

        // 2. Emit prologue
        self.emit_prologue(func.params.len())?;

        // 3. Compile statements
        for stmt in &func.body {
            self.compile_stmt(stmt)?;
        }

        // 4. Implicit return
        self.emit(Opcode::PushNull);
        self.emit(Opcode::Return);

        Ok(CompiledFunction { /* ... */ })
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<(), CompileError> {
        match expr {
            Expr::Integer(n) => self.emit(Opcode::PushInt(*n)),
            Expr::Variable(name) => {
                if let Some(slot) = self.find_local(name) {
                    self.emit(Opcode::LoadFast(slot));
                } else {
                    let idx = self.add_string(name);
                    self.emit(Opcode::LoadVar(idx));
                }
            }
            Expr::Binary { left, op, right } => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;
                self.emit(self.binary_op_to_opcode(op));
            }
            // ... all expression types
        }
        Ok(())
    }
}
```

---

## 4. Integration Strategy

### Phase 1: Hybrid Interpreter (Weeks 1-3)
- Create `src/vm/` module
- Implement bytecode types and VM execution loop
- Build minimal compiler (10 opcodes)
- Keep tree-walker as default
- **Tests:** All 490 pass (VM not yet used)

### Phase 2: Function-Level VM (Weeks 4-6)
- Compile simple user functions
- Add control flow opcodes
- Enable bytecode selectively
- **Tests:** 490 pass (VM handles 30-40% of calls)

### Phase 3: Performance Critical (Weeks 7-9)
- Add array opcodes
- Optimize loops
- Target recursive functions
- **Benchmark:** 2-3x faster on recursion

### Phase 4: OOP & Advanced (Weeks 10-12)
- Implement object opcodes
- Add exception handling
- Support closures
- **Tests:** All 490 pass with VM by default

### Phase 5: Optimization (Weeks 13-14)
- Profile and optimize
- Add specialized opcodes
- Final benchmarking
- **Target:** 2-5x overall speedup

---

## 5. Testing Approach

### 5.1 Unit Tests (Per-Opcode)
```rust
#[test]
fn test_add_integers() {
    let mut vm = VM::new();
    vm.load_bytecode(&[
        Opcode::PushInt(10),
        Opcode::PushInt(32),
        Opcode::Add,
        Opcode::Return,
    ]);
    assert_eq!(vm.execute().unwrap(), Value::Integer(42));
}
```

### 5.2 Integration Tests
- Run all 490 .vhpt tests with VM enabled
- Compare VM output vs tree-walker output
- Verify identical behavior

### 5.3 Regression Detection
```rust
#[test]
fn test_vm_vs_tree_walker() {
    let code = "<?php function factorial($n) { /* ... */ }";
    assert_eq!(run_with_tree_walker(code), run_with_vm(code));
}
```

---

## 6. Implementation Phases (14-Week Roadmap)

### **Week 1-3: Foundation**
- [x] Design instruction set
- [ ] Create `src/vm/` module
- [ ] Implement `Opcode` enum
- [ ] Build VM struct with stack
- [ ] Write 20 unit tests
- **Deliverable:** VM executes `2 + 2`

### **Week 4-6: Compiler Basics**
- [ ] Implement Compiler struct
- [ ] Compile expressions
- [ ] Compile control flow
- [ ] Test with 50 simple tests
- **Deliverable:** Non-recursive functions work

### **Week 7-9: Integration**
- [ ] Integrate into Interpreter
- [ ] Add hybrid mode
- [ ] Implement hotspot detection
- [ ] Verify all 490 tests pass
- **Deliverable:** 2x speedup on recursion

### **Week 10-11: Arrays & Loops**
- [ ] Implement array opcodes
- [ ] Optimize for/foreach
- [ ] Test array operations
- **Deliverable:** 1.5x speedup on arrays

### **Week 12-13: OOP**
- [ ] Implement object opcodes
- [ ] Compile classes
- [ ] Support method calls
- **Deliverable:** VM handles OOP

### **Week 14: Finalization**
- [ ] Exception handling
- [ ] Closures (if time permits)
- [ ] Profile and optimize
- [ ] Enable VM by default
- **Deliverable:** 2-5x overall speedup

---

## 7. File Structure

```
src/vm/                          # New VM module
├── mod.rs                       # VM execution loop (500 lines)
├── opcode.rs                    # Opcode definitions (400 lines)
├── frame.rs                     # Call frames (300 lines)
├── compiler.rs                  # AST → Bytecode (800 lines)
├── constant_pool.rs             # Constants (200 lines)
├── opcodes/
│   ├── arithmetic.rs            # Math ops (150 lines)
│   ├── control_flow.rs          # Jumps, calls (200 lines)
│   ├── arrays.rs                # Array ops (250 lines)
│   ├── objects.rs               # OOP ops (300 lines)
│   └── stack.rs                 # Stack ops (100 lines)
└── error.rs                     # Error types (100 lines)

tests/vm/
├── opcodes.rs                   # Per-opcode tests (1000 lines)
├── compiler.rs                  # Compiler tests (500 lines)
└── integration.rs               # Integration tests (300 lines)
```

**Estimated New Code:** ~4,500 lines
**Modified Code:** ~50 lines in existing interpreter

---

## 8. Performance Expectations

| Operation | Current | VM Target | Speedup |
|-----------|---------|-----------|---------|
| Recursive functions | 51.8ms | 10-15ms | **3-5x** ⚡ |
| Loops (for 1M) | 9.3ms | 5-6ms | **1.5-2x** |
| Function calls | 51.8ms | 30-35ms | **1.5x** |
| Array operations | 11.8ms | 8-10ms | **1.2-1.5x** |
| Object creation | 8.3ms | 6-7ms | **1.2x** |
| **Overall** | Baseline | - | **2-5x** ⚡ |

### Why This Works
1. Eliminate AST traversal overhead (15-20% savings)
2. Fast local variables via fixed slots
3. Tight execution loop with inline dispatch
4. Reduced allocations (stack vs recursive frames)
5. Specialized opcodes for common patterns

---

## Critical Files

1. **`src/vm/mod.rs`** (NEW) - Core VM execution
2. **`src/vm/compiler.rs`** (NEW) - AST → Bytecode
3. **`src/vm/opcode.rs`** (NEW) - Instruction set
4. **`src/interpreter/mod.rs`** (MODIFY) - VM integration
5. **`src/interpreter/functions/mod.rs`** (MODIFY) - Call routing

---

## Success Criteria

✅ All 490 tests pass with VM enabled
✅ 2-5x overall performance improvement
✅ 10x faster on recursive functions
✅ Maintainable codebase (<5,000 new lines)
✅ Hybrid mode allows tree-walker fallback

---

## Next Steps

1. Review and approve this plan
2. Begin Phase 1: Create `src/vm/mod.rs` and `opcode.rs`
3. Implement first 10 opcodes with unit tests
4. Integrate with existing interpreter in hybrid mode
