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
| **4. Arrays** | ✅ Complete | Literals, access, modification, `foreach`, 13 array functions |
| **5. Classes & Objects** | ✅ Complete | Classes, properties, methods, constructors, `$this`, static calls |
| **6. Modern PHP 8.x Features** | 🚧 In Progress | Match Expressions ✅, Named Arguments, Enums, Fibers, Pipe Operator |

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

### Phase 4: Arrays ✅

- Array literals (`[1, 2, 3]`)
- Associative arrays (`["key" => "value"]`)
- Array access (`$arr[0]`, `$arr['key']`)
- Array modification and append (`$arr[] = value`)
- `foreach` with arrays (value only and key-value)
- 13 array functions (`count`, `array_push`, `array_pop`, `in_array`, `array_keys`, `array_values`, `array_merge`, `array_reverse`, `array_search`, `array_key_exists`, `range`, etc.)

### Phase 5: Classes & Objects ✅

- Class declarations with `class` keyword
- Properties with visibility modifiers (`public`, `private`, `protected`)
- Methods with `$this` reference
- Constructors (`__construct`)
- Object instantiation with `new`
- Property access (`$obj->property`)
- Method calls (`$obj->method()`)
- Static method calls (`ClassName::method()`)
- Default property values
- Multiple objects from same class
- Inheritance (`extends`) with property/method inheritance and `parent::` calls
- Interfaces with method signatures and constants
- Interface inheritance (`extends Interface1, Interface2`)
- Class implementation of interfaces (`implements Interface1, Interface2`)
- Traits with properties and methods
- Trait composition in classes (`use Trait1, Trait2`)
- Trait conflict resolution (`insteadof`, `as`)
- Traits using other traits

**Remaining for Phase 5:**
- ✅ **Constructor Property Promotion** (PHP 8.0) - Shorthand syntax for declaring and initializing properties in constructor.
- Readonly Properties (PHP 8.1) & Classes (PHP 8.2)
- "Clone with" functionality (PHP 8.5)

### Phase 6: Modern PHP 8.x Features 🚧

This phase focuses on catching up with major features introduced in PHP 8.0 and beyond.

- ✅ **Match Expressions** (PHP 8.0) - A more powerful and safer alternative to `switch`.
- ✅ **Named Arguments** (PHP 8.0) - Pass arguments to functions based on parameter names.
- **Attributes** (PHP 8.0) - Structured metadata for classes, methods, and functions.
- **Enums** (PHP 8.1) - Support for strongly-typed enumerations.
- **Fibers** (PHP 8.1) - The foundation for lightweight, cooperative concurrency (async/await).
- **Pipe Operator** (PHP 8.5) - A functional-style operator for chaining method calls.

## Contributing to the Roadmap

Have ideas for VHP? Feel free to:
- Open an issue to discuss new features
- Submit a pull request with an implementation
- Join the discussion on existing roadmap items
