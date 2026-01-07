# Performance Benchmarks

This directory contains performance benchmarks comparing VHP and PHP execution times.

## Quick Start

```bash
# Build VHP in release mode first
make release

# Run all benchmarks
make bench

# Or run directly
python3 run_benchmarks.py
# Or use the shell wrapper
./run_benchmarks.sh
```

## Requirements

- Python 3.x (for running the benchmark script)
- PHP (any recent version)
- VHP release build

## Benchmark Suite

The benchmark suite includes the following tests:

### 1. Fibonacci (`fibonacci.php`)
Tests recursive function calls and stack performance with the classic Fibonacci algorithm (n=30).

**What it measures:**
- Recursive function call overhead
- Stack frame allocation/deallocation
- Return value handling

### 2. Array Operations (`array_operations.php`)
Tests array manipulation and built-in array functions.

**What it measures:**
- Array creation with `range()`
- Iteration with `foreach`
- Array manipulation: `array_push`, `array_pop`, `array_unshift`, `array_shift`
- Array searching: `in_array`
- Array merging: `array_merge`

### 3. String Operations (`string_operations.php`)
Tests string manipulation and built-in string functions.

**What it measures:**
- String concatenation (`.=` operator)
- Case conversion: `strtoupper`, `strtolower`
- String searching: `strpos`
- String replacement: `str_replace`
- String splitting/joining: `explode`, `implode`
- String length: `strlen`

### 4. Loops (`loops.php`)
Tests various loop constructs and iteration performance.

**What it measures:**
- Nested `for` loops
- `while` loops
- `foreach` with arrays
- Loop variable updates

### 5. Function Calls (`function_calls.php`)
Tests function call overhead and parameter passing.

**What it measures:**
- Function call overhead
- Parameter passing
- Return value handling
- Nested function calls

### 6. Class Instantiation (`class_instantiation.php`)
Tests object-oriented programming performance.

**What it measures:**
- Object instantiation (`new` operator)
- Constructor calls
- Method calls
- Property access
- Object storage in arrays

## Customization

### Environment Variables

- `VHP_BIN`: Path to VHP binary (default: `./target/release/vhp`)
- `PHP_BIN`: Path to PHP binary (default: `php`)
- `BENCH_ITERATIONS`: Number of iterations per benchmark (default: `5`)

### Examples

```bash
# Use a specific PHP version
PHP_BIN=/usr/local/bin/php8.3 python3 run_benchmarks.py

# Run more iterations for more accurate results
BENCH_ITERATIONS=10 python3 run_benchmarks.py

# Use debug build of VHP (slower, for development)
VHP_BIN=./target/debug/vhp python3 run_benchmarks.py
```

## Adding New Benchmarks

To add a new benchmark:

1. Create a new `.php` file in the `bench/` directory
2. Implement your benchmark code
3. Ensure the script produces predictable output (for verification)
4. The runner script will automatically discover and run it

Example benchmark structure:

```php
<?php
// Your benchmark description

// Your benchmark code here
$result = compute_something();

// Output results for verification
echo $result . "\n";
```

**Note**: The benchmark runner measures execution time externally, so you don't need to use `microtime()` or similar timing functions.

## Interpreting Results

The benchmark runner will display:

- **Individual Results**: Average execution time for each benchmark
- **Speedup Factor**: How many times faster/slower VHP is compared to PHP
- **Summary Table**: Overview of all benchmarks with averages

### Color Coding

- 🟢 **Green**: VHP is faster than PHP
- 🔴 **Red**: VHP is slower than PHP

## Notes

### Performance Expectations

VHP is a tree-walking interpreter written in Rust, while PHP uses a VM with JIT compilation (PHP 8+). Expected performance characteristics:

- **VHP Advantages**:
  - Zero-cost abstractions from Rust
  - Memory safety without garbage collection overhead
  - Efficient built-in function implementations

- **PHP Advantages**:
  - Highly optimized VM (Zend Engine)
  - JIT compilation for hot code paths
  - Decades of performance tuning

### Fair Comparison

For a fair comparison:

1. Use PHP 8.x (with opcache enabled if available)
2. Use VHP release build (`--release` flag)
3. Run multiple iterations (5+ recommended)
4. Benchmark on a quiet system (no heavy background processes)

### Benchmark Limitations

These microbenchmarks test specific language features in isolation. Real-world performance depends on:

- I/O operations (file system, network, database)
- Framework overhead
- Application architecture
- Memory usage patterns

## Contributing

If you have ideas for additional benchmarks that would be valuable, please open an issue or submit a PR!

### Benchmark Ideas

- Pattern matching with `match` expressions
- Enum operations
- Attribute reflection
- Exception handling overhead
- Array sorting algorithms
- Trait composition
- Namespace resolution
