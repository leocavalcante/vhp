# Moving VHP from Tree-Walking Interpreter to JIT Compilation

## Current Architecture Analysis

**VHP Today (v0.1.0):**
- **Implementation**: Tree-walking interpreter (~15,000 lines of Rust)
- **Execution model**: Direct AST traversal and evaluation
- **Memory model**: Rust ownership with runtime value types
- **Performance**: 5-8x faster than PHP on simple operations, 45x slower on recursive calls

## Complexity Assessment: Tree-Walking → JIT

### Difficulty Rating: **8/10** (Very Complex)

Moving from a tree-walking interpreter to JIT compilation is one of the most challenging transformations in language implementation. Here's why:

---

## Phase 1: Bytecode VM (Medium-High Complexity)
**Estimated Effort**: 3-6 months • **Difficulty**: 6/10

Before JIT, you need an intermediate bytecode representation:

### What Changes:
1. **New Bytecode Compiler** (NEW: ~5,000 lines)
   - Convert AST → bytecode instructions
   - Instruction set design (OpCodes)
   - Constant pool management
   - Jump/branch resolution

2. **Stack-Based VM** (NEW: ~3,000 lines)
   - Operand stack implementation
   - Call frame management
   - Exception unwinding
   - Bytecode dispatch loop

3. **Existing Code Impact**:
   - Parser/AST: **No changes** ✓
   - Interpreter: **Completely replaced** (15,000 lines → bytecode compiler)
   - Value system: **Moderate changes** (stack-oriented representation)

### Example: Before vs After

**Before (Tree-walking):**
```rust
// Direct AST execution
match expr {
    Expr::BinaryOp { left, op, right } => {
        let l = self.eval_expr(left)?;  // Recursive
        let r = self.eval_expr(right)?; // Recursive
        apply_op(l, op, r)
    }
}
```

**After (Bytecode VM):**
```rust
// Bytecode generation
fn compile_expr(&mut self, expr: &Expr) {
    match expr {
        Expr::BinaryOp { left, op, right } => {
            self.compile_expr(left);   // Emit bytecode for left
            self.compile_expr(right);  // Emit bytecode for right
            self.emit(OpCode::Add);    // Emit operation
        }
    }
}

// VM execution
fn execute(&mut self) {
    loop {
        match self.fetch_opcode() {
            OpCode::Add => {
                let r = self.stack.pop();
                let l = self.stack.pop();
                self.stack.push(l + r);
            }
            // ...
        }
    }
}
```

**Benefits of Bytecode VM:**
- ✅ **2-5x performance improvement** over tree-walking
- ✅ **Foundation for JIT** - can't skip this step
- ✅ **Better instruction caching**
- ✅ **Smaller memory footprint** (bytecode vs AST)

---

## Phase 2: JIT Compilation (Very High Complexity)
**Estimated Effort**: 6-12 months • **Difficulty**: 9/10

### What Changes:
1. **JIT Infrastructure** (NEW: ~8,000-15,000 lines)
   - Hot path detection (profiling)
   - Bytecode → native code translation
   - Register allocation
   - CPU-specific code generation
   - Code cache management
   - Deoptimization (fallback to interpreter)

2. **Assembly/LLVM Integration**:
   - **Option A**: Hand-written assembler (~10,000 lines)
     - x86-64, ARM64 code generation
     - Calling conventions
     - Memory addressing modes

   - **Option B**: LLVM backend (~5,000 lines + LLVM dependency)
     - LLVM IR generation
     - Optimization passes
     - Target-independent (but huge dependency)

3. **Advanced Features**:
   - **Type speculation** - Assume int, fallback if wrong
   - **Inline caching** - Cache method lookups
   - **Escape analysis** - Stack-allocate objects when possible
   - **Guard insertion** - Runtime type checks

### PHP's Approach (for reference)

PHP uses **OPcache JIT** (added in PHP 8.0):
- **Bytecode**: Zend VM opcodes (200+ instructions)
- **JIT**: DynASM (lightweight assembly) or LLVM
- **Tiers**: Interpretation → Function JIT → Tracing JIT
- **Codebase**: ~50,000+ lines for Zend Engine + JIT

### Example: JIT Code Generation

**Bytecode:**
```
OpCode::LoadConst(5)  // Push 5
OpCode::LoadConst(3)  // Push 3
OpCode::Add           // Pop 2, push sum
OpCode::Return        // Return TOS
```

**Generated x86-64 Assembly:**
```asm
mov rax, 5        ; Load constant 5 into register
mov rbx, 3        ; Load constant 3 into register
add rax, rbx      ; Add registers
ret               ; Return (result in rax)
```

**Complexity Factors:**
- **Type guards**: What if they're not ints? Insert runtime checks
- **Memory allocation**: What if result overflows? Box it
- **Deoptimization**: Fall back to interpreter when assumptions fail

---

## Complexity Breakdown

| Component | Lines of Code | Difficulty | Skills Required |
|-----------|---------------|------------|-----------------|
| **Current Interpreter** | 15,000 | - | Rust, Language Design |
| **Bytecode Compiler** | 5,000 | 6/10 | Compilers, IR Design |
| **Stack VM** | 3,000 | 5/10 | VM Design, Performance |
| **JIT Infrastructure** | 8,000 | 9/10 | Compilers, Low-level |
| **Assembly Codegen** | 10,000 | 10/10 | x86/ARM, Assembly |
| **LLVM Integration** | 5,000 | 7/10 | LLVM IR, Optimization |
| **Type Speculation** | 3,000 | 8/10 | Type Systems, Profiling |
| **Deoptimization** | 2,000 | 8/10 | VM Design, Control Flow |
| **Testing/Debugging** | - | 9/10 | All of the above |

**Total New Code**: 30,000-50,000 lines (2-3x current codebase)

---

## Major Challenges

### 1. **Type Speculation is Hard** (Difficulty: 9/10)
PHP is dynamically typed. You need to:
- Profile hot paths to guess types
- Insert guards: "is this still an int?"
- Deoptimize if guards fail
- Handle 100+ type coercion rules

**Example:**
```php
function add($a, $b) {
    return $a + $b;  // int? float? string? array?
}
```

JIT must handle:
- `add(1, 2)` → int addition (fast path)
- `add("1", 2)` → coercion + addition (slow path)
- `add([1], 2)` → TypeError (exception path)

### 2. **PHP Semantics are Complex** (Difficulty: 8/10)
- **Dynamic everything**: Types, properties, methods
- **Reference semantics**: `&$var` changes ownership rules
- **Error handling**: Warnings vs Exceptions vs Fatal errors
- **Coercion rules**: 100+ edge cases

### 3. **Multi-Architecture Support** (Difficulty: 10/10)
If hand-writing assembly:
- x86-64: 1,000+ instructions, complex addressing modes
- ARM64: Different ISA, different calling conventions
- Windows vs Linux vs macOS: Different ABIs

### 4. **Debugging JIT is Nightmare** (Difficulty: 10/10)
- Bugs in generated code cause segfaults
- No stack traces (you're in raw assembly)
- Race conditions in code cache
- Deoptimization bugs are subtle

---

## Alternative Approaches

### Option 1: **Bytecode VM Only** (Recommended)
**Effort**: 3-6 months • **Benefit**: 2-5x speedup

Skip JIT entirely, just implement a bytecode VM:
- ✅ **Much simpler** (6/10 difficulty)
- ✅ **Still faster** than tree-walking
- ✅ **Easier to maintain**
- ✅ **No assembly required**
- ❌ Won't match PHP JIT performance
- ❌ Recursive calls still slow

**This gets you 80% of the benefit for 20% of the effort.**

### Option 2: **LLVM Backend** (Moderate Complexity)
**Effort**: 6-9 months • **Benefit**: 5-10x speedup

Use LLVM instead of hand-written assembly:
- ✅ **Target-independent** (supports all CPUs)
- ✅ **World-class optimizations** (LLVM is mature)
- ✅ **No manual assembly**
- ❌ **Huge dependency** (LLVM is 200+ MB)
- ❌ **Compilation slower** (LLVM is heavyweight)
- ❌ Still need type speculation/guards

### Option 3: **Tiered Compilation** (PHP's Approach)
**Effort**: 12-18 months • **Benefit**: 10-50x speedup

Three-tier execution:
1. **Interpreter** (fast startup)
2. **Function JIT** (compile hot functions)
3. **Tracing JIT** (compile hot loops)

- ✅ **Best performance** (matches PHP JIT)
- ✅ **Balances startup and runtime**
- ❌ **Most complex** (9/10 difficulty)
- ❌ **Hardest to maintain**

---

## Recommendation

For VHP, I'd suggest this roadmap:

### Phase 1: **Bytecode VM** (3-6 months)
Focus on a simple, fast bytecode interpreter:
- Stack-based VM with 50-100 opcodes
- Minimal optimizations (constant folding, dead code elimination)
- Keep tree-walking as fallback during development

**Expected Results:**
- Fibonacci: 45x slower → 10-15x slower 🎯
- Arrays/strings: Stay fast (already optimized) ✓
- Codebase: +8,000 lines (manageable) ✓

### Phase 2: **Profile-Guided Optimization** (1-2 months)
Before adding JIT, optimize the VM:
- Function inlining for built-ins
- Specialized opcodes (AddInt, AddFloat)
- Better call convention (reduce stack copying)

**Expected Results:**
- Fibonacci: 10-15x slower → 5-8x slower 🎯
- Arrays/strings: 10-15% faster ✓

### Phase 3: **Consider JIT** (Optional, 6-12 months)
Only if bytecode VM isn't fast enough:
- Start with LLVM (easier than hand-written assembly)
- JIT only the hottest 1% of code
- Keep most execution in bytecode

**Expected Results:**
- Fibonacci: 5-8x slower → on par with PHP 🎯
- Arrays/strings: 20-30% faster ✓

---

## Comparison: VHP vs PHP Development

| Metric | VHP (Current) | PHP Zend Engine |
|--------|---------------|-----------------|
| **Codebase Size** | 15,000 lines | 500,000+ lines |
| **Execution Model** | Tree-walking | Bytecode VM + JIT |
| **Development Time** | 3 months (AI-assisted) | 25+ years |
| **Performance** | 5-8x faster (simple ops) | Baseline |
| **JIT Complexity** | Not implemented | ~50,000 lines |
| **Maintenance** | Low (simple design) | High (complex VM) |

---

## Conclusion

**Moving to JIT: 8/10 Complexity**

**Why it's hard:**
1. **Massive codebase change** (30,000-50,000 new lines)
2. **Requires low-level expertise** (assembly, CPU architecture)
3. **Type speculation is complex** (PHP's dynamic nature)
4. **Debugging is extremely difficult**
5. **Maintenance burden** (ongoing CPU architecture support)

**Recommended approach:**
1. ✅ **Ship bytecode VM** (simpler, faster, maintainable)
2. ✅ **Optimize hot paths** (profile-guided optimization)
3. ⚠️ **Consider JIT later** (only if needed for benchmarks)

**Reality check:**
PHP's JIT took 2+ years to develop by a team of experienced VM engineers. For a solo/small team project, a bytecode VM gives you the best ROI.

**The good news:**
VHP is already faster than PHP on common operations! The tree-walking interpreter with Rust's zero-cost abstractions is competitive. A bytecode VM would cement that lead without the complexity of JIT.

---

## Resources if You Decide to Build JIT

- **Books**:
  - "Crafting Interpreters" by Robert Nystrom (bytecode VM)
  - "Engineering a Compiler" by Cooper & Torczon (JIT theory)

- **Existing Implementations**:
  - LuaJIT (best-in-class tracing JIT, ~50,000 lines C)
  - V8 (JavaScript, very complex, 1M+ lines C++)
  - PyPy (Python, RPython framework, ~250,000 lines)

- **Rust JIT Libraries**:
  - `cranelift` (Rust-native code generator, used by Wasmtime)
  - `inkwell` (Rust LLVM bindings)
  - `dynasm-rs` (Rust port of LuaJIT's DynASM)

---

**Bottom Line**: A bytecode VM is the sweet spot for VHP. It's achievable, gives you 2-5x speedup, and doesn't require assembly expertise. JIT is a "nice to have" for the future, not a "must have" today.
