# VHP Bytecode VM Full Integration Plan

**Status:** In Progress - VM is now the default (no fallback)
**Priority:** High
**Prerequisite:** bytecode-vm.md (implemented)

---

## Progress Summary

### Completed
- ✅ **Phase 1: Built-in Function Bridge** - All 73 built-in functions accessible from VM
- ✅ **Phase 2: Foreach & Array Operations** - Foreach loops with key/value, array assignment
- ✅ **Phase 6: Integration & Default Mode** - VM is now the only execution mode
- ✅ **Increment/Decrement** - ++$x, $x++, --$x, $x-- implemented
- ✅ **Array Assignment** - $arr[$key] = $value implemented
- ✅ **Switch statements** - Including break support
- ✅ **Match expressions** - Full compilation support
- ✅ **Class compilation** - Classes compile to CompiledClass structures
- ✅ **Interface/Trait/Enum compilation** - All OOP constructs compile
- ✅ **Removed interpreter fallback** - main.rs now uses VM only

### Partial Implementation
- 🔄 **Phase 3: OOP Support** - Classes compile but property initialization needs work
- 🔄 **Phase 4: Exceptions** - try/catch compiles but Exception class needs VM support
- 🔄 **Phase 5: Closures** - Arrow functions compile, CallableCall execution incomplete

### Current State
- **272/490 tests pass** (56% pass rate)
- VM is the sole execution mode (no fallback)
- 7.3x speedup maintained on Fibonacci benchmark
- `--legacy` and `--vm` flags removed - VM is the only option

---

## Current Test Results

| Category | Status | Notes |
|----------|--------|-------|
| Control flow | ✅ Mostly passing | All loop types work |
| Functions | 🔄 Partial | Basic functions work, closures incomplete |
| Arrays | ✅ Mostly passing | Core operations work |
| Strings | ✅ Passing | All string operations work |
| Built-ins | ✅ Passing | 73 functions via bridge |
| Classes | 🔄 Partial | Compilation works, property access needs fixes |
| Interfaces | 🔄 Partial | Basic support works |
| Exceptions | ❌ Failing | Exception class not in VM |
| Closures | ❌ Failing | CallableCall not fully implemented |
| Pipe operator | ❌ Not implemented | VHP-specific feature |

---

## Key Issues to Address

### 1. OOP Property Access
Properties are being compiled but not initialized correctly in constructors.
Constructor property promotion needs implementation.

### 2. Method Inheritance
Methods from parent classes not found through inheritance chain.
Need to traverse parent class chain in VM method lookup.

### 3. Static Methods
`self::`, `static::`, `parent::` keywords not resolved at runtime.
Need special handling in VM for these class references.

### 4. Exception Class
The built-in Exception class exists in interpreter but not VM.
Need to register Exception class in VM initialization.

### 5. Callable Calls
Arrow functions and closures compile but can't be invoked.
Need to implement closure/callable invocation in VM.

---

## Architecture

The VM now consists of:

```
src/vm/
├── mod.rs          - Main VM execution loop
├── compiler.rs     - AST to bytecode compilation
├── opcode.rs       - Opcode definitions
├── frame.rs        - Call frames and loop contexts
├── class.rs        - CompiledClass, CompiledInterface, etc.
└── builtins.rs     - Bridge to interpreter builtins
```

The interpreter module is kept for:
- Value type definitions
- ObjectInstance type
- Built-in function implementations
- (Tree-walk execution is dead code, can be removed later)

---

## Next Steps

1. Fix OOP property initialization in constructors
2. Implement method inheritance chain lookup
3. Add self::/static::/parent:: resolution
4. Register Exception class in VM
5. Implement CallableCall for closures
6. Remove dead interpreter tree-walk code
7. Achieve 100% test pass rate

---

## Success Metrics

- [ ] All 490 tests pass with VM-only mode
- [x] VM is default execution mode
- [x] No interpreter fallback
- [x] 7x+ speedup maintained
- [ ] Clean removal of tree-walk interpreter code
