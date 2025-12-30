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
| **4. Arrays** | ✅ Complete | Literals, access, modification, `foreach`, 15 array functions |
| **5. Classes & Objects** | ✅ Complete | Classes, properties, methods, constructors, inheritance, interfaces, traits, readonly, property hooks, cloning |
| **6. Modern PHP 8.x Features** | ✅ Complete | Match Expressions ✅, Named Arguments ✅, Attributes ✅, Enums ✅, Pipe Operator ✅, Fibers ✅ |
| **7. PHP Core Language** | 🔄 In Progress | Exceptions ✅, Type System ✅ (runtime validation), Namespaces, Generators, Abstract/Final ✅, Magic Methods |
| **8. PHP 8.5 Features** | 🔄 In Progress | URI Extension, Clone with syntax, #[\NoDiscard], array_first/last ✅, Closures in constants |
| **9. Standard Library** | 📋 Planned | PCRE regex, sorting, array_map/filter/reduce, JSON, DateTime, file system functions |

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
- Constructor Property Promotion (PHP 8.0)
- Readonly Properties (PHP 8.1)
- Readonly Classes (PHP 8.2)
- Property hooks with get/set (PHP 8.4)
- Object cloning with `clone` keyword (PHP 5.0)
- Clone with property modification syntax (PHP 8.4)

### Phase 6: Modern PHP 8.x Features 🚧

This phase focuses on catching up with major features introduced in PHP 8.0 and beyond.

- ✅ **Match Expressions** (PHP 8.0) - A more powerful and safer alternative to `switch`.
- ✅ **Named Arguments** (PHP 8.0) - Pass arguments to functions based on parameter names.
- ✅ **Attributes** (PHP 8.0) - Structured metadata syntax parsing and AST storage. Full reflection API support.
- ✅ **Enums** (PHP 8.1) - Pure and backed enums with case access, properties, and built-in methods (`cases()`, `from()`, `tryFrom()`).
- ✅ **Pipe Operator** (PHP 8.5) - Functional-style operator for chaining function calls with left-to-right flow.
- ✅ **Fibers** (PHP 8.1) - The foundation for lightweight, cooperative concurrency (async/await).

### Phase 7: PHP Core Language Compatibility 📋

This phase focuses on implementing core PHP language features that are essential for PHP compatibility. These features have been part of PHP for many versions and are fundamental to running most PHP code.

#### Exception Handling ✅

- [x] **try/catch statements** - Basic exception handling with catch blocks
- [x] **throw keyword** - Throwing exceptions
- [x] **finally blocks** - Code that always executes regardless of exceptions
- [x] **Exception class** - The base exception class with getMessage() and getCode() methods
- [x] **Multiple catch blocks** - Catching different exception types
- [x] **Multi-catch** (PHP 7.1) - Catching multiple exception types in one block `catch (TypeA | TypeB $e)`
- [x] **Throw expression** (PHP 8.0) - Using throw in expressions (arrow functions, null coalesce, ternary)

#### Type System ✅

- [x] **Type declarations** - Parameter and return type hints (`int`, `string`, `float`, `bool`, `array`, `callable`, `object`, `iterable`, `mixed`)
- [x] **Nullable types** (PHP 7.1) - `?int`, `?string` syntax
- [x] **Union types** (PHP 8.0) - `int|string`, `int|null`
- [x] **Intersection types** (PHP 8.1) - `Iterator&Countable` (parsing)
- [x] **void return type** (PHP 7.1) - Functions that return nothing
- [x] **never return type** (PHP 8.1) - Functions that never return (throw or exit)
- [x] **static return type** (PHP 8.0) - Return type for late static binding
- [x] **Runtime type validation** - Full enforcement of parameter and return types
- [x] **Class type hints** - Custom class/interface types
- [ ] **DNF types** (PHP 8.2) - Disjunctive Normal Form `(A&B)|C`
- [ ] **true/false/null as standalone types** (PHP 8.2)

#### Namespaces

- [ ] **namespace declaration** - `namespace App\Controllers;`
- [ ] **use statements** - `use App\Models\User;`
- [ ] **use aliases** - `use App\Models\User as UserModel;`
- [ ] **Group use declarations** (PHP 7.0) - `use App\Models\{User, Post};`
- [ ] **Global namespace fallback** - `\strlen()`, `\Exception`
- [ ] **Namespace constants** - `namespace\CONST_NAME`

#### Generators & Iterators

- [ ] **yield keyword** - Generator syntax
- [ ] **yield from** (PHP 7.0) - Generator delegation
- [ ] **Generator return values** (PHP 7.0) - `return` in generators
- [ ] **Iterator interface** - Custom iterators
- [ ] **IteratorAggregate** - Objects that return iterators

#### Abstract & Final

- [x] **abstract classes** - Classes that cannot be instantiated ✅
- [x] **abstract methods** - Methods that must be implemented by subclasses ✅
- [x] **final classes** - Classes that cannot be extended ✅
- [x] **final methods** - Methods that cannot be overridden ✅
- [ ] **final constants** (PHP 8.1) - Constants that cannot be overridden

#### Magic Methods

- [x] **__toString()** - String representation of objects ✅
- [x] **__invoke()** - Callable objects ✅
- [x] **__get()/__set()** - Property overloading ✅
- [x] **__isset()/__unset()** - Property checking ✅
- [x] **__call()/__callStatic()** - Method overloading ✅
- [x] **__clone()** - Custom clone behavior ✅
- [ ] **__debugInfo()** - Custom var_dump output
- [ ] **__sleep()/__wakeup()** - Serialization hooks
- [ ] **__serialize()/__unserialize()** (PHP 7.4) - Modern serialization

#### Additional OOP Features

- [x] **Anonymous classes** (PHP 7.0) - `new class { ... }`
- [x] **Property hooks** (PHP 8.4) - `get`/`set` accessors on properties
- [x] **Static properties** (PHP 5.0) - `static $property` with visibility modifiers
- [x] **Late static binding** (PHP 5.3) - `static::`, `self::`, `parent::`
- [x] **Readonly static properties** (PHP 8.3) - Immutable class-level properties
- [ ] **Asymmetric visibility** (PHP 8.4) - `public private(set)` property visibility
- [ ] **Static property visibility** (PHP 8.5) - Asymmetric visibility for static properties
- [ ] **Object comparison** - `==` vs `===` for objects
- [ ] **Covariance & Contravariance** (PHP 7.4) - LSP-compatible type widening/narrowing
- [ ] **Constants in traits** (PHP 8.2)
- [ ] **#[\Override] attribute** (PHP 8.3) - Marking overridden methods

#### Control Flow Additions

- [x] **Alternative syntax** - `if:`, `endif;`, `foreach:`, `endforeach;`, etc. ✅
- [ ] **goto statement** - Jump to label
- [ ] **declare directive** - `declare(strict_types=1);`

#### Function Features

- [x] **Arrow functions** (PHP 7.4) - `fn($x) => $x * 2` ✅
- [x] **Variadic functions** - `function f(...$args)`
- [x] **Argument unpacking** - `f(...$array)`
- [x] **First-class callables** (PHP 8.1) - `$fn = strlen(...)` ✅
- [ ] **Closures in constants** (PHP 8.5) - Static closures in constant expressions

### Phase 8: PHP 8.5 New Features 📋

Features from the latest PHP release that VHP should support to be a true PHP 8.5 superset.

- [ ] **URI Extension** (PHP 8.5) - Built-in URI/URL parsing and manipulation via `Uri\Rfc3986\Uri` class
- [ ] **Clone with syntax** (PHP 8.5) - `clone($obj, ['prop' => 'value'])` syntax (VHP has basic clone)
- [ ] **#[\NoDiscard] attribute** (PHP 8.5) - Warn when return values are ignored
- [ ] **Closures in constant expressions** (PHP 8.5) - Static closures in attributes and defaults
- [ ] **First-class callables in constants** (PHP 8.5) - `strlen(...)` in constant expressions
- [x] **array_first() / array_last()** (PHP 8.5) - Get first/last element of array
- [ ] **#[\DelayedTargetValidation]** (PHP 8.5) - Delay attribute target validation
- [ ] **Final property promotion** (PHP 8.5) - `final` in constructor property promotion
- [ ] **Attributes on constants** (PHP 8.5) - Apply attributes to constants
- [ ] **Error backtraces** (PHP 8.5) - Fatal errors include backtraces

### Phase 9: Standard Library Expansion 📋

Expanding built-in functions to match PHP's standard library.

#### String Functions (missing)

- [ ] `sprintf` format specifiers: `%d`, `%s`, `%f`, `%x`, `%b`, padding, precision
- [ ] `sscanf`, `fprintf`, `vprintf`, `vsprintf`
- [ ] `preg_match`, `preg_match_all`, `preg_replace`, `preg_split` (PCRE)
- [ ] `str_split`, `chunk_split`, `wordwrap`
- [ ] `str_shuffle`, `str_word_count`
- [ ] `number_format`, `money_format`
- [ ] `html_entity_decode`, `htmlentities`, `htmlspecialchars`, `htmlspecialchars_decode`
- [ ] `strip_tags`, `addslashes`, `stripslashes`
- [ ] `quoted_printable_encode`, `quoted_printable_decode`
- [ ] `convert_uuencode`, `convert_uudecode`
- [ ] `ctype_*` functions (`ctype_alpha`, `ctype_digit`, etc.)

#### Array Functions (missing)

- [ ] `array_map`, `array_filter`, `array_reduce`, `array_walk`
- [ ] `array_slice`, `array_splice`, `array_chunk`
- [ ] `array_combine`, `array_fill`, `array_fill_keys`
- [ ] `array_flip`, `array_unique`, `array_count_values`
- [ ] `array_diff`, `array_diff_key`, `array_diff_assoc`
- [ ] `array_intersect`, `array_intersect_key`, `array_intersect_assoc`
- [ ] `array_column`, `array_multisort`
- [ ] `sort`, `rsort`, `asort`, `arsort`, `ksort`, `krsort`, `usort`, `uasort`, `uksort`
- [ ] `shuffle`, `array_rand`
- [ ] `array_pad`, `array_product`, `array_sum`
- [ ] `list()` assignment, `extract()`, `compact()`

#### Math Functions (missing)

- [ ] `log`, `log10`, `log1p`, `exp`, `expm1`
- [ ] `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`
- [ ] `sinh`, `cosh`, `tanh`, `asinh`, `acosh`, `atanh`
- [ ] `fmod`, `intdiv`, `fdiv`
- [ ] `pi`, `M_PI`, `M_E` constants
- [ ] `hypot`, `deg2rad`, `rad2deg`
- [ ] `base_convert`, `bindec`, `octdec`, `hexdec`, `decbin`, `decoct`, `dechex`
- [ ] `is_nan`, `is_finite`, `is_infinite`

#### Date/Time Functions

- [ ] `time`, `mktime`, `strtotime`
- [ ] `date`, `gmdate`, `strftime`
- [ ] `DateTime` class and related classes
- [ ] `DateTimeImmutable`, `DateInterval`, `DatePeriod`

#### JSON Functions

- [ ] `json_encode`, `json_decode`
- [ ] `json_last_error`, `json_last_error_msg`
- [ ] `JSON_*` constants

#### File System Functions

- [ ] `file_get_contents`, `file_put_contents`
- [ ] `fopen`, `fclose`, `fread`, `fwrite`, `fgets`
- [ ] `file_exists`, `is_file`, `is_dir`, `is_readable`, `is_writable`
- [ ] `mkdir`, `rmdir`, `unlink`, `rename`, `copy`
- [ ] `glob`, `scandir`, `readdir`
- [ ] `realpath`, `dirname`, `basename`, `pathinfo`

#### Miscellaneous

- [ ] `class_exists`, `interface_exists`, `trait_exists`, `function_exists`
- [ ] `get_class`, `get_parent_class`, `is_a`, `is_subclass_of`
- [ ] `method_exists`, `property_exists`
- [ ] `call_user_func`, `call_user_func_array`
- [ ] `constant`, `define`, `defined`

## Contributing to the Roadmap

Have ideas for VHP? Feel free to:
- Open an issue to discuss new features
- Submit a pull request with an implementation
- Join the discussion on existing roadmap items
