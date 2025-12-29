# Plan: Implement Interfaces and Traits for VHP

## Overview

Add PHP 8.x-compatible **Interfaces** and **Traits** to the VHP interpreter. This completes the "Remaining for Phase 5" item in the roadmap.

## Implementation Phases

The implementation is split into two phases: **Interfaces first**, then **Traits**.

---

## Phase A: Interfaces

### A.1 Token Additions
**File:** `src/token.rs` (add after line 37)

```rust
Interface,    // interface
Implements,   // implements
```

### A.2 Lexer Changes
**File:** `src/lexer.rs` (in `keyword_or_identifier()` match block)

```rust
"interface" => TokenKind::Interface,
"implements" => TokenKind::Implements,
```

### A.3 AST Additions
**File:** `src/ast/stmt.rs`

Add new structs:
```rust
/// Interface method signature (no body)
#[derive(Debug, Clone)]
pub struct InterfaceMethodSignature {
    pub name: String,
    pub params: Vec<FunctionParam>,
}

/// Interface constant
#[derive(Debug, Clone)]
pub struct InterfaceConstant {
    pub name: String,
    pub value: Expr,
}
```

Add `Stmt::Interface` variant:
```rust
Interface {
    name: String,
    parents: Vec<String>,  // extends Interface1, Interface2
    methods: Vec<InterfaceMethodSignature>,
    constants: Vec<InterfaceConstant>,
},
```

Modify `Stmt::Class` to add `interfaces`:
```rust
Class {
    name: String,
    parent: Option<String>,
    interfaces: Vec<String>,  // NEW: implements Foo, Bar
    properties: Vec<Property>,
    methods: Vec<Method>,
},
```

### A.4 Parser Changes
**File:** `src/parser/stmt.rs`

1. Add `parse_interface()` method - parse interface declaration
2. Add `parse_interface_method()` - parse method signature (no body, ends with `;`)
3. Add `parse_interface_constant()` - parse `const NAME = value;`
4. Modify `parse_class()` to parse `implements Interface1, Interface2` after extends
5. Add `TokenKind::Interface` case to `parse_statement()`

### A.5 Interpreter Changes
**File:** `src/interpreter/mod.rs`

Add `InterfaceDefinition` struct:
```rust
pub struct InterfaceDefinition {
    pub name: String,
    pub parents: Vec<String>,
    pub methods: Vec<(String, Vec<FunctionParam>)>,  // (name, params)
    pub constants: HashMap<String, Value>,
}
```

Add `interfaces: HashMap<String, InterfaceDefinition>` to `Interpreter` struct.

Add handler for `Stmt::Interface`:
- Validate parent interfaces exist
- Collect method signatures (inherit from parents)
- Store interface definition

Update `Stmt::Class` handler:
- Validate all implemented interfaces exist
- Verify class implements ALL methods from each interface
- Validate parameter counts match

### A.6 Interface Tests
**Directory:** `tests/interfaces/`

| Test File | Description |
|-----------|-------------|
| `interface_basic.vhpt` | Basic declaration + implements |
| `interface_multiple.vhpt` | `implements A, B` |
| `interface_extends.vhpt` | Interface extending interface |
| `interface_extends_multiple.vhpt` | `interface A extends B, C` |
| `interface_constants.vhpt` | `const VERSION = "1.0";` |
| `interface_missing_method_error.vhpt` | Error when method not implemented |
| `interface_missing_error.vhpt` | Error when interface doesn't exist |

---

## Phase B: Traits

### B.1 Token Additions
**File:** `src/token.rs` (add after Interface/Implements)

```rust
Trait,        // trait
Use,          // use (for traits in class)
Insteadof,    // insteadof
```

Note: `As` token already exists (line 19).

### B.2 Lexer Changes
**File:** `src/lexer.rs`

```rust
"trait" => TokenKind::Trait,
"use" => TokenKind::Use,
"insteadof" => TokenKind::Insteadof,
```

### B.3 AST Additions
**File:** `src/ast/stmt.rs`

Add trait use and conflict resolution structs:
```rust
/// Trait usage in class
#[derive(Debug, Clone)]
pub struct TraitUse {
    pub traits: Vec<String>,
    pub resolutions: Vec<TraitResolution>,
}

/// Conflict resolution
#[derive(Debug, Clone)]
pub enum TraitResolution {
    InsteadOf {
        trait_name: String,
        method: String,
        excluded_traits: Vec<String>,
    },
    Alias {
        trait_name: Option<String>,
        method: String,
        alias: String,
        visibility: Option<Visibility>,
    },
}
```

Add `Stmt::Trait` variant:
```rust
Trait {
    name: String,
    uses: Vec<String>,  // traits using other traits
    properties: Vec<Property>,
    methods: Vec<Method>,
},
```

Modify `Stmt::Class` to add `trait_uses`:
```rust
Class {
    name: String,
    parent: Option<String>,
    interfaces: Vec<String>,
    trait_uses: Vec<TraitUse>,  // NEW
    properties: Vec<Property>,
    methods: Vec<Method>,
},
```

### B.4 Parser Changes
**File:** `src/parser/stmt.rs`

1. Add `parse_trait()` - parse trait declaration (like class but no parent)
2. Add `parse_trait_use()` - parse `use TraitA, TraitB { ... };`
3. Add `parse_trait_resolution()` - parse `insteadof` and `as` clauses
4. Modify `parse_class()` to parse trait uses at start of class body
5. Add `TokenKind::Trait` case to `parse_statement()`

### B.5 Interpreter Changes
**File:** `src/interpreter/mod.rs`

Add `TraitDefinition` struct:
```rust
pub struct TraitDefinition {
    pub name: String,
    pub uses: Vec<String>,
    pub properties: Vec<Property>,
    pub methods: HashMap<String, UserFunction>,
    pub method_visibility: HashMap<String, Visibility>,
}
```

Add `traits: HashMap<String, TraitDefinition>` to `Interpreter` struct.

Add handler for `Stmt::Trait`:
- Import methods from used traits
- Add own methods (overriding)
- Store trait definition

Update `Stmt::Class` handler for trait composition:
- Collect methods from all used traits
- Apply `insteadof` resolutions (exclude methods)
- Apply `as` resolutions (create aliases, change visibility)
- Detect unresolved conflicts (error)
- Add trait properties to class
- Class methods override trait methods

### B.6 Trait Tests
**Directory:** `tests/traits/`

| Test File | Description |
|-----------|-------------|
| `trait_basic.vhpt` | Basic trait declaration and use |
| `trait_properties.vhpt` | Trait with properties |
| `trait_multiple.vhpt` | `use A, B;` |
| `trait_override.vhpt` | Class method overrides trait |
| `trait_insteadof.vhpt` | Conflict resolution with `insteadof` |
| `trait_alias.vhpt` | `method as aliasName;` |
| `trait_visibility.vhpt` | `method as protected;` |
| `trait_use_trait.vhpt` | Trait using another trait |
| `trait_conflict_error.vhpt` | Error on unresolved conflict |

---

## Critical Files to Modify

| File | Changes |
|------|---------|
| `src/token.rs` | Add 5 tokens: Interface, Implements, Trait, Use, Insteadof |
| `src/lexer.rs` | Add keyword recognition |
| `src/ast/stmt.rs` | Add Interface, Trait AST nodes; modify Class |
| `src/parser/stmt.rs` | Add parse_interface(), parse_trait(), parse_trait_use() |
| `src/interpreter/mod.rs` | Add InterfaceDefinition, TraitDefinition; update execute_stmt |
| `tests/interfaces/*.vhpt` | 7 new test files |
| `tests/traits/*.vhpt` | 9 new test files |
| `AGENTS.md` | Update features and roadmap |
| `README.md` | Update features section |
| `docs/roadmap.md` | Mark interfaces/traits complete |
| `docs/features.md` | Add interfaces/traits documentation |

---

## Implementation Order

1. **Interfaces - Tokens & Lexer**: Add Interface, Implements tokens
2. **Interfaces - AST**: Add InterfaceMethodSignature, InterfaceConstant, Stmt::Interface
3. **Interfaces - Parser**: Add parse_interface(), update parse_class() for implements
4. **Interfaces - Interpreter**: Add InterfaceDefinition, registration, validation
5. **Interfaces - Tests**: Create tests/interfaces/ with 7 test files
6. **Traits - Tokens & Lexer**: Add Trait, Use, Insteadof tokens
7. **Traits - AST**: Add TraitUse, TraitResolution, Stmt::Trait
8. **Traits - Parser**: Add parse_trait(), parse_trait_use(), parse_trait_resolution()
9. **Traits - Interpreter**: Add TraitDefinition, composition logic
10. **Traits - Tests**: Create tests/traits/ with 9 test files
11. **Documentation**: Update AGENTS.md, README.md, docs/

---

## Success Criteria

- [ ] All new tests pass (`make test`)
- [ ] Existing tests still pass (no regressions)
- [ ] `make lint` passes
- [ ] Documentation updated
