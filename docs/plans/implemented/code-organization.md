# Code Organization Plan: Keep Files Under 300-500 Lines

## Status: Planned

## Overview

This plan establishes guidelines and processes for maintaining manageable file sizes in the VHP codebase. The goal is to keep all Rust source files under 300-500 lines to improve maintainability, readability, and navigation.

## Background

Large files are difficult to navigate, understand, and modify. Current codebase has several files exceeding the target:

**Files Requiring Immediate Attention:**
- `src/vm/mod.rs` - 4,295 lines (8.6x target)
- `src/parser/stmt/mod.rs` - 1,504 lines (3x target)
- `src/parser/expr/primary.rs` - 937 lines (1.9x target)
- `src/vm/compiler/definitions.rs` - 779 lines (1.6x target)
- `src/vm/compiler/expr.rs` - 682 lines (1.4x target)

**Files Approaching Limit:**
- `src/runtime/value.rs` - 665 lines
- `src/vm/opcode.rs` - 552 lines
- `src/vm/ops/object_ops.rs` - 490 lines
- `src/vm/values.rs` - 468 lines

## Requirements

### Target Guidelines

- **Optimal file size**: 200-400 lines
- **Maximum file size**: 500 lines (hard limit)
- **Target maximum**: 300 lines (soft limit)

### File Organization Principles

1. **Single Responsibility**: Each file/module should have one clear purpose
2. **Logical Grouping**: Group related functions/types together
3. **Clear Boundaries**: Files should have clear interfaces between them
4. **Minimal Dependencies**: Minimize circular dependencies
5. **Testable**: Each module should be independently testable

## Implementation Plan

### Phase 1: Immediate Large File Refactoring (Priority: HIGH)

#### 1.1 Refactor `src/vm/mod.rs` (4,295 lines)

**Current Structure Issues:**
- VM execution loop mixed with helper functions
- Opcode handlers and value operations interleaved
- No clear separation between concerns

**Proposed Structure:**
```
src/vm/
├── mod.rs                    # VM struct, main execution loop (200 lines)
├── frame.rs                  # Already exists (165 lines) ✓
├── class.rs                  # Already exists (193 lines) ✓
├── objects.rs                # Already exists (121 lines) ✓
├── values.rs                 # Already exists (468 lines) - keep
├── ops/                      # Already exists, check sizes
│   ├── mod.rs                # (28 lines) ✓
│   ├── arithmetic.rs         # Already exists (128 lines) ✓
│   ├── arrays.rs             # Already exists (148 lines) ✓
│   ├── closures.rs           # Already exists (49 lines) ✓
│   ├── comparison.rs         # Already exists (65 lines) ✓
│   ├── control_flow.rs       # Already exists (124 lines) ✓
│   ├── exceptions.rs         # Already exists (102 lines) ✓
│   ├── logical_bitwise.rs    # Already exists (24 lines) ✓
│   ├── misc.rs               # Already exists (170 lines) ✓
│   ├── object_ops.rs         # (490 lines) - needs splitting
│   ├── strings.rs            # Already exists (13 lines) ✓
│   └── functions.rs          # Already exists (265 lines) - needs review
├── methods.rs                 # Already exists (208 lines) ✓
├── builtins.rs                # Already exists (238 lines) ✓
├── reflection.rs              # Already exists (279 lines) ✓
├── compiler.rs                # Already exists (355 lines) - needs review
└── compiler/                  # Already exists, check sizes
    ├── compiler_types.rs      # Already exists (47 lines) ✓
    ├── definitions.rs         # Already exists (779 lines) - needs splitting
    ├── expr.rs                # Already exists (682 lines) - needs splitting
    ├── expr_helpers.rs       # Already exists (253 lines) - needs review
    ├── functions.rs           # Already exists (299 lines) - needs review
    ├── if_match.rs            # Already exists (175 lines) ✓
    ├── loops.rs               # Already exists (188 lines) ✓
    ├── stmt.rs                # Already exists (206 lines) ✓
    └── try_catch.rs           # Already exists (64 lines) ✓
```

**Refactoring Steps:**

1. **Extract opcode handlers** to new modules:
   - `vm/opcode_dispatch.rs` - Opcode execution switch (150-200 lines)
   - `vm/bytecode_interpreter.rs` - Bytecode interpretation logic (200-250 lines)

2. **Extract value operations**:
   - Move value coercion/conversion to `runtime/value_conversion.rs`
   - Move value comparison to `runtime/value_comparison.rs`
   - Keep Value enum in `runtime/value.rs`

3. **Extract VM state management**:
   - `vm/state.rs` - VM state, stack, frame management (200-250 lines)
   - `vm/execution.rs` - Main execution loop (150-200 lines)

4. **Update imports** in all affected files

**Success Criteria:**
- `vm/mod.rs` reduced to 200-300 lines
- All new modules under 300 lines
- All tests pass
- No change in functionality

#### 1.2 Refactor `src/parser/stmt/mod.rs` (1,504 lines)

**Current Structure Issues:**
- Mix of different statement types
- No clear separation between parsing logic

**Proposed Structure:**
```
src/parser/stmt/
├── mod.rs                    # Statement dispatcher, exports (150 lines)
├── expressions.rs            # Expression statements (150 lines)
├── control_flow.rs           # Already exists (322 lines) - needs splitting
│   ├── if.rs                # if/elseif/else, match (200 lines)
│   ├── loops.rs             # while, do-while, for, foreach (200 lines)
│   └── switch.rs            # switch/case (100 lines)
├── functions.rs             # Function declarations, arrow functions (200 lines)
├── classes.rs               # Already exists (340 lines) - needs review
├── interfaces.rs            # Already exists (235 lines) - needs review
├── traits.rs                # Already exists (233 lines) - needs review
├── enums.rs                 # Already exists (226 lines) - needs review
├── declarations.rs          # Variable, constant declarations (200 lines)
└── try_catch.rs             # try/catch/finally (150 lines)
```

**Refactoring Steps:**

1. **Split `control_flow.rs`** into separate modules
2. **Extract parsing helpers** to `parser/helpers.rs`
3. **Create statement factories** for complex statements
4. **Update imports** in `parser/mod.rs`

**Success Criteria:**
- `parser/stmt/mod.rs` reduced to 150-200 lines
- All new modules under 300 lines
- All tests pass

#### 1.3 Refactor `src/parser/expr/primary.rs` (937 lines)

**Current Structure Issues:**
- All primary expressions in one file
- Complex nested parsing logic

**Proposed Structure:**
```
src/parser/expr/
├── mod.rs                    # Expression dispatcher, exports (200 lines)
├── primary.rs               # Literals, variables, parentheses (200 lines)
├── operators.rs             # Binary/unary operators (200 lines)
├── function_calls.rs        # Function call parsing (200 lines)
├── object_access.rs         # Property access, method calls (200 lines)
├── special.rs               # Already exists (179 lines) - needs review
├── assignments.rs           # Assignment operators (150 lines)
└── postfix.rs              # Already exists (143 lines) - needs review
```

**Refactoring Steps:**

1. **Extract expression categories** by type
2. **Create specialized parsers** for each category
3. **Maintain precedence** logic in `mod.rs`
4. **Test expression parsing** extensively

**Success Criteria:**
- `parser/expr/primary.rs` reduced to 200 lines
- All new modules under 300 lines
- All tests pass

#### 1.4 Refactor `src/vm/compiler/definitions.rs` (779 lines)

**Current Structure Issues:**
- Mix of class, interface, trait, enum compilation
- Complex property/method handling

**Proposed Structure:**
```
src/vm/compiler/
├── definitions/
│   ├── mod.rs              # Definitions dispatcher (100 lines)
│   ├── classes.rs          # Class compilation (250 lines)
│   ├── interfaces.rs       # Interface compilation (200 lines)
│   ├── traits.rs           # Trait compilation (200 lines)
│   ├── enums.rs            # Enum compilation (150 lines)
│   ├── properties.rs       # Property handling (200 lines)
│   ├── methods.rs          # Method handling (200 lines)
│   └── visibility.rs       # Visibility modifiers (100 lines)
```

**Refactoring Steps:**

1. **Create subdirectory** `compiler/definitions/`
2. **Split by definition type**
3. **Extract common patterns** to helper functions
4. **Test each definition type** independently

**Success Criteria:**
- `compiler/definitions.rs` reduced to 100 lines
- All new modules under 300 lines
- All tests pass

#### 1.5 Refactor `src/vm/compiler/expr.rs` (682 lines)

**Current Structure Issues:**
- All expression types in one file
- Complex branching logic

**Proposed Structure:**
```
src/vm/compiler/expr/
├── mod.rs                   # Expression dispatcher (150 lines)
├── literals.rs             # Literals, variables (150 lines)
├── binary_ops.rs           # Binary operators (200 lines)
├── unary_ops.rs            # Unary operators (150 lines)
├── function_calls.rs       # Function call compilation (200 lines)
├── object_access.rs        # Property/method access (200 lines)
├── assignments.rs          # Assignment expressions (150 lines)
└── control_flow.rs         # Ternary, null coalescing (150 lines)
```

**Refactoring Steps:**

1. **Create subdirectory** `compiler/expr/`
2. **Split by expression type**
3. **Extract common helpers** to `expr_helpers.rs`
4. **Test each expression type** independently

**Success Criteria:**
- `compiler/expr.rs` reduced to 150 lines
- All new modules under 300 lines
- All tests pass

### Phase 2: Address Approaching Limit Files (Priority: MEDIUM)

#### 2.1 Review and Refactor `src/runtime/value.rs` (665 lines)

**Assessment:**
- Contains Value enum + trait impls
- May need to split trait implementations

**Proposed Split:**
```
src/runtime/
├── value.rs                 # Value enum, basic constructors (200 lines)
├── value_ops.rs             # Value operations (arithmetic, etc.) (200 lines)
├── value_conversion.rs     # Type conversion (150 lines)
└── value_comparison.rs      # Comparison operations (150 lines)
```

#### 2.2 Review and Refactor `src/vm/opcode.rs` (552 lines)

**Assessment:**
- Contains all opcode definitions
- May need splitting by category

**Proposed Split:**
```
src/vm/
├── opcode.rs                # Opcode enum (200 lines)
├── opcode_info.rs           # Opcode metadata (150 lines)
└── opcode_print.rs          # Opcode display/debug (150 lines)
```

#### 2.3 Review and Refactor `src/vm/ops/object_ops.rs` (490 lines)

**Assessment:**
- All object operations
- May need splitting by operation type

**Proposed Split:**
```
src/vm/ops/
├── object_ops.rs            # Property access (150 lines)
├── method_ops.rs            # Method calls (150 lines)
└── class_ops.rs             # Class operations (150 lines)
```

### Phase 3: Establish Preventive Measures (Priority: HIGH)

#### 3.1 CI File Size Check

**Implementation:**
```bash
# Add to Makefile or CI script
check-file-sizes:
	@echo "Checking file sizes..."
	@find src -name "*.rs" -exec sh -c 'lines=$$(wc -l < "$$1"); if [ $$lines -gt 500 ]; then echo "$$1: $$lines lines (OVER LIMIT)"; exit 1; elif [ $$lines -gt 300 ]; then echo "$$1: $$lines lines (WARNING)"; fi' _ {} \;

# Add to lint target
lint: check-file-sizes
```

#### 3.2 File Size Monitoring

**Implementation:**
- Add `make check-file-sizes` to CI pipeline
- Fail builds if files exceed 500 lines
- Warn if files exceed 300 lines

#### 3.3 Documentation Guidelines

**Update AGENTS.md:**
```markdown
## Adding New Features

### File Size Guidelines

- Keep new files under 300 lines
- When adding code to existing files, check file size first
- If file exceeds 300 lines, create new module instead
- Split files over 500 lines immediately

### Creating New Modules

1. Identify logical grouping
2. Create new module file
3. Move related code
4. Update mod.rs exports
5. Update imports across codebase
6. Run tests
7. Check file sizes again
```

#### 3.4 Code Review Checklist

Add to PR template:
```markdown
## Code Review Checklist

- [ ] No file exceeds 500 lines
- [ ] New files are under 300 lines
- [ ] File organization follows single responsibility principle
- [ ] Module boundaries are clear
```

### Phase 4: Ongoing Maintenance (Priority: MEDIUM)

#### 4.1 Regular File Size Audits

**Schedule:**
- Weekly automated check in CI
- Monthly manual review during retrospectives

#### 4.2 Refactoring Tickets

**Process:**
1. When file exceeds 400 lines, create refactoring ticket
2. Prioritize by file size (largest first)
3. Schedule during regular maintenance periods
4. Ensure tests pass before/after refactoring

#### 4.3 Education and Training

**Topics:**
- Module design principles
- Code organization patterns
- Refactoring techniques
- Testing strategies during refactoring

## Implementation Details

### File Splitting Patterns

#### Pattern 1: By Functionality
```rust
// Before: large.rs (800 lines)
mod large {
    fn feature_a() { /* 200 lines */ }
    fn feature_b() { /* 200 lines */ }
    fn feature_c() { /* 200 lines */ }
    fn feature_d() { /* 200 lines */ }
}

// After:
// mod.rs
pub mod feature_a;
pub mod feature_b;
pub mod feature_c;
pub mod feature_d;

// feature_a.rs (200 lines)
pub fn feature_a() { /* ... */ }
```

#### Pattern 2: By Data Type
```rust
// Before: values.rs (800 lines)
pub enum Value {
    Int(i64),
    String(String),
    // ...
}

impl Value {
    fn arithmetic_ops(&self) { /* 300 lines */ }
    fn comparison_ops(&self) { /* 300 lines */ }
    fn conversion_ops(&self) { /* 200 lines */ }
}

// After:
// value.rs (200 lines)
pub enum Value {
    Int(i64),
    String(String),
    // ...
}

// value_arithmetic.rs (300 lines)
impl Value {
    pub fn arithmetic_ops(&self) { /* ... */ }
}

// value_comparison.rs (300 lines)
impl Value {
    pub fn comparison_ops(&self) { /* ... */ }
}

// value_conversion.rs (200 lines)
impl Value {
    pub fn conversion_ops(&self) { /* ... */ }
}
```

#### Pattern 3: By Abstraction Level
```rust
// Before: parser.rs (900 lines)
mod parser {
    fn token_stream() { /* 100 lines */ }
    fn ast_construction() { /* 300 lines */ }
    fn error_handling() { /* 100 lines */ }
    fn parse_expression() { /* 200 lines */ }
    fn parse_statement() { /* 200 lines */ }
}

// After:
// mod.rs (200 lines)
pub mod tokens;
pub mod ast;
pub mod errors;
pub mod expr_parser;
pub mod stmt_parser;

// tokens.rs (100 lines)
// ast.rs (300 lines)
// errors.rs (100 lines)
// expr_parser.rs (200 lines)
// stmt_parser.rs (200 lines)
```

### Testing Strategy

#### During Refactoring

1. **Before splitting:**
   - Run full test suite: `make test`
   - Ensure all tests pass
   - Document current behavior

2. **After splitting:**
   - Run full test suite: `make test`
   - Ensure all tests pass
   - Verify no functionality lost
   - Check for compile errors

3. **Integration tests:**
   - Test cross-module interactions
   - Verify public API unchanged
   - Test module boundaries

#### File Size Tests

```bash
# Test file sizes
test-file-sizes:
	@find src -name "*.rs" | while read f; do \
		lines=$$(wc -l < "$$f"); \
		if [ $$lines -gt 500 ]; then \
			echo "FAIL: $$f has $$lines lines (max 500)"; \
			exit 1; \
		fi; \
	done
	@echo "PASS: All files under 500 lines"
```

## Dependencies

### Internal Dependencies

- Existing test suite
- CI/CD pipeline
- Development workflow

### External Dependencies

None required - this is code organization only

## Success Criteria

### Phase 1 Success Criteria
- [ ] `src/vm/mod.rs` reduced to 200-300 lines
- [ ] `src/parser/stmt/mod.rs` reduced to 150-200 lines
- [ ] `src/parser/expr/primary.rs` reduced to 200 lines
- [ ] `src/vm/compiler/definitions.rs` reduced to 100 lines
- [ ] `src/vm/compiler/expr.rs` reduced to 150 lines
- [ ] All tests pass
- [ ] No functionality changes
- [ ] Documentation updated

### Phase 2 Success Criteria
- [ ] `src/runtime/value.rs` split appropriately
- [ ] `src/vm/opcode.rs` split appropriately
- [ ] `src/vm/ops/object_ops.rs` split appropriately
- [ ] All files under 500 lines
- [ ] All tests pass

### Phase 3 Success Criteria
- [ ] CI file size check implemented
- [ ] Build fails if files exceed 500 lines
- [ ] AGENTS.md updated with guidelines
- [ ] Code review checklist updated

### Phase 4 Success Criteria
- [ ] Weekly CI checks running
- [ ] Monthly audits scheduled
- [ ] Refactoring process documented
- [ ] Team training complete

## Metrics to Track

- Number of files exceeding 300 lines
- Number of files exceeding 500 lines
- Average file size per module
- Time spent on refactoring
- Number of refactoring tickets created
- Code quality metrics (maintainability index)

## Open Questions

1. **Strict enforcement** - Should 500-line limit be enforced in CI or just warning?
2. **Exceptions** - Are there valid reasons for files over 500 lines?
3. **Migration priority** - Which file should be split first?
4. **Testing coverage** - Need more integration tests for cross-module interactions?

## Related Documentation

- [AGENTS.md](../../AGENTS.md) - Project instructions
- [README.md](../../README.md) - Project overview
- [architecture.md](../../docs/architecture.md) - Architecture documentation

## References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Clean Code by Robert C. Martin](https://www.oreilly.com/library/view/clean-code-a/9780136083238/)
- [Refactoring by Martin Fowler](https://refactoring.com/)

## Progress Tracking

### Phase 1: Immediate Large File Refactoring
- [ ] Refactor `src/vm/mod.rs` (4,295 lines)
- [ ] Refactor `src/parser/stmt/mod.rs` (1,504 lines)
- [ ] Refactor `src/parser/expr/primary.rs` (937 lines)
- [ ] Refactor `src/vm/compiler/definitions.rs` (779 lines)
- [ ] Refactor `src/vm/compiler/expr.rs` (682 lines)

### Phase 2: Address Approaching Limit Files
- [ ] Review `src/runtime/value.rs` (665 lines)
- [ ] Review `src/vm/opcode.rs` (552 lines)
- [ ] Review `src/vm/ops/object_ops.rs` (490 lines)

### Phase 3: Establish Preventive Measures
- [ ] Implement CI file size check
- [ ] Add file size monitoring
- [ ] Update AGENTS.md guidelines
- [ ] Update code review checklist

### Phase 4: Ongoing Maintenance
- [ ] Schedule regular audits
- [ ] Create refactoring process
- [ ] Document best practices
- [ ] Provide team training

**Total: 0/19 tasks completed**

## Next Steps

1. Review and approve this plan
2. Prioritize Phase 1 files for refactoring
3. Begin with `src/vm/mod.rs` (largest file)
4. Implement CI checks
5. Update documentation
6. Establish ongoing maintenance process
