---
layout: default
title: Testing
nav_order: 8
---

# Testing

VHP uses `.vhpt` files (inspired by PHP's `.phpt` format) for testing.

## Test File Format

### Basic Test

```
--TEST--
Addition operator
--FILE--
<?php
echo 2 + 3;
--EXPECT--
5
```

### Error Test

```
--TEST--
Division by zero
--FILE--
<?php
echo 10 / 0;
--EXPECT_ERROR--
Division by zero
```

## Test Sections

| Section | Required | Description |
|---------|----------|-------------|
| `--TEST--` | Yes | Test name (displayed in output) |
| `--FILE--` | Yes | PHP code to execute |
| `--EXPECT--` | Yes* | Expected output (exact match) |
| `--EXPECT_ERROR--` | Yes* | Expected error substring |
| `--DESCRIPTION--` | No | Detailed description |
| `--SKIPIF--` | No | Reason to skip (for unimplemented features) |

*One of `--EXPECT--` or `--EXPECT_ERROR--` is required.

## Running Tests

```bash
# Run all tests (compact output)
vhp test

# Verbose output (shows each test name)
vhp test -v

# Run tests from a custom directory
vhp test mydir

# Run a single test file
vhp test path/to/test.vhpt
```

## Example Output

```
$ vhp test -v

Running 120 tests...
  PASS Addition operator
  PASS Basic if statement
  PASS For loop with break
  PASS User-defined function
  PASS Built-in strlen
  PASS String concatenation
  SKIP Array literals (Arrays not yet implemented)
  ...

Tests: 120 total, 119 passed, 0 failed, 0 errors, 1 skipped
```

## Test Organization

Tests are organized by feature in the `tests/` directory:

```
tests/
├── builtins/       # Built-in function tests
├── comments/       # Comment syntax tests
├── control_flow/   # Control flow tests (if, while, for, switch)
├── echo/           # Echo statement tests
├── errors/         # Error handling tests
├── expressions/    # Expression evaluation tests
├── functions/      # User-defined function tests
├── html/           # HTML passthrough tests
├── numbers/        # Numeric literal tests
├── operators/      # Operator tests
├── strings/        # String literal tests
├── tags/           # PHP tag tests
└── variables/      # Variable tests
```

## Writing Tests

### Best Practices

1. **One concept per test** - Test a single feature or behavior
2. **Descriptive names** - The `--TEST--` section should clearly describe what's being tested
3. **Minimal code** - Keep the `--FILE--` section as short as possible
4. **Exact expectations** - The `--EXPECT--` section must match output exactly (including whitespace)

### Example: Testing a New Feature

When adding a new feature, create corresponding tests:

```
--TEST--
New feature: basic usage
--FILE--
<?php
// Test the basic case
--EXPECT--
expected output
```

```
--TEST--
New feature: edge case
--FILE--
<?php
// Test an edge case
--EXPECT--
expected output
```

```
--TEST--
New feature: error handling
--FILE--
<?php
// Test error case
--EXPECT_ERROR--
expected error message
```

## Skipping Tests

Use `--SKIPIF--` for features not yet implemented:

```
--TEST--
Array push function
--SKIPIF--
Arrays not yet implemented
--FILE--
<?php
$arr = [1, 2, 3];
array_push($arr, 4);
echo count($arr);
--EXPECT--
4
```

Skipped tests appear in the summary but don't cause failures.
