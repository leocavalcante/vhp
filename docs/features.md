---
layout: default
title: Features
nav_order: 2
---

# Features

VHP supports a comprehensive subset of PHP syntax and semantics.

## Basic Syntax

- PHP tags: `<?php`, `?>`, `<?=` (short echo)
- `echo` statement with comma-separated expressions
- String literals (single/double quoted) with escape sequences
- Integer, float, boolean, and null literals
- Comments: `//`, `/* */`, `#`
- HTML passthrough (mixed PHP/HTML)

## Variables & Assignment

```php
<?php
$name = "VHP";
$count = 42;
$count += 8;  // Compound assignment
echo "$name: $count";  // Output: VHP: 50
```

### Supported Assignment Operators

- Basic: `=`
- Compound: `+=`, `-=`, `*=`, `/=`, `%=`, `.=`

## Operators

### Arithmetic

```php
<?php
echo 2 + 3 * 4;      // 14 (correct precedence!)
echo 2 ** 10;        // 1024 (power operator)
```

### Comparison

```php
<?php
echo 1 == "1" ? "loose" : "strict";   // loose
echo 1 === "1" ? "loose" : "strict";  // strict
```

Supported: `==`, `===`, `!=`, `!==`, `<`, `>`, `<=`, `>=`, `<=>` (spaceship)

### Logical Operators

```php
<?php
$a = true && false;   // false
$b = true || false;   // true
$c = !true;           // false
$d = true and false;  // false (lower precedence)
$e = true or false;   // true (lower precedence)
$f = true xor true;   // false
```

Supported: `&&`, `||`, `!`, `and`, `or`, `xor`

### Null Coalescing

```php
<?php
$user = $name ?? "Anonymous";
```

### Ternary

```php
<?php
echo $age >= 18 ? "adult" : "minor";
```

### Increment/Decrement

```php
<?php
$i = 0;
echo ++$i;  // 1 (pre-increment)
echo $i++;  // 1 (post-increment, $i is now 2)
```

## Control Flow

### If-Elseif-Else

```php
<?php
$score = 85;
if ($score >= 90) {
    echo "A";
} elseif ($score >= 80) {
    echo "B";
} else {
    echo "C";
}
```

### While Loop

```php
<?php
$i = 0;
while ($i < 5) {
    echo $i++;
}
```

### For Loop

```php
<?php
for ($i = 0; $i < 5; $i++) {
    echo $i;
}
```

### Do-While Loop

```php
<?php
$i = 0;
do {
    echo $i++;
} while ($i < 3);
```

### Switch Statement

```php
<?php
$day = 1;
switch ($day) {
    case 1:
        echo "Monday";
        break;
    case 2:
        echo "Tuesday";
        break;
    default:
        echo "Other day";
}
```

### Break and Continue

```php
<?php
for ($i = 0; $i < 10; $i++) {
    if ($i == 3) continue;  // Skip 3
    if ($i == 7) break;     // Stop at 7
    echo $i;
}
```

## PHP-Compatible Type Coercion

```php
<?php
// Loose equality with type juggling
echo 0 == "0" ? "yes" : "no";     // yes
echo 0 == "" ? "yes" : "no";      // yes
echo 0 == false ? "yes" : "no";   // yes

// Strict equality (no coercion)
echo 0 === "0" ? "yes" : "no";    // no
echo 0 === false ? "yes" : "no";  // no
```

## Functions

### User-Defined Functions

```php
<?php
function greet($name) {
    return "Hello, " . $name . "!";
}
echo greet("World");  // Hello, World!
```

### Default Parameters

```php
<?php
function power($base, $exp = 2) {
    return $base ** $exp;
}
echo power(3);     // 9
echo power(2, 10); // 1024
```

### Recursive Functions

```php
<?php
function factorial($n) {
    if ($n <= 1) return 1;
    return $n * factorial($n - 1);
}
echo factorial(5); // 120
```

### Built-in Functions (50+)

```php
<?php
echo strlen("Hello");              // 5
echo strtoupper("hello");          // HELLO
echo substr("Hello World", 0, 5);  // Hello
echo str_repeat("ab", 3);          // ababab
echo abs(-42);                     // 42
echo round(3.7);                   // 4
echo max(1, 5, 3);                 // 5
echo sprintf("Name: %s, Age: %d", "John", 25);
```

#### String Functions (24)

`strlen`, `substr`, `strtoupper`, `strtolower`, `trim`, `ltrim`, `rtrim`, `str_repeat`, `str_replace`, `strpos`, `strrev`, `ucfirst`, `lcfirst`, `ucwords`, `str_starts_with`, `str_ends_with`, `str_contains`, `str_pad`, `explode`, `implode`/`join`, `sprintf`, `chr`, `ord`

#### Math Functions (9)

`abs`, `ceil`, `floor`, `round`, `max`, `min`, `pow`, `sqrt`, `rand`/`mt_rand`

#### Type Functions (13)

`intval`, `floatval`/`doubleval`, `strval`, `boolval`, `gettype`, `is_null`, `is_bool`, `is_int`/`is_integer`/`is_long`, `is_float`/`is_double`/`is_real`, `is_string`, `is_numeric`, `isset`, `empty`

#### Output Functions (4)

`print`, `var_dump`, `print_r`, `printf`
