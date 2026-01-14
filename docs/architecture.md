---
layout: default
title: Architecture
nav_order: 7
---

# Architecture

VHP follows a classic compiler pipeline with four main stages, ending in a bytecode virtual machine for efficient execution.

## Pipeline Overview

```
┌─────────────┐    ┌─────────┐    ┌────────┐    ┌─────────────┐    ┌───────┐
│ Source Code │───▶│  Lexer  │───▶│ Parser │───▶│  Compiler   │───▶│   VM  │───▶ Output
└─────────────┘    └─────────┘    └────────┘    └─────────────┘    └───────┘
                       │              │               │               │
                   Tokens          AST         Bytecode       Execute
```

1. **Lexer** (`lexer/`): Converts source text into tokens, handles PHP/HTML mode switching
2. **Parser** (`parser/`): Builds AST from tokens using recursive descent with Pratt parsing for operator precedence
3. **Compiler** (`vm/compiler/`): Compiles AST to bytecode instructions
4. **VM** (`vm/`): Executes bytecode with stack-based virtual machine

## Project Structure

```
src/
├── main.rs              # CLI entry point, argument parsing
├── token.rs             # Token type definitions (TokenKind, Token)
├── lexer/               # Lexical analysis (modularized)
│   ├── mod.rs           # Main lexer logic
│   ├── strings.rs       # String tokenization
│   └── operators.rs     # Operator recognition
├── test_runner.rs       # .vhpt test framework
├── ast/                 # Abstract Syntax Tree (modularized)
│   ├── mod.rs           # Module exports
│   ├── expr.rs          # Expression AST nodes
│   ├── stmt.rs          # Statement AST nodes
│   └── ops.rs           # Operator definitions
├── parser/              # Recursive descent parser (modularized)
│   ├── mod.rs           # Module exports
│   ├── precedence.rs    # Operator precedence (Pratt parsing)
│   ├── expr/            # Expression parsing
│   │   ├── mod.rs       # Expression dispatcher
│   │   ├── literals_parsing.rs
│   │   ├── arrow_anonymous_parsing.rs
│   │   ├── callable_parsing.rs
│   │   ├── postfix.rs
│   │   └── special.rs
│   └── stmt/            # Statement parsing
│       ├── mod.rs       # Statement dispatcher
│       ├── attribute_parsing.rs
│       ├── class.rs
│       ├── control_flow.rs
│       ├── declarations.rs
│       ├── enum_.rs
│       ├── interface.rs
│       ├── member_parsing.rs
│       ├── namespace_parsing.rs
│       ├── trait_.rs
│       └── type_parsing.rs
├── runtime/             # Value types and built-in functions
│   ├── mod.rs           # Runtime exports and types
│   ├── value/           # Value type definitions
│   │   ├── mod.rs       # Value enum and core methods
│   │   ├── array_key.rs # Array key type
│   │   ├── object_instance.rs # ObjectInstance, ExceptionValue
│   │   └── value_helpers.rs   # Value coercion helpers
│   └── builtins/        # Built-in function modules
│       ├── mod.rs       # Module exports
│       ├── array.rs     # Array functions (20)
│       ├── fileio.rs    # File I/O functions (10)
│       ├── json.rs      # JSON functions (2)
│       ├── math.rs      # Math functions (16)
│       ├── output.rs    # Output functions (4)
│       ├── reflection.rs # Reflection functions (8)
│       ├── string.rs    # String functions (23)
│       ├── types.rs     # Type functions (14)
│       └── pcre.rs      # PCRE regex functions (stub)
└── vm/                  # Bytecode Virtual Machine (primary execution engine)
    ├── mod.rs           # VM struct, main execution loop dispatcher
    ├── execution.rs     # VM execution loop
    ├── opcode.rs        # Opcode definitions
    ├── frame.rs         # Call frames and loop contexts
    ├── class.rs         # Class definition types
    ├── class_registration.rs # Built-in class registration
    ├── compiled_types.rs # CompiledFunction, Constant
    ├── methods.rs       # Method definition types
    ├── objects.rs       # Object instantiation and cloning
    ├── helpers.rs       # VM helper functions
    ├── reflection.rs    # Runtime reflection support
    ├── builtins.rs      # Built-in function bridge
    ├── type_validation.rs # Type hint validation
    ├── ops/             # Opcode execution modules (12 modules)
    │   ├── mod.rs       # Module exports
    │   ├── arithmetic.rs # Arithmetic opcode handlers
    │   ├── arrays.rs    # Array opcode handlers
    │   ├── call_ops.rs  # Function call opcodes
    │   ├── callable_ops.rs # First-class callable opcodes
    │   ├── comparison.rs # Comparison opcode handlers
    │   ├── control_flow.rs # Control flow opcode handlers
    │   ├── exceptions.rs # Exception opcode handlers
    │   ├── logical_bitwise.rs # Logical/bitwise handlers
    │   ├── method_calls.rs # Method call opcodes
    │   ├── misc.rs      # Miscellaneous opcode handlers
    │   ├── named_call_ops.rs # Named argument call opcodes
    │   ├── object_creation.rs # Object/class creation
    │   ├── property_access.rs # Property access handlers
    │   ├── property_ops.rs # Property operation handlers
    │   ├── static_ops.rs # Static property/method handlers
    │   └── strings.rs   # String opcode handlers
    └── compiler/        # AST to bytecode compiler (12 modules)
        ├── mod.rs       # Main compiler struct
        ├── assignment_compilation.rs # Variable assignment
        ├── class_compilation.rs # Class definition compilation
        ├── compiler_types.rs # Type/name resolution
        ├── expr.rs      # Expression compilation
        ├── expr_helpers.rs # Expression compilation helpers
        ├── functions.rs # Function/closure compilation
        ├── if_match.rs  # if/match/switch compilation
        ├── interface_compilation.rs # Interface compilation
        ├── loops.rs     # Loop compilation
        ├── object_access_compilation.rs # Property access compilation
        ├── stmt.rs      # Statement dispatcher
        ├── trait_enum_compilation.rs # Trait/enum compilation
        └── try_catch.rs # try/catch/finally compilation

tests/                   # Test suite organized by feature
├── arrays/              # Array tests
├── attributes/          # Attribute syntax and reflection tests
├── builtins/            # Built-in function tests
├── classes/             # Class and object tests
├── comments/            # Comment syntax tests
├── control_flow/        # Control flow tests
├── echo/                # Echo statement tests
├── enums/               # Enum tests
├── errors/              # Error handling tests
├── exceptions/          # Exception handling tests
├── expressions/         # Expression evaluation tests
├── fibers/              # Fiber tests
├── fileio/              # File I/O tests
├── functions/           # User-defined function tests
├── generators/          # Generator tests
├── html/                # HTML passthrough tests
├── interfaces/          # Interface tests
├── json/                # JSON tests
├── namespaces/          # Namespace tests
├── numbers/             # Numeric literal tests
├── operators/           # Operator tests
├── strings/             # String literal and escape sequence tests
├── tags/                # PHP tag tests
├── traits/              # Trait tests
├── types/               # Type declaration and validation tests
└── variables/           # Variable assignment and scope tests
```

## Components

### Token (`token.rs`)

Defines all token types recognized by the lexer:

```rust
pub enum TokenKind {
    // Keywords
    Echo, If, Else, While, For, Function, Return, Match, Class, Trait, Interface,
    Enum, Abstract, Final, Static, Public, Private, Protected,
    // ... and many more including PHP 8.x keywords

    // Literals
    String(String),
    Integer(i64),
    Float(f64),
    True, False, Null,

    // Operators
    Plus, Minus, Star, Slash, Mod, Pow,
    // ... and more including pipe operator (|>) for PHP 8.5
}
```

### Lexer (`lexer/`)

Converts source code into a stream of tokens:

- **mod.rs**: Main lexer logic and tokenization entry point
- **strings.rs**: String tokenization with escape sequence handling
- **operators.rs**: Operator recognition including pipe operator (PHP 8.5)

Features:
- Handles PHP/HTML mode switching
- Recognizes keywords, literals, and operators
- Tracks line and column positions for error reporting
- Supports escape sequences in strings (`\n`, `\t`, `\\`, etc.)

### AST (`ast/`)

Defines the abstract syntax tree structure in separate modules:

```rust
// ast/expr.rs - Expression nodes (236 lines)
pub enum Expr {
    // Literals
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Null,

    // Variable
    Variable(String),

    // Array literal
    Array(Vec<ArrayElement>),

    // Operations
    Binary { left: Box<Expr>, op: BinaryOp, right: Box<Expr> },
    Unary { op: UnaryOp, expr: Box<Expr> },
    Assign { var: String, op: AssignOp, value: Box<Expr> },
    ArrayAssign { array: Box<Expr>, index: Option<Box<Expr>>, op: AssignOp, value: Box<Expr> },

    // Control flow
    Ternary { condition: Box<Expr>, then_expr: Box<Expr>, else_expr: Box<Expr> },
    Match { expr: Box<Expr>, arms: Vec<MatchArm>, default: Option<Box<Expr>> },

    // Function/Method calls
    FunctionCall { name: String, args: Vec<Argument> },
    MethodCall { object: Box<Expr>, method: String, args: Vec<Argument> },
    New { class_name: String, args: Vec<Argument> },

    // First-class callables
    CallableFromFunction(String),
    CallableFromMethod { object: Box<Expr>, method: String },
    CallableFromStaticMethod { class: String, method: String },

    // Arrow functions (PHP 7.4)
    ArrowFunction { params: Vec<FunctionParam>, body: Box<Expr> },

    // Object-oriented
    PropertyAccess { object: Box<Expr>, property: String },
    StaticMethodCall { class_name: String, method: String, args: Vec<Argument> },
    StaticPropertyAccess { class: String, property: String },

    // Generators (PHP 5.5+)
    Yield { key: Option<Box<Expr>>, value: Option<Box<Expr>> },
    YieldFrom(Box<Expr>),

    // ... and more (EnumCase, Clone, CloneWith, FiberSuspend, etc.)
}

// ast/stmt.rs - Statement nodes (412 lines)
pub enum Stmt {
    Echo(Vec<Expr>),
    Expression(Expr),
    Html(String),
    If { condition: Expr, then_branch: Vec<Stmt>, elseif_branches: Vec<(Expr, Vec<Stmt>)>, else_branch: Option<Vec<Stmt>> },
    While { condition: Expr, body: Vec<Stmt> },
    DoWhile { body: Vec<Stmt>, condition: Expr },
    For { init: Option<Expr>, condition: Option<Expr>, update: Option<Expr>, body: Vec<Stmt> },
    Foreach { array: Expr, key: Option<String>, value: String, body: Vec<Stmt> },
    Switch { expr: Expr, cases: Vec<SwitchCase>, default: Option<Vec<Stmt>> },
    Break,
    Continue,
    Function { name: String, params: Vec<FunctionParam>, return_type: Option<TypeHint>, body: Vec<Stmt>, attributes: Vec<Attribute> },
    Return(Option<Expr>),
    Class { name: String, is_abstract: bool, is_final: bool, readonly: bool, parent: Option<QualifiedName>, interfaces: Vec<QualifiedName>, trait_uses: Vec<TraitUse>, properties: Vec<Property>, methods: Vec<Method>, attributes: Vec<Attribute> },
    Interface { name: String, parents: Vec<QualifiedName>, methods: Vec<InterfaceMethodSignature>, constants: Vec<InterfaceConstant>, attributes: Vec<Attribute> },
    Trait { name: String, uses: Vec<String>, properties: Vec<Property>, methods: Vec<Method>, attributes: Vec<Attribute> },
    Enum { name: String, backing_type: EnumBackingType, cases: Vec<EnumCase>, methods: Vec<Method>, attributes: Vec<Attribute> },
    TryCatch { try_body: Vec<Stmt>, catch_clauses: Vec<CatchClause>, finally_body: Option<Vec<Stmt>> },
    Throw(Expr),
    Namespace { name: Option<QualifiedName>, body: NamespaceBody },
    Use(Vec<UseItem>),
    GroupUse(GroupUse),
    Declare { directives: Vec<DeclareDirective>, body: Option<Vec<Stmt>> },
}

// ast/ops.rs - Operator definitions
pub enum BinaryOp { Add, Sub, Mul, Div, Mod, Pow, Concat, Eq, Ne, Identical, NotIdentical, Lt, Le, Gt, Ge, Spaceship, And, Or, Xor, BitwiseAnd, BitwiseOr, BitwiseXor, ShiftLeft, ShiftRight }
pub enum UnaryOp { Neg, Not, BitwiseNot, PreInc, PreDec, PostInc, PostDec }
pub enum AssignOp { Assign, AddAssign, SubAssign, MulAssign, DivAssign, ModAssign, ConcatAssign }
```

### Parser (`parser/`)

Builds AST from tokens using:

- **Recursive descent** for statements (in `parser/stmt/`)
- **Pratt parsing** for operator precedence in expressions (in `parser/expr/` and `parser/precedence.rs`)
- Modular structure with dedicated parsers for different language features

Key modules:
- `mod.rs`: Main parser entry point and dispatcher
- `precedence.rs`: Operator precedence table for Pratt parsing
- `expr/`: Expression parsing with sub-modules for literals, arrow functions, callables, postfix operations
- `stmt/`: Statement parsing including class, interface, trait, enum, control flow

### Runtime (`runtime/`)

Handles value representation and built-in functions:

**Value types** (`runtime/value/`):
- `Value` enum: Null, Bool, Integer, Float, String, Array, Object, Fiber, Closure, Generator, EnumCase, Exception
- `ArrayKey`: Integer or String keys for arrays
- `ObjectInstance`: Object properties and magic methods
- `Closure`: Captured variables for closures/arrow functions
- `FiberInstance`: Fiber state management
- `GeneratorInstance`: Generator state (partial implementation)

**Built-in functions** (`runtime/builtins/`):
- `string.rs` (364 lines): 23 string functions (strlen, substr, strtoupper, etc.)
- `math.rs` (195 lines): 16 math functions (abs, ceil, floor, sin, cos, tan, log, log10, exp, pi, etc.)
- `array.rs` (467 lines): 20 array functions (count, array_push, array_pop, etc.)
- `types.rs` (~120 lines): 14 type functions (intval, is_null, is_int, etc.)
- `output.rs` (193 lines): 4 output functions (print, var_dump, print_r, printf)
- `reflection.rs` (359 lines): 8 reflection functions for attributes
- `json.rs` (413 lines): json_encode, json_decode
- `fileio.rs` (159 lines): 10 file I/O functions

### VM (`vm/`)

The stack-based bytecode virtual machine that executes compiled PHP code:

**Core modules**:
- `mod.rs`: VM struct definition with stack, frames, globals, handlers
- `execution.rs`: Main execution loop with opcode dispatch
- `opcode.rs` (489 lines): Complete instruction set (~70 opcodes)
- `frame.rs`: Call frame and exception handler structures
- `type_validation.rs`: Runtime type hint validation

**Opcode execution** (`vm/ops/`): 12 modules handling different opcode categories
- `arithmetic.rs`: Add, Sub, Mul, Div, Mod, Pow, Neg
- `arrays.rs`: NewArray, ArrayPush, ArraySet, ArrayGet, ArrayAppend, ArrayUnpack
- `call_ops.rs`: Call, CallBuiltin, CallSpread, CallNamed
- `callable_ops.rs`: CallCallable for first-class callables
- `comparison.rs`: Eq, Ne, Identical, NotIdentical, Lt, Le, Gt, Ge, Spaceship
- `control_flow.rs`: Jump, JumpIfFalse, JumpIfTrue, LoopStart, Break, Continue
- `exceptions.rs`: TryStart, TryEnd, Throw, Catch, FinallyStart, FinallyEnd
- `logical_bitwise.rs`: Not, And, Or, Xor, BitwiseAnd/Or/Xor/Not, ShiftLeft/Right
- `method_calls.rs`: CallMethod, CallMethodOnLocal, CallStaticMethod
- `misc.rs`: Pop, Dup, Swap, Nop, Echo, Print, TypeCheck, InstanceOf
- `named_call_ops.rs`: CallNamed, CallStaticMethodNamed for named arguments
- `object_creation.rs`: NewObject, NewFiber, Clone, CallConstructor
- `property_access.rs`: LoadProperty, StoreProperty, IssetProperty, UnsetProperty
- `property_ops.rs`: Property assignment and modification
- `static_ops.rs`: LoadStaticProp, StoreStaticProp

**Compiler** (`vm/compiler/`): 12 modules for AST to bytecode compilation
- `mod.rs`: Main compiler struct with string pool and compilation entry point
- `stmt.rs`: Statement dispatcher
- `expr.rs`: Expression compilation with precedence handling
- `expr_helpers.rs`: Helper functions for expression compilation
- `functions.rs`: Function, closure, and arrow function compilation
- `if_match.rs`: if/elseif/else, match, and switch compilation
- `loops.rs`: while, do-while, for, foreach compilation
- `try_catch.rs`: try/catch/finally compilation
- `class_compilation.rs`: Class property and method compilation
- `interface_compilation.rs`: Interface method signatures
- `trait_enum_compilation.rs`: Trait and enum compilation
- `object_access_compilation.rs`: Property and method access compilation
- `assignment_compilation.rs`: Variable and property assignment
- `compiler_types.rs`: Type resolution utilities

## Design Principles

### Zero Dependencies

VHP uses only Rust's standard library. This ensures:
- Fast compilation
- Small binary size
- No supply chain risks
- Easy auditing

### Bytecode VM Architecture

Instead of tree-walking interpretation, VHP compiles PHP to bytecode and executes it with a stack-based VM:

1. **Compilation**: AST is compiled to bytecode instructions
2. **Execution**: VM executes instructions using an operand stack
3. **Frames**: Each function call creates a new call frame
4. **Optimization**: Constants are pooled, variables are indexed

Benefits:
- Faster repeated execution (no re-parsing)
- Better locality of reference
- Potential for JIT compilation in future

### PHP Compatibility

Every feature aims for PHP 8.x behavioral compatibility:
- Same type coercion rules
- Same operator precedence
- Same truthiness semantics
- Case-insensitive keywords and function names

### Incremental Development

Each feature is:
- Implemented in small, focused changes
- Covered by comprehensive tests
- Documented in this documentation
