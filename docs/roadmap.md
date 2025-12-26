---
layout: default
title: Roadmap
nav_order: 6
---

# Roadmap

VHP is being developed incrementally, with each phase adding new capabilities while maintaining backwards compatibility.

## Development Phases

| Phase | Status | Features |
|-------|--------|----------|
| **1. Variables & Operators** | ✅ Complete | Variables, assignment, arithmetic, comparison, logical, ternary, null coalescing |
| **2. Control Flow** | ✅ Complete | `if`/`else`, `while`, `for`, `do-while`, `switch`, `break`/`continue` |
| **3. Functions** | ✅ Complete | Declarations, calls, returns, parameters, 50+ built-ins |
| **4. Arrays** | 🚧 Next | Literals, access, modification, `foreach` iteration |
| **5. Classes & Objects** | 📋 Planned | Classes, properties, methods, inheritance, interfaces |
| **6. VHP Extensions** | 💡 Future | Type inference, pattern matching, async/await |

## Phase Details

### Phase 1: Variables & Operators ✅

- Variables (`$name`)
- Assignment (`=`) and compound assignment (`+=`, `-=`, etc.)
- Arithmetic operators (`+`, `-`, `*`, `/`, `%`, `**`)
- String concatenation (`.`)
- Comparison operators (`==`, `===`, `!=`, `!==`, `<`, `>`, `<=`, `>=`, `<=>`)
- Logical operators (`&&`, `||`, `!`, `and`, `or`, `xor`)
- Null coalescing (`??`)
- Ternary operator (`? :`)
- Increment/decrement (`++`, `--`)

### Phase 2: Control Flow ✅

- `if`/`elseif`/`else` statements
- `while` loops
- `do`...`while` loops
- `for` loops
- `foreach` loops (syntax parsing - requires arrays for full support)
- `switch`/`case`/`default`
- `break`/`continue`

### Phase 3: Functions ✅

- Function declarations
- Function calls
- Return statements
- Parameters (by value, by reference)
- Default parameter values
- 50+ built-in functions

### Phase 4: Arrays 🚧

- Array literals (`[]`, `array()`)
- Array access (`$arr[0]`, `$arr['key']`)
- Array modification
- `foreach` with arrays
- Array functions (`array_push`, `array_pop`, `count`, etc.)

### Phase 5: Classes & Objects 📋

- Class declarations
- Properties and methods
- Constructors
- Visibility (public, private, protected)
- Inheritance
- Interfaces and traits
- Static members

### Phase 6: VHP Extensions 💡

Beyond PHP compatibility, VHP aims to introduce modern language features:

- **Type inference** - Automatic type detection
- **Pattern matching** - More powerful switch alternatives
- **Null coalescing improvements** - Enhanced null safety
- **Async/await** - Native asynchronous programming
- **Better error messages** - Developer-friendly diagnostics

## Contributing to the Roadmap

Have ideas for VHP? Feel free to:
- Open an issue to discuss new features
- Submit a pull request with an implementation
- Join the discussion on existing roadmap items
