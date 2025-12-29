# Plan: Constructor Property Promotion (PHP 8.0)

## Overview

Implement PHP 8.0's Constructor Property Promotion, which allows declaring and initializing class properties directly in the constructor signature.

**Before:**
```php
class User {
    public $name;
    private $age;

    public function __construct($name, $age) {
        $this->name = $name;
        $this->age = $age;
    }
}
```

**After:**
```php
class User {
    public function __construct(
        public $name,
        private $age
    ) {}
}
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/ast/stmt.rs` | Add `visibility` field to `FunctionParam` |
| `src/parser/stmt.rs` | Parse visibility modifiers on constructor params |
| `src/interpreter/mod.rs` | Extract promoted properties during class registration |
| `tests/classes/*.vhpt` | Add test files |
| `AGENTS.md`, `README.md`, `docs/features.md`, `docs/roadmap.md` | Update documentation |

## Implementation Steps

### Step 1: Extend AST (`src/ast/stmt.rs`)

Add optional visibility to `FunctionParam` struct (line 143-150):

```rust
pub struct FunctionParam {
    pub name: String,
    pub default: Option<Expr>,
    #[allow(dead_code)]
    pub by_ref: bool,
    /// Visibility for constructor property promotion (PHP 8.0)
    pub visibility: Option<Visibility>,
}
```

### Step 2: Update Parser (`src/parser/stmt.rs`)

**2a.** Modify `parse_method()` (lines 601-682) to:
1. Detect when parsing `__construct` (case-insensitive)
2. Before parsing each parameter, check for visibility tokens (`Public`, `Private`, `Protected`)
3. Store visibility in `FunctionParam`
4. Error if visibility is used on non-constructor methods

**2b.** Update all `FunctionParam` instantiations to include `visibility: None`:
- `parse_function()` (around line 501)
- `parse_interface_method()` (around line 724)

### Step 3: Update Interpreter (`src/interpreter/mod.rs`)

Modify class registration in `Stmt::Class` handling (lines 1709-1826):

1. After building `methods_map` from methods (lines 1783-1791)
2. Find the `__construct` method in the list
3. For each parameter with `visibility.is_some()`:
   - Create a `Property` with that visibility and `default: None`
   - Add to `all_properties` (this makes it auto-initialized at object creation)
   - Prepend an assignment `Stmt::Expression(Expr::Assign { target: PropertyAccess($this, param_name), value: Variable(param_name) })` to the constructor body

**Note:** By prepending assignments to the constructor body, promoted properties are assigned before user code runs.

### Step 4: Add Tests (`tests/classes/`)

Create test files:
1. `constructor_promotion_basic.vhpt` - Basic public/private promoted params
2. `constructor_promotion_mixed.vhpt` - Mix of promoted and regular params
3. `constructor_promotion_defaults.vhpt` - Promoted params with default values
4. `constructor_promotion_with_body.vhpt` - Constructor with both promotion and body code
5. `constructor_promotion_all_visibilities.vhpt` - public, protected, private
6. `constructor_promotion_error.vhpt` - Error when visibility on non-constructor

### Step 5: Update Documentation

- Mark "Constructor Property Promotion" as complete in roadmap
- Add examples to features documentation

## Key Considerations

1. **Assignment Order**: Promoted property assignments happen before constructor body runs
2. **Inheritance**: Each class handles its own promoted properties independently
3. **Default Values**: When promoted param has default, property gets that value if arg not provided
4. **Visibility Enforcement**: Currently not enforced (same as existing properties)

## Test Cases

```php
// Basic promotion
class A { public function __construct(public $x) {} }
$a = new A(1);
echo $a->x; // 1

// Mixed params
class B { public function __construct(public $x, $y) { echo $y; } }
$b = new B(1, 2); // echoes 2
echo $b->x; // 1

// With defaults
class C { public function __construct(public $x = 10) {} }
$c = new C();
echo $c->x; // 10

// Error case
class D { public function foo(public $x) {} } // ERROR
```
