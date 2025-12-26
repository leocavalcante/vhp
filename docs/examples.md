---
layout: default
title: Examples
nav_order: 5
---

# Examples

## Hello World

```php
<?php
echo "Hello, VHP!\n";
```

## Variables and Math

```php
<?php
$a = 10;
$b = 5;
$c = ($a + $b) * 2 - $a / $b;
echo $c;  // Output: 28
```

## Mixed HTML/PHP

```php
<!DOCTYPE html>
<html>
<body>
    <h1><?= "Welcome to VHP!" ?></h1>
    <?php
    $items = 3;
    echo "<p>You have $items items.</p>";
    ?>
</body>
</html>
```

## Null Safety

```php
<?php
$config = null;
$timeout = $config ?? 30;
echo "Timeout: $timeout";  // Output: Timeout: 30
```

## FizzBuzz

```php
<?php
for ($i = 1; $i <= 100; $i++) {
    if ($i % 15 == 0) {
        echo "FizzBuzz\n";
    } elseif ($i % 3 == 0) {
        echo "Fizz\n";
    } elseif ($i % 5 == 0) {
        echo "Buzz\n";
    } else {
        echo $i . "\n";
    }
}
```

## Factorial (Recursive)

```php
<?php
function factorial($n) {
    if ($n <= 1) return 1;
    return $n * factorial($n - 1);
}

echo "5! = " . factorial(5) . "\n";  // 120
echo "10! = " . factorial(10) . "\n"; // 3628800
```

## Fibonacci Sequence

```php
<?php
function fibonacci($n) {
    if ($n <= 1) return $n;
    return fibonacci($n - 1) + fibonacci($n - 2);
}

echo "Fibonacci sequence: ";
for ($i = 0; $i < 10; $i++) {
    echo fibonacci($i);
    if ($i < 9) echo ", ";
}
// Output: 0, 1, 1, 2, 3, 5, 8, 13, 21, 34
```

## String Manipulation

```php
<?php
$text = "  Hello, World!  ";

echo strlen($text) . "\n";           // 17
echo trim($text) . "\n";             // "Hello, World!"
echo strtoupper($text) . "\n";       // "  HELLO, WORLD!  "
echo str_replace("World", "VHP", $text) . "\n";  // "  Hello, VHP!  "
echo strrev(trim($text)) . "\n";     // "!dlroW ,olleH"
```

## Temperature Converter

```php
<?php
function celsiusToFahrenheit($c) {
    return $c * 9 / 5 + 32;
}

function fahrenheitToCelsius($f) {
    return ($f - 32) * 5 / 9;
}

$celsius = 100;
$fahrenheit = 32;

echo $celsius . "°C = " . celsiusToFahrenheit($celsius) . "°F\n";
echo $fahrenheit . "°F = " . fahrenheitToCelsius($fahrenheit) . "°C\n";
```

## Prime Number Checker

```php
<?php
function isPrime($n) {
    if ($n < 2) return false;
    if ($n == 2) return true;
    if ($n % 2 == 0) return false;

    $i = 3;
    while ($i * $i <= $n) {
        if ($n % $i == 0) return false;
        $i += 2;
    }
    return true;
}

echo "Primes up to 50: ";
for ($i = 2; $i <= 50; $i++) {
    if (isPrime($i)) {
        echo $i . " ";
    }
}
// Output: 2 3 5 7 11 13 17 19 23 29 31 37 41 43 47
```
