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

## Arrays

### Array Literals

```php
<?php
$numbers = [1, 2, 3, 4, 5];
$mixed = ["hello", 42, true, null];
$empty = [];
```

### Associative Arrays

```php
<?php
$person = [
    "name" => "John",
    "age" => 30,
    "city" => "NYC"
];
echo $person["name"];  // John
```

### Array Access and Modification

```php
<?php
$arr = [10, 20, 30];
echo $arr[0];          // 10
$arr[1] = 25;          // Modify
$arr[] = 40;           // Append
```

### Foreach Loop

```php
<?php
$colors = ["red", "green", "blue"];

// Value only
foreach ($colors as $color) {
    echo $color . "\n";
}

// Key and value
$prices = ["apple" => 1.50, "banana" => 0.75];
foreach ($prices as $fruit => $price) {
    echo "$fruit: \$$price\n";
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

### Built-in Functions (65+)

```php
<?php
echo strlen("Hello");              // 5
echo strtoupper("hello");          // HELLO
echo substr("Hello World", 0, 5);  // Hello
echo str_repeat("ab", 3);          // ababab
echo abs(-42);                     // 42
echo round(3.7);                   // 4
echo max(1, 5, 3);                 // 5
echo count([1, 2, 3]);             // 3
echo sprintf("Name: %s, Age: %d", "John", 25);
```

#### String Functions (24)

`strlen`, `substr`, `strtoupper`, `strtolower`, `trim`, `ltrim`, `rtrim`, `str_repeat`, `str_replace`, `strpos`, `strrev`, `ucfirst`, `lcfirst`, `ucwords`, `str_starts_with`, `str_ends_with`, `str_contains`, `str_pad`, `explode`, `implode`/`join`, `sprintf`, `chr`, `ord`

#### Math Functions (9)

`abs`, `ceil`, `floor`, `round`, `max`, `min`, `pow`, `sqrt`, `rand`/`mt_rand`

#### Array Functions (13)

`count`/`sizeof`, `array_push`, `array_pop`, `array_shift`, `array_unshift`, `array_keys`, `array_values`, `in_array`, `array_search`, `array_reverse`, `array_merge`, `array_key_exists`, `range`

#### Type Functions (14)

`intval`, `floatval`/`doubleval`, `strval`, `boolval`, `gettype`, `is_null`, `is_bool`, `is_int`/`is_integer`/`is_long`, `is_float`/`is_double`/`is_real`, `is_string`, `is_array`, `is_numeric`, `isset`, `empty`

#### Output Functions (4)

`print`, `var_dump`, `print_r`, `printf`

## Classes & Objects

### Class Declaration

```php
<?php
class Person {
    public $name;
    public $age = 0;

    function __construct($name, $age) {
        $this->name = $name;
        $this->age = $age;
    }

    function greet() {
        return "Hello, my name is " . $this->name;
    }
}
```

### Creating Objects

```php
<?php
$person = new Person("Alice", 30);
echo $person->name;     // Alice
echo $person->greet();  // Hello, my name is Alice
```

### Properties

```php
<?php
class Box {
    public $value = 10;    // With default value
    public $label;         // Without default (null)
}

$box = new Box();
echo $box->value;  // 10
$box->value = 20;  // Modify property
```

### Visibility Modifiers

VHP supports visibility modifiers for properties and methods:

- `public` - Accessible from anywhere (default)
- `protected` - Accessible from class and subclasses
- `private` - Accessible only within the class

### Constructors

```php
<?php
class Rectangle {
    public $width;
    public $height;

    function __construct($w, $h) {
        $this->width = $w;
        $this->height = $h;
    }

    function area() {
        return $this->width * $this->height;
    }
}

$rect = new Rectangle(10, 5);
echo $rect->area();  // 50
```

### Static Method Calls

```php
<?php
class Math {
    function square($n) {
        return $n * $n;
    }
}

echo Math::square(5);  // 25
```

### Multiple Objects

```php
<?php
class Counter {
    public $count = 0;

    function increment() {
        $this->count = $this->count + 1;
    }
}

$a = new Counter();
$b = new Counter();
$a->increment();
$a->increment();
$b->increment();
echo $a->count . ", " . $b->count;  // 2, 1
```

## Match Expressions (PHP 8.0)

Match expressions are a more powerful alternative to switch statements. They return a value and use strict comparison.

### Basic Match

```php
<?php
$food = "apple";
$result = match($food) {
    "apple" => "fruit",
    "carrot" => "vegetable",
    "chicken" => "meat",
    default => "unknown",
};
echo $result;  // fruit
```

### Multiple Conditions

```php
<?php
$num = 2;
$result = match($num) {
    1, 2, 3 => "low",
    4, 5, 6 => "medium",
    default => "high",
};
echo $result;  // low
```

### Strict Comparison

Match uses strict (`===`) comparison, unlike switch which uses loose (`==`):

```php
<?php
$val = "1";
echo match($val) {
    1 => "integer",
    "1" => "string",
};  // string
```

### Match with Expressions

```php
<?php
$grade = 85;
echo match(true) {
    $grade >= 90 => "A",
    $grade >= 80 => "B",
    $grade >= 70 => "C",
    default => "F",
};  // B
```

## Named Arguments (PHP 8.0)

Named arguments allow you to pass arguments to functions based on parameter names, making code more readable and allowing you to skip optional parameters.

### Basic Named Arguments

```php
<?php
function greet($name, $greeting = "Hello") {
    return "$greeting, $name!";
}

echo greet(name: "Leo");  // Hello, Leo!
echo greet(greeting: "Hi", name: "World");  // Hi, World!
```

### Skipping Optional Parameters

```php
<?php
function configure($host, $port, $debug = false, $verbose = false) {
    echo "Host: $host, Port: $port";
    if ($debug) echo ", Debug: on";
    if ($verbose) echo ", Verbose: on";
}

// Can skip parameters in the middle
configure(host: "localhost", port: 8080, verbose: true);
// Output: Host: localhost, Port: 8080, Verbose: on
```

### Any Argument Order

```php
<?php
function format($color, $size, $bold = false) {
    return "Color: $color, Size: $size";
}

// Arguments can be in any order when named
echo format(size: 14, bold: true, color: "red");
```

### With Methods and Constructors

```php
<?php
class Point {
    public $x;
    public $y;
    
    public function __construct($x = 0, $y = 0) {
        $this->x = $x;
        $this->y = $y;
    }
}

// Works with constructors
$p = new Point(y: 5, x: 3);

// Works with method calls
class Calculator {
    public function add($a, $b) {
        return $a + $b;
    }
}

$calc = new Calculator();
echo $calc->add(b: 10, a: 5);  // 15
```

### Mixing Positional and Named Arguments

```php
<?php
function build($type, $size, $color = "white") {
    return "$type, $size, $color";
}

// Positional arguments come first, then named
echo build("box", 10, color: "blue");
```

## Interfaces

Interfaces define contracts that classes must follow, specifying method signatures without implementations.

### Basic Interface

```php
<?php
interface Drawable {
    function draw();
    function getColor();
}

class Circle implements Drawable {
    function draw() {
        echo "Drawing a circle";
    }

    function getColor() {
        return "red";
    }
}

$shape = new Circle();
$shape->draw();  // Drawing a circle
```

### Multiple Interfaces

Classes can implement multiple interfaces:

```php
<?php
interface Drawable {
    function draw();
}

interface Resizable {
    function resize($scale);
}

class Rectangle implements Drawable, Resizable {
    function draw() {
        echo "Drawing rectangle";
    }

    function resize($scale) {
        echo "Resizing by $scale";
    }
}
```

### Interface Inheritance

Interfaces can extend other interfaces:

```php
<?php
interface Shape {
    function area();
}

interface Colorable {
    function getColor();
}

interface ColoredShape extends Shape, Colorable {
    function render();
}

class Square implements ColoredShape {
    function area() {
        return 100;
    }

    function getColor() {
        return "blue";
    }

    function render() {
        echo "Rendering colored shape";
    }
}
```

### Interface Constants

Interfaces can define constants:

```php
<?php
interface Config {
    const VERSION = "1.0";
    const MAX_SIZE = 100;
}

echo Config::VERSION;  // 1.0
```

## Traits

Traits enable code reuse in single inheritance languages by allowing methods to be shared across multiple classes.

### Basic Trait

```php
<?php
trait Logger {
    function log($message) {
        echo "[LOG] $message\n";
    }
}

class Application {
    use Logger;

    function run() {
        $this->log("Application started");
    }
}

$app = new Application();
$app->run();  // [LOG] Application started
```

### Multiple Traits

Classes can use multiple traits:

```php
<?php
trait Timestampable {
    function timestamp() {
        return "2024-01-01";
    }
}

trait Identifiable {
    function getId() {
        return 42;
    }
}

class Entity {
    use Timestampable, Identifiable;
}

$entity = new Entity();
echo $entity->getId();  // 42
```

### Trait Properties

Traits can define properties that become part of the using class:

```php
<?php
trait Counter {
    public $count = 0;

    function increment() {
        $this->count = $this->count + 1;
    }
}

class Session {
    use Counter;
}

$session = new Session();
$session->increment();
echo $session->count;  // 1
```

### Overriding Trait Methods

Class methods override trait methods:

```php
<?php
trait DefaultBehavior {
    function greet() {
        return "Hello from trait";
    }
}

class CustomClass {
    use DefaultBehavior;

    function greet() {
        return "Hello from class";
    }
}

$obj = new CustomClass();
echo $obj->greet();  // Hello from class
```

### Traits Using Other Traits

Traits can use other traits:

```php
<?php
trait A {
    function methodA() {
        echo "A";
    }
}

trait B {
    use A;

    function methodB() {
        echo "B";
    }
}

class MyClass {
    use B;
}

$obj = new MyClass();
$obj->methodA();  // A (inherited through trait B)
$obj->methodB();  // B
```

### Conflict Resolution

When multiple traits define the same method, conflicts must be resolved:

```php
<?php
trait A {
    function conflict() {
        echo "From A";
    }
}

trait B {
    function conflict() {
        echo "From B";
    }
}

// This would cause an error without resolution:
// class MyClass {
//     use A, B;  // Error: conflict() defined in both traits
// }

// Use 'insteadof' to resolve conflicts
class MyClass {
    use A, B {
        A::conflict insteadof B;
    }
}

$obj = new MyClass();
$obj->conflict();  // From A
```

### Method Aliasing

Create aliases for trait methods:

```php
<?php
trait Greeter {
    function greet() {
        echo "Hello";
    }
}

class Welcome {
    use Greeter {
        greet as sayHello;
    }
}

$w = new Welcome();
$w->greet();      // Hello
$w->sayHello();   // Hello
```

