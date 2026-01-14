# Extended Array Functions

## Status: Planned

## Overview

Implement remaining essential array functions not yet supported in VHP. These functions are critical for data manipulation in PHP applications.

## Current Status

VHP currently supports 21 array functions. Many commonly used functions are still missing.

## Background

Array manipulation is fundamental to PHP. Missing functions block real-world PHP applications that rely on these standard library features.

## Already Implemented (21 functions)

`count`, `sizeof`, `array_push`, `array_pop`, `array_shift`, `array_unshift`, `array_keys`, `array_values`, `in_array`, `array_search`, `array_reverse`, `array_merge`, `array_key_exists`, `range`, `array_first`, `array_last`, `array_map`, `array_filter`, `array_reduce`, `array_sum`, `array_unique`

## Functions to Implement

### High Priority (Most Common)

1. **array_slice**
   ```php
   array_slice($array, $offset, $length = null, $preserve_keys = false): array
   ```
   - Extract a slice of an array
   - Support negative offsets
   - Support preserve_keys flag

2. **array_splice**
   ```php
   array_splice(&$array, $offset, $length = null, $replacement = []): array
   ```
   - Remove a portion of array and replace it
   - Modify array in-place
   - Return removed elements

3. **array_combine**
   ```php
   array_combine($keys, $values): array|false
   ```
   - Create array using one array for keys and another for values

4. **array_fill**
   ```php
   array_fill($start_index, $count, $value): array
   ```
   - Fill array with values

5. **array_fill_keys**
   ```php
   array_fill_keys($keys, $value): array
   ```
   - Fill array with values, using keys array

6. **array_flip**
   ```php
   array_flip($array): array
   ```
   - Exchange keys and values

7. **array_intersect**
   ```php
   array_intersect($array1, $array2, ...): array
   ```
   - Computes intersection of arrays (compare values)

8. **array_intersect_key**
   ```php
   array_intersect_key($array1, $array2, ...): array
   ```
   - Computes intersection of arrays (compare keys)

9. **array_intersect_assoc**
   ```php
   array_intersect_assoc($array1, $array2, ...): array
   ```
   - Computes intersection of arrays (compare keys and values)

10. **array_diff**
    ```php
    array_diff($array1, $array2, ...): array
    ```
    - Computes difference of arrays (compare values)

11. **array_diff_key**
    ```php
    array_diff_key($array1, $array2, ...): array
    ```
    - Computes difference of arrays (compare keys)

12. **array_diff_assoc**
    ```php
    array_diff_assoc($array1, $array2, ...): array
    ```
    - Computes difference of arrays (compare keys and values)

13. **array_chunk**
    ```php
    array_chunk($array, $length, $preserve_keys = false): array
    ```
    - Split array into chunks

14. **array_column**
    ```php
    array_column($array, $column_key, $index_key = null): array
    ```
    - Return values from single column in multi-dimensional array

15. **array_product**
    ```php
    array_product($array): int|float
    ```
    - Calculate product of values

16. **array_count_values**
    ```php
    array_count_values($array): array
    ```
    - Counts all values in an array

### Medium Priority (Common)

17. **sort**
    ```php
    sort(&$array): bool
    ```
    - Sort array in ascending order

18. **rsort**
    ```php
    rsort(&$array): bool
    ```
    - Sort array in descending order

19. **asort**
    ```php
    asort(&$array): bool
    ```
    - Sort array maintaining index association

20. **arsort**
    ```php
    arsort(&$array): bool
    ```
    - Sort array in descending order, maintaining index association

21. **ksort**
    ```php
    ksort(&$array): bool
    ```
    - Sort array by key

22. **krsort**
    ```php
    krsort(&$array): bool
    ```
    - Sort array by key in descending order

23. **usort**
    ```php
    usort(&$array, $callback): bool
    ```
    - Sort array by values using a user-defined comparison function

24. **uasort**
    ```php
    uasort(&$array, $callback): bool
    ```
    - Sort array with a user-defined comparison function and maintain index association

25. **uksort**
    ```php
    uksort(&$array, $callback): bool
    ```
    - Sort array by keys using a user-defined comparison function

26. **shuffle**
    ```php
    shuffle(&$array): bool
    ```
    - Shuffle array randomly

27. **array_rand**
    ```php
    array_rand($array, $num = 1): int|string|array
    ```
    - Pick random keys from array

28. **array_pad**
    ```php
    array_pad($array, $length, $value): array
    ```
    - Pad array to specified length with value

29. **array_walk**
    ```php
    array_walk(&$array, $callback, $arg = null): bool
    ```
    - Apply user function to each member of array

### Low Priority (Specialized)

30. **list()** (language construct)
    ```php
    list($var1, $var2) = $array;
    ```
    - Assign variables as if they were an array

31. **extract**
    ```php
    extract($array, $flags = EXTR_OVERWRITE, $prefix = ""): int
    ```
    - Import variables into current symbol table from array

32. **compact**
    ```php
    compact($var1, $var2, ...): array
    ```
    - Create array containing variables and their values

33. **array_multisort**
    ```php
    array_multisort(&$array1, ...): bool
    ```
    - Sort multiple or multi-dimensional arrays

34. **array_replace**
    ```php
    array_replace($array, ...$replacements): array
    ```
    - Replace elements from passed arrays into first array

35. **array_replace_recursive**
    ```php
    array_replace_recursive($array, ...$replacements): array
    ```
    - Replace elements recursively from passed arrays into first array

36. **array_merge_recursive**
    ```php
    array_merge_recursive(...$arrays): array
    ```
    - Merge one or more arrays recursively

37. **array_intersect_uassoc**
    ```php
    array_intersect_uassoc($array1, $array2, $key_compare_func): array
    ```
    - Computes intersection with additional index check and compare by key callback

38. **array_diff_uassoc**
    ```php
    array_diff_uassoc($array1, $array2, $key_compare_func): array
    ```
    - Computes difference with additional index check and compare by key callback

39. **array_uintersect**
    ```php
    array_uintersect($array1, $array2, $value_compare_func): array
    ```
    - Computes intersection using callback for data comparison

40. **array_udiff**
    ```php
    array_udiff($array1, $array2, $value_compare_func): array
    ```
    - Computes difference using callback for data comparison

41. **natcasesort**
    ```php
    natcasesort(&$array): bool
    ```
    - Sort array using case insensitive "natural order" algorithm

42. **natsort**
    ```php
    natsort(&$array): bool
    ```
    - Sort array using a "natural order" algorithm

## Implementation Plan

### Phase 1: Slice and Splice Operations

**File:** `runtime/builtins/array_slice.rs` (new)

**Tasks:**
- [ ] Implement array_slice
- [ ] Implement array_splice
- [ ] Handle negative offsets
- [ ] Handle preserve_keys flag
- [ ] Add tests

### Phase 2: Array Creation and Filling

**File:** `runtime/builtins/array_fill.rs` (new)

**Tasks:**
- [ ] Implement array_combine
- [ ] Implement array_fill
- [ ] Implement array_fill_keys
- [ ] Add tests

### Phase 3: Array Transformation

**File:** `runtime/builtins/array_transform.rs` (new)

**Tasks:**
- [ ] Implement array_flip
- [ ] Implement array_chunk
- [ ] Implement array_column
- [ ] Implement array_count_values
- [ ] Add tests

### Phase 4: Array Intersection

**File:** `runtime/builtins/array_intersect.rs` (new)

**Tasks:**
- [ ] Implement array_intersect
- [ ] Implement array_intersect_key
- [ ] Implement array_intersect_assoc
- [ ] Add tests

### Phase 5: Array Difference

**File:** `runtime/builtins/array_diff.rs` (new)

**Tasks:**
- [ ] Implement array_diff
- [ ] Implement array_diff_key
- [ ] Implement array_diff_assoc
- [ ] Add tests

### Phase 6: Array Aggregation

**File:** `runtime/builtins/array_agg.rs` (new)

**Tasks:**
- [ ] Implement array_product
- [ ] Add to existing file if array_sum is already there
- [ ] Add tests

### Phase 7: Sorting Functions

**File:** `runtime/builtins/array_sort.rs` (new)

**Tasks:**
- [ ] Implement sort
- [ ] Implement rsort
- [ ] Implement asort
- [ ] Implement arsort
- [ ] Implement ksort
- [ ] Implement krsort
- [ ] Implement usort
- [ ] Implement uasort
- [ ] Implement uksort
- [ ] Implement shuffle
- [ ] Implement natcasesort
- [ ] Implement natsort
- [ ] Add tests for all sorting functions

### Phase 8: Random and Utility

**File:** `runtime/builtins/array_util.rs` (new)

**Tasks:**
- [ ] Implement array_rand
- [ ] Implement array_pad
- [ ] Implement array_walk
- [ ] Add tests

### Phase 9: Language Constructs

**File:** `parser/stmt.rs` (existing) and `vm/compiler/stmt.rs` (existing)

**Tasks:**
- [ ] Implement list() parsing
- [ ] Implement list() compilation
- [ ] Implement list() execution
- [ ] Add tests

### Phase 10: Variable Manipulation

**File:** `runtime/builtins/array_vars.rs` (new)

**Tasks:**
- [ ] Implement extract
- [ ] Implement compact
- [ ] Add tests

### Phase 11: Advanced Operations

**File:** `runtime/builtins/array_advanced.rs` (new)

**Tasks:**
- [ ] Implement array_multisort
- [ ] Implement array_replace
- [ ] Implement array_replace_recursive
- [ ] Implement array_merge_recursive
- [ ] Implement callback-based intersection/diff functions
- [ ] Add tests

## Implementation Details

### Sorting Algorithms

For sorting functions, use Rust's stable sort:
```rust
use std::collections::HashMap;

// Basic sort
array.sort_by(|a, b| a.cmp(b));

// Custom comparison (usort)
array.sort_by(|a, b| {
    let result = call_user_func(callback, [a.clone(), b.clone()])?;
    // Compare result: -1, 0, 1
});
```

### Comparison Semantics

- Values: Use PHP's loose comparison (`==`) unless specified
- Keys: Use strict comparison (`===`)
- Maintain PHP's type coercion rules

### Callback Handling

Functions with user callbacks need to:
1. Extract callback from Value
2. Call the function with appropriate arguments
3. Handle return values (-1, 0, 1 for comparisons)

```rust
fn call_compare_callback(callback: &Value, a: &Value, b: &Value) -> Result<Ordering, String> {
    let result = call_user_func(callback, [a.clone(), b.clone()])?;

    match result {
        Value::Int(n) => {
            match n.cmp(&0) {
                Ordering::Less => Ok(Ordering::Less),
                Ordering::Equal => Ok(Ordering::Equal),
                Ordering::Greater => Ok(Ordering::Greater),
            }
        },
        Value::Float(f) => {
            if f < 0.0 { Ok(Ordering::Less) }
            else if f > 0.0 { Ok(Ordering::Greater) }
            else { Ok(Ordering::Equal) }
        },
        _ => Err("Comparison callback must return number".to_string()),
    }
}
```

## Dependencies

- Existing array infrastructure
- Existing function call mechanism
- Existing value comparison logic

## Testing Strategy

1. **Unit Tests**: Each array function
2. **Integration Tests**: Combined use of multiple functions
3. **Edge Cases**: Empty arrays, non-arrays, mixed types
4. **Compatibility Tests**: Match PHP 8.x behavior

**Test file structure:**
```
tests/array/
├── slice.vhpt
├── splice.vhpt
├── fill.vhpt
├── intersect.vhpt
├── diff.vhpt
├── sort.vhpt
└── ...
```

## Success Criteria

- All high and medium priority functions implemented
- Functions match PHP 8.x behavior for common use cases
- Sorting is stable and correct
- Callbacks work correctly
- Edge cases handled properly
- Performance is acceptable

## Performance Considerations

- Use efficient algorithms (O(n log n) for sorting)
- Avoid unnecessary copying
- Use iterators where possible
- Cache frequently accessed values

## Open Questions

- Should we support array_multisort with multiple sort flags?
- How to handle extract() with conflicting variable names?
- Performance vs compatibility for sorting algorithms?

## References

- PHP array functions documentation: https://www.php.net/manual/en/book.array.php

## Related Plans

- Collections/Iterator interface (future)
- Sorting algorithms optimization (future)
