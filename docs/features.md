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

### Array First/Last (PHP 8.5)

```php
<?php
$arr = [10, 20, 30, 40];

echo array_first($arr);  // 10
echo array_last($arr);   // 40

// Works with associative arrays
$assoc = ['a' => 1, 'b' => 2, 'c' => 3];
echo array_first($assoc);  // 1
echo array_last($assoc);   // 3

// Empty array returns null
var_dump(array_first([]));  // NULL
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

### Variadic Functions

Variadic functions accept a variable number of arguments using the `...` operator:

```php
<?php
function sum(...$numbers) {
    $total = 0;
    foreach ($numbers as $n) {
        $total += $n;
    }
    return $total;
}
echo sum(1, 2, 3);       // 6
echo sum(1, 2, 3, 4, 5); // 15

// Mix regular and variadic parameters
function greet($greeting, ...$names) {
    foreach ($names as $name) {
        echo $greeting . ", " . $name . "!\n";
    }
}
greet("Hello", "Alice", "Bob", "Charlie");
// Hello, Alice!
// Hello, Bob!
// Hello, Charlie!
```

### Argument Unpacking

Use `...` to unpack arrays into function arguments:

```php
<?php
function add($a, $b, $c) {
    return $a + $b + $c;
}

$numbers = [1, 2, 3];
echo add(...$numbers); // 6

// Works with built-in functions too
$values = [3, 1, 4, 1, 5];
echo max(...$values); // 5
```

### Built-in Functions (95+)

```php
<?php
// String functions
echo strlen("Hello");              // 5
echo strtoupper("hello");          // HELLO
echo substr("Hello World", 0, 5);  // Hello
echo str_repeat("ab", 3);          // ababab

// Math functions
echo abs(-42);                     // 42
echo round(3.7);                   // 4
echo max(1, 5, 3);                 // 5

// Array functions
echo count([1, 2, 3]);             // 3
echo array_sum([1, 2, 3, 4]);    // 10

// JSON functions
$json = json_encode(["name" => "John", "age" => 30]);
$data = json_decode('{"name":"John","age":30}');

// File I/O
$content = file_get_contents("data.txt");
file_put_contents("output.txt", "Hello, World!");

// Formatting
echo sprintf("Name: %s, Age: %d", "John", 25);
```

#### String Functions (23)

`strlen`, `substr`, `strtoupper`, `strtolower`, `trim`, `ltrim`, `rtrim`, `str_repeat`, `str_replace`, `strpos`, `strrev`, `ucfirst`, `lcfirst`, `ucwords`, `str_starts_with`, `str_ends_with`, `str_contains`, `str_pad`, `explode`, `implode`/`join`, `sprintf`, `chr`, `ord`

#### Math Functions (9)

`abs`, `ceil`, `floor`, `round`, `max`, `min`, `pow`, `sqrt`, `rand`/`mt_rand`

#### Array Functions (21)

`count`/`sizeof`, `array_push`, `array_pop`, `array_shift`, `array_unshift`, `array_keys`, `array_values`, `in_array`, `array_search`, `array_reverse`, `array_merge`, `array_key_exists`, `range`, `array_first`, `array_last`, `array_map`, `array_filter`, `array_reduce`, `array_sum`, `array_unique`

#### Type Functions (14)

`intval`, `floatval`/`doubleval`, `strval`, `boolval`, `gettype`, `is_null`, `is_bool`, `is_int`/`is_integer`/`is_long`, `is_float`/`is_double`/`is_real`, `is_string`, `is_array`, `is_numeric`, `isset`, `empty`

#### Output Functions (4)

`print`, `var_dump`, `print_r`, `printf`

#### Reflection Functions (8)

`get_class_attributes`, `get_method_attributes`, `get_property_attributes`, `get_function_attributes`, `get_parameter_attributes`, `get_method_parameter_attributes`, `get_interface_attributes`, `get_trait_attributes`

#### JSON Functions (2)

`json_encode`, `json_decode`

#### File I/O Functions (10)

`file_get_contents`, `file_put_contents`, `file_exists`, `is_file`, `is_dir`, `filemtime`, `filesize`, `unlink`, `is_readable`, `is_writable`

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

### Constructor Property Promotion (PHP 8.0)

Shorthand syntax for declaring and initializing properties directly in the constructor:

```php
<?php
class User {
    public function __construct(
        public $name,
        private $age
    ) {}

    public function getAge() {
        return $this->age;
    }
}

$user = new User("Alice", 30);
echo $user->name;  // Alice
echo $user->getAge();  // 30
```

You can mix promoted and regular parameters:

```php
<?php
class Product {
    public function __construct(
        public $name,
        public $price,
        $currency = "USD"
    ) {
        echo "Price: $price $currency";
    }
}

$p = new Product("Widget", 9.99);  // Price: 9.99 USD
echo $p->name;  // Widget
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

### Static Properties (PHP 5.0+)

Static properties are class-level variables shared across all instances of a class. They are accessed using the class name or special keywords like `self::`, `parent::`, or `static::`.

#### Basic Static Property

```php
<?php
class Counter {
    public static $count = 0;
}

Counter::$count = 5;
echo Counter::$count;  // 5
```

#### Static Property with Visibility

```php
<?php
class Config {
    private static $debug = false;
    protected static $token = "abc";
    public static $api_key = "xyz";
}

echo Config::$api_key;  // xyz
```

#### Accessing Static Properties within Class Methods

Use `self::$property` to access static properties from within class methods:

```php
<?php
class Counter {
    public static $count = 0;

    public function increment() {
        self::$count++;
    }

    public static function getCount() {
        return self::$count;
    }
}

$a = new Counter();
$b = new Counter();
$a->increment();
$b->increment();
echo Counter::getCount();  // 2
```

#### Static Property Inheritance

Child classes inherit parent static properties but have separate storage:

```php
<?php
class Base {
    protected static $value = "base";

    public static function getValue() {
        return self::$value;
    }
}

class Child extends Base {
    protected static $value = "child";
}

echo Base::getValue();   // base
echo Child::getValue();  // child
```

#### Late Static Binding (PHP 5.3+)

Use `static::$property` for late static binding, which refers to the called class rather than the defined class:

```php
<?php
class Animal {
    protected static $type = "Animal";

    public static function getType() {
        return static::$type;  // LSB: refers to called class
    }
}

class Dog extends Animal {
    protected static $type = "Dog";
}

echo Animal::getType();  // Animal
echo Dog::getType();     // Dog
```

#### Difference Between self:: and static::

`self::` refers to the class where the method is defined, while `static::` refers to the class that was called (late static binding):

```php
<?php
class Base {
    protected static $name = "Base";

    public static function withSelf() {
        return self::$name;    // Always "Base"
    }

    public static function withStatic() {
        return static::$name;  // Called class's value
    }
}

class Child extends Base {
    protected static $name = "Child";
}

echo Child::withSelf();    // Base
echo Child::withStatic();  // Child
```

#### Readonly Static Properties (PHP 8.3+)

Static properties can be declared as readonly:

```php
<?php
class Config {
    public static readonly $version = "1.0.0";
}

echo Config::$version;       // 1.0.0
Config::$version = "2.0.0";  // Error: Cannot modify readonly property
```

#### Array Operations on Static Properties

Static properties can hold arrays and support array operations:

```php
<?php
class Collection {
    public static $items = [];
}

Collection::$items[] = "a";
Collection::$items[] = "b";
echo count(Collection::$items);  // 2
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

## Abstract Classes and Methods

Abstract classes provide a way to define base classes that cannot be instantiated directly. They can contain both abstract methods (without implementations) that must be implemented by child classes, and concrete methods with full implementations.

### Basic Abstract Class

```php
<?php
abstract class Animal {
    abstract function speak();
    
    function describe() {
        echo "I am an animal";
    }
}

class Dog extends Animal {
    function speak() {
        echo "Woof!";
    }
}

$dog = new Dog();
$dog->speak();     // Woof!
$dog->describe();  // I am an animal
```

### Cannot Instantiate Abstract Classes

Attempting to instantiate an abstract class directly results in an error:

```php
<?php
abstract class Shape {
    abstract function area();
}

$shape = new Shape();  // Error: Cannot instantiate abstract class Shape
```

### Multiple Abstract Methods

Abstract classes can have multiple abstract methods:

```php
<?php
abstract class Repository {
    abstract function find($id);
    abstract function save($entity);
    abstract function delete($id);
    
    function count() {
        return 0;  // Default implementation
    }
}

class UserRepository extends Repository {
    function find($id) {
        return "User $id";
    }
    
    function save($entity) {
        echo "Saving $entity";
    }
    
    function delete($id) {
        echo "Deleting $id";
    }
}
```

### Abstract Classes with Constructors

Abstract classes can have constructors that are called when child classes are instantiated:

```php
<?php
abstract class Entity {
    public $id;
    
    function __construct($id) {
        $this->id = $id;
    }
    
    abstract function getName();
}

class User extends Entity {
    public $name;
    
    function __construct($id, $name) {
        $this->id = $id;
        $this->name = $name;
    }
    
    function getName() {
        return $this->name;
    }
}

$user = new User(1, "John");
echo $user->getName();  // John
```

### Key Points

- Abstract classes cannot be instantiated directly
- Abstract methods have no body (no `{}`)
- Non-abstract child classes must implement all abstract methods
- Abstract classes can contain both abstract and concrete methods
- Abstract methods can only exist in abstract classes

## Asymmetric Visibility (PHP 8.4)

Asymmetric visibility allows different visibility modifiers for reading and writing properties. This enables the common pattern of public read access but restricted write access without requiring explicit getter/setter methods.

### Syntax

```php
<?php
class ClassName {
    public private(set) $property;     // Public read, private write
    public protected(set) $property;   // Public read, protected write
    protected private(set) $property;  // Protected read, private write
}
```

The syntax is: `<read-visibility> <write-visibility>(set) $property`

### Basic Usage

```php
<?php
class User {
    public private(set) string $name;

    public function __construct(string $name) {
        $this->name = $name;  // OK: write inside class
    }
}

$user = new User("Alice");
echo $user->name;      // OK: public read
$user->name = "Bob";   // Error: Cannot modify private property
```

### Public Read, Protected Write

```php
<?php
class Base {
    public protected(set) int $count = 0;
}

class Child extends Base {
    public function increment() {
        $this->count++;  // OK: protected write in subclass
    }
}

$child = new Child();
echo $child->count;     // OK: public read (0)
$child->increment();
echo $child->count;     // OK: public read (1)
$child->count = 5;      // Error: Cannot modify protected property
```

### Protected Read, Private Write

```php
<?php
class Base {
    protected private(set) string $status = "pending";

    private function updateStatus(string $newStatus) {
        $this->status = $newStatus;  // OK: private write inside class
    }
}

class Child extends Base {
    public function getStatus() {
        return $this->status;  // OK: protected read in subclass
    }

    public function setStatus(string $s) {
        $this->status = $s;  // Error: Cannot modify private property
    }
}
```

### With Static Properties

Asymmetric visibility works with static properties:

```php
<?php
class Config {
    public private(set) static int $version = 1;

    public static function upgrade() {
        self::$version++;  // OK: write inside class
    }
}

echo Config::$version;  // OK: public read (1)
Config::upgrade();
echo Config::$version;  // OK: public read (2)
Config::$version = 5;   // Error: Cannot modify private property
```

### Multiple Properties

Classes can have multiple properties with different asymmetric visibility:

```php
<?php
class Data {
    public private(set) int $id;
    public protected(set) string $name;
    public int $value;  // Symmetric visibility

    public function __construct(int $id, string $name, int $value) {
        $this->id = $id;
        $this->name = $name;
        $this->value = $value;
    }
}

$data = new Data(1, "test", 42);
echo $data->id;      // OK: public read
echo $data->name;    // OK: public read
echo $data->value;   // OK: public read

$data->value = 100;  // OK: public write
$data->id = 2;       // Error: Cannot modify private property
$data->name = "x";   // Error: Cannot modify protected property (outside class)
```

### With Inheritance

Write visibility is inherited and respected across the class hierarchy:

```php
<?php
class Base {
    public protected(set) string $status = "pending";
}

class Child extends Base {
    public function approve() {
        $this->status = "approved";  // OK: protected write in subclass
    }
}

$child = new Child();
echo $child->status;  // pending
$child->approve();
echo $child->status;  // approved
$child->status = "x"; // Error: Cannot modify protected property
```

### Validation Rules

The write visibility must be equal to or more restrictive than the read visibility:

**Valid combinations:**
- `public private(set)` ✓ (most common)
- `public protected(set)` ✓
- `protected private(set)` ✓

**Invalid combinations:**
- `private public(set)` ✗ (write cannot be less restrictive)
- `private protected(set)` ✗
- `protected public(set)` ✗
- `public public(set)` ✗ (redundant, just use `public`)
- `private private(set)` ✗ (redundant, just use `private`)

### Incompatibilities

Asymmetric visibility cannot be combined with:

**Readonly properties:**
```php
<?php
class User {
    public private(set) readonly string $name;  // Error
}
// Error: Readonly properties cannot have asymmetric visibility
```

Both features solve the same problem (restricting writes), so they're mutually exclusive.

**Property hooks with set:**
```php
<?php
class Temperature {
    public private(set) float $celsius {  // Error
        set {
            $this->celsius = $value;
        }
    }
}
// Error: Properties with set hooks cannot have asymmetric visibility
```

If you need custom set logic, use property hooks instead of asymmetric visibility.

### Use Cases

Asymmetric visibility is ideal for:

1. **Immutable public data** - Properties that should be publicly readable but only internally modifiable
2. **Controlled state changes** - Properties that can only be modified through specific methods
3. **Counter/statistics** - Public counters that can only be incremented internally
4. **Configuration** - Public config that can only be set during initialization
5. **IDs and timestamps** - Values that should never be changed after creation

### Notes

- Asymmetric visibility is purely a visibility control mechanism
- It doesn't affect property initialization in constructors
- Works with constructor property promotion
- Compatible with attributes
- Works with both instance and static properties
- Write visibility is checked at runtime based on the current class context

## Final Classes and Methods

The `final` keyword prevents classes from being extended and methods from being overridden.

### Final Classes

A final class cannot be extended by any other class:

```php
<?php
final class Singleton {
    public function getValue() {
        return 42;
    }
}

$obj = new Singleton();
echo $obj->getValue();  // 42

// class Extended extends Singleton {} // Error!
```

### Cannot Extend Final Classes

Attempting to extend a final class results in an error:

```php
<?php
final class Base {}
class Child extends Base {}  // Error: cannot extend final class Base
```

### Final Methods

A final method cannot be overridden by child classes:

```php
<?php
class Base {
    final public function locked() {
        return "locked";
    }
    
    public function unlocked() {
        return "base";
    }
}

class Child extends Base {
    // Cannot override locked()!
    
    public function unlocked() {
        return "child";
    }
}

$c = new Child();
echo $c->locked();    // locked
echo $c->unlocked();  // child
```

### Cannot Override Final Methods

Attempting to override a final method results in an error:

```php
<?php
class Base {
    final public function noOverride() {
        return "final";
    }
}

class Child extends Base {
    public function noOverride() {}  // Error: Cannot override final method
}
```

### Final Static Methods

Final can be combined with static:

```php
<?php
class Util {
    final public static function helper() {
        return "helped";
    }
}

echo Util::helper();  // helped
```

### Key Points

- Final classes cannot be extended by any class
- Final methods cannot be overridden by child classes
- `final` and `abstract` cannot be used together
- Private methods can technically be final but it's redundant
- Constructors can be final (prevents child from changing construction)

## Readonly Properties (PHP 8.1)

Readonly properties can only be assigned once and cannot be modified afterward. They're useful for immutable data structures.

### Basic Readonly Properties

```php
<?php
class Point {
    public readonly $x;
    public readonly $y;

    public function __construct($x, $y) {
        $this->x = $x;  // Assignment in constructor is allowed
        $this->y = $y;
    }
}

$p = new Point(10, 20);
echo $p->x;  // 10
$p->x = 30;  // Error: Cannot modify readonly property
```

### Constructor Property Promotion with Readonly

```php
<?php
class User {
    public function __construct(
        public readonly string $id,
        public readonly string $email
    ) {}
}

$user = new User("123", "user@example.com");
echo $user->id;  // 123
// Cannot modify $user->id or $user->email
```

## Property Hooks (PHP 8.4)

Property hooks allow you to define custom logic for getting and setting property values using `get` and `set` hooks. This enables computed properties, validation, and side effects without explicit getter/setter methods.

### Get Hook (Expression Syntax)

Use a get hook to compute property values dynamically.

**Syntax:**
```php
<?php
public type $property {
    get => expression;
}
```

**Example:**
```php
<?php
class Circle {
    public float $radius = 5.0;

    public float $diameter {
        get => $this->radius * 2;
    }
}

$c = new Circle();
echo $c->diameter;  // 10 (computed from radius)
$c->radius = 10.0;
echo $c->diameter;  // 20 (automatically updated)
```

### Set Hook (Block Syntax)

Use a set hook to validate, transform, or trigger side effects when setting a property. The incoming value is available via the special `$value` variable.

**Syntax:**
```php
<?php
public type $property {
    set {
        // Custom logic here
        // $value contains the incoming value
    }
}
```

**Example:**
```php
<?php
class Temperature {
    private float $celsius = 0;

    public float $fahrenheit {
        get => $this->celsius * 9/5 + 32;
        set {
            $this->celsius = ($value - 32) * 5/9;
        }
    }
}

$t = new Temperature();
$t->fahrenheit = 212;
echo $t->fahrenheit;  // 212
```

### Combined Get and Set Hooks

Properties can have both get and set hooks to create a complete abstraction.

**Example:**
```php
<?php
class User {
    private string $rawPassword = "";

    public string $password {
        get => "***REDACTED***";
        set {
            $this->rawPassword = hash('sha256', $value);
            echo "Password updated securely";
        }
    }
}

$user = new User();
$user->password = "secret123";  // Password updated securely
echo $user->password;           // ***REDACTED***
```

### Side Effects with Set Hooks

Set hooks can trigger logging, validation, or other side effects.

**Example:**
```php
<?php
class Logger {
    private array $log = [];

    public string $message {
        set {
            $this->log[] = $value;
            echo "Logged: " . $value;
        }
    }
}

$logger = new Logger();
$logger->message = "Hello";  // Logged: Hello
```

### Key Points

- **Expression syntax** (`get => expr`) for simple computed properties
- **Block syntax** (`set { ... }`) for complex set logic
- **$value variable** available in set hooks with the incoming value
- **Computed properties** don't need backing storage
- **Validation** can be performed in set hooks
- **Side effects** like logging can be triggered on property access
- **Type hints** work with property hooks
- **Visibility modifiers** (public, private, protected) apply to the hooks

### Limitations

- **No get-only writeable**: Get hooks make properties read-only by default
- **Set hooks require explicit backing storage** if you want to store the value
- **Hooks cannot be abstract** or in interfaces (PHP 8.4 limitation)

## Readonly Classes (PHP 8.2)

Readonly classes make all properties implicitly readonly without needing to mark each property individually.

### Basic Readonly Class

```php
<?php
readonly class Point {
    public function __construct(
        public $x,
        public $y
    ) {}
}

$p = new Point(1.5, 2.5);
echo $p->x;  // 1.5
$p->x = 3.0; // Error: Cannot modify readonly property
```

### Readonly Class with Explicit Properties

```php
<?php
readonly class User {
    public $name;
    private $age;

    public function __construct($name, $age) {
        $this->name = $name;
        $this->age = $age;
    }

    public function getAge() {
        return $this->age;
    }
}

$user = new User("John", 30);
echo $user->name;  // John
echo $user->getAge();  // 30
$user->name = "Jane";  // Error: Cannot modify readonly property
```

### Key Differences from Explicit Readonly Properties

- All properties are implicitly readonly (no need for `readonly` keyword on each property)
- Properties cannot have explicit `readonly` modifier (redundant)
- More concise for immutable classes
- All visibility modifiers work (public, protected, private)

## Object Cloning

### Basic Clone (PHP 5.0+)

The `clone` operator creates a shallow copy of an object. All property values are copied to the new object, but nested objects are copied by reference.

**Syntax:**

```php
<?php
$cloned = clone $original;
```

**Example:**

```php
<?php
class Point {
    public function __construct(
        public float $x,
        public float $y
    ) {}
}

$p1 = new Point(1.0, 2.0);
$p2 = clone $p1;

echo $p1->x;  // 1.0
echo $p2->x;  // 1.0

$p2->x = 3.0;
echo $p1->x;  // 1.0 (original unchanged)
echo $p2->x;  // 3.0
```

**Notes:**
- Creates a shallow copy (nested objects are shared by reference)
- Original object remains unchanged when modifying cloned object's properties
- Works with all classes and objects

### Clone With (PHP 8.4+)

The `clone with` syntax creates a copy while modifying specific properties in a single expression. This is especially useful for immutable objects with readonly properties.

**Syntax:**

```php
<?php
$cloned = clone $original with {
    property1: value1,
    property2: value2,
};
```

**Example:**

```php
<?php
readonly class ImmutablePoint {
    public function __construct(
        public float $x,
        public float $y
    ) {}
}

$p1 = new ImmutablePoint(1.0, 2.0);
$p2 = clone $p1 with { x: 3.0 };

echo $p1->x;  // 1.0
echo $p1->y;  // 2.0
echo $p2->x;  // 3.0
echo $p2->y;  // 2.0 (unchanged)
```

**Multiple Properties:**

```php
<?php
$p1 = new ImmutablePoint(1.0, 2.0);
$p2 = clone $p1 with { x: 3.0, y: 4.0 };

echo $p2->x;  // 3.0
echo $p2->y;  // 4.0
```

**With Expressions:**

Property values can be any expression, including references to the original object:

```php
<?php
$p1 = new Point(10.0, 5.0);
$p2 = clone $p1 with {
    x: $p1->x * 2.0,
    y: $p1->y + 10.0
};

echo $p2->x;  // 20.0
echo $p2->y;  // 15.0
```

**With Readonly Properties:**

Clone with allows re-initialization of readonly properties in the cloned object:

```php
<?php
class User {
    public function __construct(
        public readonly string $id,
        public readonly string $email
    ) {}
}

$user1 = new User("123", "old@example.com");
$user2 = clone $user1 with { email: "new@example.com" };

echo $user2->email;  // new@example.com
```

**Notes:**
- At least one property modification is required
- Modified properties must exist on the object
- Trailing commas are allowed: `clone $obj with { x: 1, }`
- Works seamlessly with readonly classes and properties
- Property values are evaluated at clone time

### Shallow Copy Behavior

Both `clone` and `clone with` perform shallow copies. If an object contains references to other objects, those references are copied (not the objects themselves):

```php
<?php
class Inner {
    public function __construct(public int $value) {}
}

class Outer {
    public function __construct(public Inner $inner) {}
}

$inner = new Inner(10);
$o1 = new Outer($inner);
$o2 = clone $o1;

$o2->inner->value = 20;

echo $o1->inner->value;  // 20 (affected by change to clone)
echo $o2->inner->value;  // 20
```

## Magic Methods

Magic methods are special methods that PHP calls automatically in certain situations. They enable custom behavior for common operations like string conversion, property access, and method calls.

### __toString

Called when an object is used in a string context (echo, concatenation, etc.).

```php
<?php
class User {
    private $name;

    public function __construct($name) {
        $this->name = $name;
    }

    public function __toString(): string {
        return $this->name;
    }
}

$user = new User("Alice");
echo $user;           // Alice
echo "Hello, " . $user;  // Hello, Alice
```

### __invoke

Called when an object is used as a function.

```php
<?php
class Adder {
    private $base;

    public function __construct($base) {
        $this->base = $base;
    }

    public function __invoke($n) {
        return $this->base + $n;
    }
}

$add5 = new Adder(5);
echo $add5(10);  // 15
```

### __get and __set

Called when accessing or setting undefined/inaccessible properties.

```php
<?php
class MagicProps {
    private $data = [];

    public function __get($name) {
        return $this->data[$name] ?? "undefined";
    }

    public function __set($name, $value) {
        $this->data[$name] = $value;
    }
}

$obj = new MagicProps();
$obj->foo = "bar";
echo $obj->foo;      // bar
echo $obj->missing;  // undefined
```

### __isset and __unset

Called when `isset()` or `unset()` is used on undefined/inaccessible properties.

```php
<?php
class Container {
    private $items = [];

    public function __set($name, $value) {
        $this->items[$name] = $value;
    }

    public function __isset($name) {
        return isset($this->items[$name]);
    }

    public function __unset($name) {
        unset($this->items[$name]);
    }
}

$c = new Container();
$c->key = "value";
echo isset($c->key) ? "yes" : "no";  // yes
unset($c->key);
echo isset($c->key) ? "yes" : "no";  // no
```

### __call and __callStatic

Called when invoking undefined/inaccessible methods.

```php
<?php
class Wrapper {
    public function __call($method, $args) {
        return "Called $method with " . count($args) . " args";
    }

    public static function __callStatic($method, $args) {
        return "Static: $method";
    }
}

$w = new Wrapper();
echo $w->unknownMethod(1, 2, 3);  // Called unknownMethod with 3 args
echo Wrapper::anything();         // Static: anything
```

### Supported Magic Methods

| Method | Purpose | Status |
|--------|---------|--------|
| `__construct` | Object initialization | Implemented |
| `__toString` | String conversion | Implemented |
| `__invoke` | Callable objects | Implemented |
| `__get` | Property read overloading | Implemented |
| `__set` | Property write overloading | Implemented |
| `__isset` | isset() overloading | Implemented |
| `__unset` | unset() overloading | Implemented |
| `__call` | Method call overloading | Implemented |
| `__callStatic` | Static method call overloading | Implemented |
| `__clone` | Clone behavior | Implemented |

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

## Attributes (PHP 8.0)

Attributes provide a way to add structured metadata to declarations. VHP currently supports parsing attribute syntax and storing them in the AST.

### Basic Attribute Syntax

```php
<?php
#[Route("/api/users")]
class UserController {
    public function index() {
        echo "Users list";
    }
}

$controller = new UserController();
$controller->index();  // Users list
```

### Attributes with Arguments

Attributes can accept both positional and named arguments:

```php
<?php
// Positional arguments
#[Route("/posts", "GET")]
class PostController {
    public function list() {
        echo "Posts";
    }
}

// Named arguments
#[Route(path: "/users", method: "POST")]
class UserController {
    public function create() {
        echo "Create user";
    }
}

// Mixed arguments
#[Cache(3600, driver: "redis")]
class DataService {
    public function fetch() {
        echo "Fetching data";
    }
}
```

### Multiple Attributes

Classes, methods, properties, and functions can have multiple attributes:

```php
<?php
// Multiple attributes on separate lines
#[Deprecated]
#[Replaced(by: "UserServiceV2")]
interface UserService {
    function getUsers();
}

// Multiple attributes in single brackets
#[Route("/admin"), Authenticated, RateLimit(100)]
class AdminController {
    public function dashboard() {
        echo "Admin dashboard";
    }
}
```

### Attributes on Different Declarations

Attributes can be applied to various language constructs:

```php
<?php
// On classes
#[Entity(table: "users")]
class User {}

// On interfaces
#[Deprecated]
interface OldInterface {}

// On traits
#[Internal]
trait HelperMethods {}

// On methods
class Controller {
    #[Route("/profile")]
    public function profile() {
        echo "Profile";
    }
}

// On properties
class Model {
    #[Column(type: "string", length: 255)]
    public $name;
}

// On functions
#[Pure]
function calculate($x) {
    return $x * 2;
}

// On parameters (including constructor promotion)
class Point {
    public function __construct(
        #[Positive] public $x,
        #[Positive] public $y
    ) {}
}
```

### Runtime Attribute Reflection

VHP provides built-in functions to retrieve attributes at runtime:

```php
<?php
#[Route("/api/users")]
class UserController {
    #[ValidateRequest]
    public function create(#[FromBody] $data) {}
}

// Get class attributes
$class_attrs = get_class_attributes("UserController");
// Returns: [["name" => "Route", "arguments" => [["name" => null, "value" => "/api/users"]]]]

// Get method attributes
$method_attrs = get_method_attributes("UserController", "create");

// Get parameter attributes
$param_attrs = get_method_parameter_attributes("UserController", "create", "data");
```

**Available Reflection Functions:**

| Function | Description |
|----------|-------------|
| `get_class_attributes($class)` | Get all attributes for a class |
| `get_method_attributes($class, $method)` | Get all attributes for a method |
| `get_property_attributes($class, $property)` | Get all attributes for a property |
| `get_function_attributes($function)` | Get all attributes for a function |
| `get_parameter_attributes($function, $param)` | Get all attributes for a function parameter |
| `get_method_parameter_attributes($class, $method, $param)` | Get all attributes for a method parameter |
| `get_interface_attributes($interface)` | Get all attributes for an interface |
| `get_trait_attributes($trait)` | Get all attributes for a trait |

**Return Format:**

Each function returns an array of attributes. Each attribute is an associative array:

```php
<?php
[
    "name" => "AttributeName",
    "arguments" => [
        [
            "name" => "param_name",  // null for positional args
            "value" => "param_value"
        ]
    ]
]
```

### Current Implementation Status

VHP fully supports:
- ✅ Parsing attribute syntax
- ✅ Storing attributes in the AST
- ✅ All attribute argument forms (positional, named, mixed)
- ✅ Attributes on all declarations (classes, methods, properties, functions, parameters, etc.)
- ✅ Attribute reflection API for runtime retrieval

## Enums (PHP 8.1)

Enums (Enumerations) provide a way to define a type with a fixed set of possible values. VHP supports both pure enums (cases without values) and backed enums (cases backed by int or string values).

### Pure Enums

Pure enums define named cases without scalar backing values.

**Syntax:**

```php
<?php
enum EnumName {
    case CaseName;
    case AnotherCase;
}
```

**Example:**

```php
<?php
enum Status {
    case Pending;
    case Active;
    case Archived;
}

$status = Status::Active;
echo $status->name;  // Active
```

**Key Points:**
- Cases have no backing values
- Access the case name via the `->name` property
- No `->value` property available

### Backed Enums (Int)

Backed enums with integer values provide both name and value properties.

**Syntax:**

```php
<?php
enum EnumName: int {
    case CaseName = value;
}
```

**Example:**

```php
<?php
enum Priority: int {
    case Low = 1;
    case Medium = 5;
    case High = 10;
}

$priority = Priority::High;
echo $priority->name;   // High
echo $priority->value;  // 10
```

### Backed Enums (String)

Backed enums with string values work similarly to int-backed enums.

**Syntax:**

```php
<?php
enum EnumName: string {
    case CaseName = 'value';
}
```

**Example:**

```php
<?php
enum Color: string {
    case Red = 'red';
    case Green = 'green';
    case Blue = 'blue';
}

$color = Color::Red;
echo $color->name;   // Red
echo $color->value;  // red
```

### Built-in Enum Methods

All enums have access to built-in methods for introspection and validation.

#### cases()

Returns an array of all enum cases.

```php
<?php
enum Status {
    case Pending;
    case Active;
    case Archived;
}

$cases = Status::cases();
echo count($cases);        // 3
echo $cases[0]->name;      // Pending
echo $cases[1]->name;      // Active
```

**Works with:** Pure and backed enums

#### from()

Retrieves an enum case by its backing value. Throws an error if the value is not found.

```php
<?php
enum Priority: int {
    case Low = 1;
    case Medium = 5;
    case High = 10;
}

$priority = Priority::from(5);
echo $priority->name;  // Medium

// Error: Value not found
$invalid = Priority::from(99);  // Error: Value '99' is not a valid backing value
```

**Works with:** Backed enums only (int or string)
**Throws:** Error if value not found

#### tryFrom()

Retrieves an enum case by its backing value. Returns `null` if the value is not found (safe version of `from()`).

```php
<?php
enum Priority: int {
    case Low = 1;
    case Medium = 5;
    case High = 10;
}

$priority = Priority::tryFrom(5);
echo $priority->name;  // Medium

$invalid = Priority::tryFrom(99);
var_dump($invalid);  // NULL
```

**Works with:** Backed enums only (int or string)
**Returns:** Enum case or `null` if not found

### Enum Case Properties

Enum cases expose properties for introspection:

| Property | Available On | Description |
|----------|--------------|-------------|
| `->name` | All enums | String name of the case |
| `->value` | Backed enums only | Backing value (int or string) |

**Example:**

```php
<?php
enum Status {
    case Pending;
}

enum Priority: int {
    case Low = 1;
}

$status = Status::Pending;
echo $status->name;   // Pending
// $status->value;    // Error: Pure enum case doesn't have 'value' property

$priority = Priority::Low;
echo $priority->name;   // Low
echo $priority->value;  // 1
```

### Using Enums

#### In Variables

```php
<?php
enum Status {
    case Active;
    case Inactive;
}

$current = Status::Active;
echo $current->name;  // Active
```

#### In Arrays

```php
<?php
enum Status {
    case Pending;
    case Active;
    case Archived;
}

$statuses = [Status::Pending, Status::Active];
echo $statuses[0]->name;  // Pending
echo $statuses[1]->name;  // Active
```

#### In Switch Statements

```php
<?php
enum Status {
    case Pending;
    case Active;
    case Archived;
}

$status = Status::Active;

switch ($status->name) {
    case "Pending":
        echo "Waiting";
        break;
    case "Active":
        echo "Running";
        break;
    case "Archived":
        echo "Done";
        break;
}
// Output: Running
```

#### In Comparisons

```php
<?php
enum Status {
    case Pending;
    case Active;
}

$s1 = Status::Active;
$s2 = Status::Active;
$s3 = Status::Pending;

var_dump($s1 === $s2);  // bool(true)
var_dump($s1 === $s3);  // bool(false)
```

### Enum Validation

VHP enforces strict validation rules for enums:

**Pure Enums:**
- Cannot have case values
- Must have at least one case
- Case names must be unique

**Backed Enums:**
- Must declare backing type (`: int` or `: string`)
- All cases must have values matching the backing type
- Backing values must be unique
- Must have at least one case

**Error Examples:**

```php
<?php
// Error: Pure enum cannot have case values
enum Status {
    case Pending = 1;  // Error
}

// Error: Backed enum must have case values
enum Priority: int {
    case Low;  // Error: missing value
}

// Error: Wrong backing type
enum Color: int {
    case Red = "red";  // Error: string value for int-backed enum
}

// Error: Duplicate values
enum Level: int {
    case Low = 1;
    case Medium = 1;  // Error: duplicate value
}
```

### Notes

- **Case Sensitivity**: Enum names are case-insensitive (like classes), but case names are case-sensitive (`Status::Active` ≠ `Status::ACTIVE`)
- **Type Safety**: Enum cases don't automatically coerce to other types
- **String Representation**: When converted to string, displays as `EnumName::CaseName`
- **Boolean Context**: All enum cases are truthy
- **Comparison**: Use strict comparison (`===`) to compare enum cases

## #[\Override] Attribute (PHP 8.3)

The `#[\Override]` attribute explicitly marks methods that override parent class, interface, or trait methods. The PHP engine validates that the method actually overrides something at class definition time, catching typos and refactoring errors.

### Basic Usage

```php
<?php
class Animal {
    public function makeSound() {
        return "...";
    }
}

class Dog extends Animal {
    #[\Override]
    public function makeSound() {
        return "Woof!";
    }
}

$dog = new Dog();
echo $dog->makeSound();  // Woof!
```

### Why Use #[\Override]?

**Without `#[\Override]` - Silent failure:**
```php
<?php
class Animal {
    public function makeSound() {
        return "...";
    }
}

class Cat extends Animal {
    // Typo! Should be makeSound, but no error
    public function makeSounds() {
        return "Meow!";
    }
}

$cat = new Cat();
echo $cat->makeSound();  // "..." (wrong - uses parent method)
```

**With `#[\Override]` - Caught at definition time:**
```php
<?php
class Bird extends Animal {
    #[\Override]
    public function makeSounds() {  // Error immediately!
        return "Tweet!";
    }
}
// Error: Bird::makeSounds has #[\Override] attribute, but no matching parent method exists
```

### Validating Interface Methods

The `#[\Override]` attribute works with interface implementation:

```php
<?php
interface Drawable {
    public function draw();
}

class Circle implements Drawable {
    #[\Override]
    public function draw() {
        return "Drawing circle";
    }
}

$circle = new Circle();
echo $circle->draw();  // Drawing circle
```

### Validating Trait Methods

Works with trait composition:

```php
<?php
trait Greetable {
    public function greet() {
        return "Hello";
    }
}

class Person {
    use Greetable;

    #[\Override]
    public function greet() {
        return "Hi there!";
    }
}

$person = new Person();
echo $person->greet();  // Hi there!
```

### Validating Abstract Methods

Implementing abstract methods counts as overriding:

```php
<?php
abstract class Shape {
    abstract public function area();
}

class Circle extends Shape {
    private $radius = 5;

    #[\Override]
    public function area() {
        return 3.14 * $this->radius * $this->radius;
    }
}

$circle = new Circle();
echo $circle->area();  // 78.5
```

### Inheritance Chain Validation

`#[\Override]` validates against the entire parent hierarchy:

```php
<?php
class GrandParent {
    public function legacy() {
        return "old";
    }
}

class Parent extends GrandParent {
    // Doesn't override legacy()
}

class Child extends Parent {
    #[\Override]
    public function legacy() {
        return "new";  // Valid: overrides GrandParent::legacy()
    }
}

$child = new Child();
echo $child->legacy();  // new
```

### Multiple Interfaces

Works with multiple interface implementations:

```php
<?php
interface A {
    public function foo();
}

interface B {
    public function bar();
}

class C implements A, B {
    #[\Override]
    public function foo() {
        return "foo";
    }

    #[\Override]
    public function bar() {
        return "bar";
    }
}

$c = new C();
echo $c->foo() . $c->bar();  // foobar
```

### Case Insensitivity

Both the attribute name and method name matching are case-insensitive:

```php
<?php
class Base {
    public function Method() {
        return "base";
    }
}

class Child extends Base {
    #[\override]  // lowercase attribute works
    public function method() {  // case-insensitive method match
        return "child";
    }
}

$child = new Child();
echo $child->method();  // child
```

### Error Cases

**No parent method:**
```php
<?php
class Standalone {
    #[\Override]
    public function test() {
        return "fail";
    }
}
// Error: Standalone::test has #[\Override] attribute, but no matching parent method exists
```

**Typo in method name:**
```php
<?php
class Animal {
    public function speak() {
        return "...";
    }
}

class Dog extends Animal {
    #[\Override]
    public function bark() {  // Error: doesn't override speak()
        return "Woof!";
    }
}
// Error: Dog::bark has #[\Override] attribute, but no matching parent method exists
```

### Override Sources

A method marked with `#[\Override]` must exist in one of:

1. **Parent class** (any ancestor in the hierarchy)
2. **Implemented interface** (any interface or parent interface)
3. **Used trait** (any trait or nested trait)

### Key Points

- **Validation timing**: Checked at class definition time, not at method call time
- **Case insensitive**: Both attribute name and method names are case-insensitive
- **Works with**: Parent classes, interfaces, traits, abstract methods, magic methods
- **Error format**: `ClassName::methodName has #[\Override] attribute, but no matching parent method exists`
- **Static methods**: Can use `#[\Override]` on static method overrides
- **Constructor**: Can use `#[\Override]` on `__construct` if parent has it
- **PHP 8.3 feature**: Part of the PHP 8.3 release

### Use Cases

1. **Refactoring safety**: Catch method name changes in parent classes
2. **API contract**: Document that a method is meant to override parent behavior
3. **Typo prevention**: Detect typos in override method names at definition time
4. **Interface compliance**: Ensure interface methods are actually implemented
5. **Team coordination**: Make override intent explicit for code reviewers

## Pipe Operator (PHP 8.5)

The pipe operator (`|>`) enables functional-style function chaining, making data transformations more readable by flowing left-to-right.

### Basic Usage

Instead of nesting function calls:
```php
<?php
$result = strtoupper(trim("  hello  "));
echo $result;  // HELLO
```

Use the pipe operator:
```php
<?php
$text = "  hello  ";
$result = $text |> trim(...) |> strtoupper(...);
echo $result;  // HELLO
```

### With Additional Arguments

The piped value is inserted as the first argument, with additional arguments following:

```php
<?php
$text = "hello world";
$result = $text |> substr(..., 0, 5) |> strtoupper(...);
echo $result;  // HELLO
```

### Multiple Transformations

Chain multiple functions for complex data transformations:

```php
<?php
$text = "  HELLO WORLD  ";
$result = $text
    |> trim(...)
    |> strtolower(...)
    |> ucfirst(...);
echo $result;  // Hello world
```

### With User-Defined Functions

Works seamlessly with custom functions:

```php
<?php
function double($x) {
    return $x * 2;
}

function addTen($x) {
    return $x + 10;
}

$result = 5 |> double(...) |> addTen(...);
echo $result;  // 20 (5 * 2 + 10)
```

### Benefits

- **Readability**: Operations flow in the order they're applied (left-to-right)
- **Composability**: Easy to add or remove transformation steps
- **Functional Style**: Enables point-free programming patterns
- **Clarity**: Avoids deeply nested function calls

### Technical Details

- **Precedence**: Lower precedence than most operators but higher than assignment
- **Associativity**: Left-associative, so `a |> b |> c` evaluates as `(a |> b) |> c`
- **Placeholder**: The `...` syntax indicates where the piped value is inserted (always as the first argument)
- **Function Types**: Works with built-in functions and user-defined functions

### Example: Traditional vs. Pipe

**Traditional nested calls:**
```php
<?php
$result = ucfirst(strtolower(trim("  HELLO WORLD  ")));
```

**With pipe operator:**
```php
<?php
$result = "  HELLO WORLD  "
    |> trim(...)
    |> strtolower(...)
    |> ucfirst(...);
```

Both produce the same output (`"Hello world"`), but the pipe version is more readable and easier to modify.

## Fibers (PHP 8.1)

Fibers provide cooperative multitasking with full-stack, interruptible functions. Unlike generators, Fibers can be suspended from anywhere in the call stack and maintain their own execution context.

### Core Concepts

- **Full-stack interruption**: Can suspend from deeply nested function calls
- **Cooperative multitasking**: Explicit suspend/resume control
- **Own call stack**: Each Fiber maintains independent execution state

### Basic Usage

```php
<?php
function fiberFunction() {
    echo "Start of fiber\n";
    $value = Fiber::suspend("suspended");
    echo "Resumed with: " . $value . "\n";
    return "fiber_result";
}

$fiber = new Fiber('fiberFunction');

// Start the fiber - runs until suspend
$suspendedValue = $fiber->start();
echo "Fiber suspended with: " . $suspendedValue . "\n";

// Resume with a value
$result = $fiber->resume("resume_data");
echo "Fiber returned: " . $result . "\n";
```

Output:
```
Start of fiber
Fiber suspended with: suspended
Resumed with: resume_data
Fiber returned: fiber_result
```

### Fiber API

#### Constructor
```php
$fiber = new Fiber(callable $callback);
```

#### Control Methods
```php
$fiber->start(mixed ...$args): mixed     // Start execution
$fiber->resume(mixed $value = null): mixed  // Resume from suspension
$fiber->getReturn(): mixed               // Get return value after termination
```

#### Status Methods
```php
$fiber->isStarted(): bool      // Has the fiber been started?
$fiber->isSuspended(): bool    // Is currently suspended?
$fiber->isTerminated(): bool   // Has execution completed?
```

#### Static Methods
```php
Fiber::suspend(mixed $value = null): mixed  // Suspend current fiber
Fiber::getCurrent(): ?Fiber                 // Get currently running fiber
```

### State Management

```php
<?php
function testFunction() {
    return 42;
}

$fiber = new Fiber('testFunction');

// Before starting
echo $fiber->isStarted() ? "true" : "false";    // false
echo $fiber->isSuspended() ? "true" : "false";  // false
echo $fiber->isTerminated() ? "true" : "false"; // false

$result = $fiber->start();

// After completion
echo $fiber->isStarted() ? "true" : "false";    // true
echo $fiber->isSuspended() ? "true" : "false";  // false
echo $fiber->isTerminated() ? "true" : "false"; // true
```

### Current Implementation Limitations

VHP's Fiber implementation provides the core API and basic functionality:

- ✅ Basic Fiber creation and execution
- ✅ State checking methods (`isStarted`, `isSuspended`, `isTerminated`)
- ✅ `Fiber::getCurrent()` and `Fiber::suspend()`
- ✅ Return value handling with `getReturn()`
- ⚠️ **Suspend/resume is MVP-limited** - Full call-stack suspension requires additional runtime support

The current implementation covers the essential Fiber API and enables basic cooperative multitasking patterns. Advanced suspend/resume scenarios may require additional development.

## Exception Handling (PHP 8.0)

VHP provides comprehensive exception handling with try/catch/finally blocks, throw statements and expressions, and support for exception inheritance.

### Basic Try/Catch

The simplest form of exception handling uses `try` to wrap potentially failing code and `catch` to handle exceptions.

**Syntax:**
```php
<?php
try {
    // Code that may throw an exception
} catch (ExceptionType $variable) {
    // Handle the exception
}
```

**Example:**
```php
<?php
try {
    throw new Exception("Something went wrong");
} catch (Exception $e) {
    echo "Caught: " . $e->getMessage();
}
// Output: Caught: Something went wrong
```

### Exception Class

The base `Exception` class provides methods to retrieve information about the exception.

**Available Methods:**
- `getMessage()` - Returns the exception message string
- `getCode()` - Returns the exception code (integer)

**Example:**
```php
<?php
try {
    throw new Exception("Error message", 500);
} catch (Exception $e) {
    echo $e->getMessage();  // Error message
    echo $e->getCode();     // 500
}
```

### Multiple Catch Blocks

Handle different exception types with separate catch blocks. Catch blocks are evaluated in order, and the first matching type is executed.

**Syntax:**
```php
<?php
try {
    // Code
} catch (SpecificException $e) {
    // Handle specific exception
} catch (Exception $e) {
    // Handle general exception
}
```

**Example:**
```php
<?php
class ValidationException extends Exception {}
class DatabaseException extends Exception {}

function process($data) {
    if (empty($data)) {
        throw new ValidationException("Data is empty");
    }
    throw new DatabaseException("Connection failed");
}

try {
    process("");
} catch (ValidationException $e) {
    echo "Validation error: " . $e->getMessage();
} catch (DatabaseException $e) {
    echo "Database error: " . $e->getMessage();
} catch (Exception $e) {
    echo "Unknown error: " . $e->getMessage();
}
// Output: Validation error: Data is empty
```

### Multi-Catch (PHP 7.1)

Catch multiple exception types in a single block using the pipe operator (`|`). This is useful when the same handling logic applies to different exception types.

**Syntax:**
```php
<?php
catch (TypeA | TypeB | TypeC $e) {
    // Handle any of these types
}
```

**Example:**
```php
<?php
class NetworkException extends Exception {}
class TimeoutException extends Exception {}

try {
    throw new NetworkException("Connection lost");
} catch (NetworkException | TimeoutException $e) {
    echo "Communication error: " . $e->getMessage();
}
// Output: Communication error: Connection lost
```

### Try/Catch/Finally

The `finally` block always executes, regardless of whether an exception was thrown or caught. Use it for cleanup operations.

**Syntax:**
```php
<?php
try {
    // Code that may throw
} catch (Exception $e) {
    // Handle exception
} finally {
    // Always executes
}
```

**Example:**
```php
<?php
try {
    echo "try\n";
    throw new Exception("error");
} catch (Exception $e) {
    echo "catch\n";
} finally {
    echo "finally\n";
}
// Output:
// try
// catch
// finally
```

### Finally Without Catch

A `finally` block can exist without a `catch` block. The exception will propagate after the finally block executes.

**Example:**
```php
<?php
function cleanup() {
    try {
        echo "Opening resource\n";
        throw new Exception("Failed");
    } finally {
        echo "Cleanup\n";
    }
}

try {
    cleanup();
} catch (Exception $e) {
    echo "Caught: " . $e->getMessage();
}
// Output:
// Opening resource
// Cleanup
// Caught: Failed
```

### Throw as Expression (PHP 8.0)

In PHP 8.0+, `throw` can be used as an expression in contexts that previously only allowed values, such as arrow functions, null coalescing operators, and ternary expressions.

**With Null Coalescing:**
```php
<?php
function getValue($value) {
    return $value ?? throw new Exception("Value required");
}

try {
    $result = getValue(null);
} catch (Exception $e) {
    echo "Error: " . $e->getMessage();
}
// Output: Error: Value required
```

**With Ternary:**
```php
<?php
$age = -5;
$result = $age >= 0 
    ? "Age is $age" 
    : throw new Exception("Invalid age");
```

**In Arrow Functions:**
```php
<?php
$validate = fn($x) => $x > 0 ? $x : throw new Exception("Must be positive");

try {
    echo $validate(-1);
} catch (Exception $e) {
    echo $e->getMessage();  // Must be positive
}
```

### Exception Inheritance

Custom exception classes can extend `Exception` or other exception classes, enabling hierarchical exception handling.

**Example:**
```php
<?php
class CustomException extends Exception {}
class ChildException extends CustomException {}

// Catching parent catches children
try {
    throw new ChildException("child error");
} catch (CustomException $e) {
    echo "Caught via parent: " . $e->getMessage();
}
// Output: Caught via parent: child error
```

### Nested Try/Catch

Try/catch blocks can be nested to handle exceptions at different levels.

**Example:**
```php
<?php
try {
    echo "outer try\n";
    try {
        echo "inner try\n";
        throw new Exception("inner exception");
    } catch (Exception $e) {
        echo "inner catch: " . $e->getMessage() . "\n";
        throw new Exception("re-thrown");
    }
} catch (Exception $e) {
    echo "outer catch: " . $e->getMessage();
}
// Output:
// outer try
// inner try
// inner catch: inner exception
// outer catch: re-thrown
```

### Uncaught Exceptions

If an exception is not caught, VHP will terminate execution and display an error message.

**Example:**
```php
<?php
throw new Exception("This will crash");
// Fatal error: Uncaught Exception: This will crash
```

### Best Practices

- **Specific Before General**: Order catch blocks from most specific to most general exception types
- **Use Finally for Cleanup**: Place resource cleanup (file handles, connections) in finally blocks
- **Informative Messages**: Provide clear, actionable error messages
- **Appropriate Exception Types**: Create custom exception classes for different error categories
- **Don't Catch Everything**: Only catch exceptions you can handle appropriately
- **Preserve Context**: When re-throwing, consider preserving the original exception context

### Notes

- **Case-insensitive**: Exception class names follow PHP's case-insensitive class naming
- **Exception Hierarchy**: All exceptions must extend the base `Exception` class
- **Finally Always Runs**: The finally block executes even if there's a return statement in try or catch
- **Multi-catch Order**: In multi-catch, types are checked left to right
- **Expression Context**: Throw expressions have higher precedence than most operators

## Type Declarations (PHP 7.0+)

VHP supports PHP's type declaration syntax for parameters and return types with **full runtime validation**. Type mismatches throw descriptive TypeErrors.

### Parameter Type Hints

Type hints are validated at runtime to ensure type safety.

```php
<?php
function greet(string $name, int $age): void {
    echo "Hello, $name. You are $age years old.";
}

greet("Alice", 30); // Output: Hello, Alice. You are 30 years old.
greet(123, "thirty"); // TypeError: Expected string for parameter $name, got int
```

**Supported simple types:**
- `int` - Integer values
- `float` - Floating-point values
- `string` - String values
- `bool` - Boolean values (true/false)
- `array` - Array values
- `object` - Object instances
- `callable` - Callable values (functions, closures)
- `iterable` - Arrays or Traversable objects
- `mixed` - Any type (PHP 8.0+)

### Nullable Types (PHP 7.1)

Prefix a type with `?` to allow null values.

```php
<?php
function setName(?string $name): void {
    if ($name === null) {
        echo "Name not provided";
    } else {
        echo "Name: $name";
    }
}

setName(null);    // Output: Name not provided
setName("Bob");   // Output: Name: Bob
setName(123);     // TypeError: Expected ?string, got int
```

### Union Types (PHP 8.0)

Use `|` to accept multiple types.

```php
<?php
function process(int|string $value): string {
    if (is_int($value)) {
        return "Number: " . $value;
    }
    return "Text: " . $value;
}

echo process(42);      // Output: Number: 42
echo process("hello"); // Output: Text: hello
echo process(true);    // TypeError: Expected int|string, got bool
```

Union types can include null:

```php
<?php
function getValue(): int|float|null {
    return null;
}
```

### Intersection Types (PHP 8.1)

Use `&` to require multiple types (typically interfaces).

```php
<?php
function process(Iterator&Countable $collection): int {
    return count($collection);
}
```

### DNF Types (PHP 8.2)

DNF (Disjunctive Normal Form) types allow complex type declarations combining union (`|`) and intersection (`&`) types. The format is `(A&B)|C` meaning "either an object that implements both A and B, or just C".

**Syntax:**

```php
<?php
// Basic DNF: (A&B)|C
function process((Loggable&Serializable)|Cacheable $obj): void {
    // Accept objects that implement both Loggable AND Serializable,
    // OR objects that implement Cacheable
}

// Multiple intersections: (A&B)|(C&D)
function handle((Iterator&Countable)|(ArrayAccess&Traversable) $collection): void {
    // Complex type combinations
}

// DNF with simple type: (A&B)|C|D
function accept((A&B)|C|D $value): void {
    // Mixed intersection and simple types
}
```

**Example:**

```php
<?php
interface Loggable {
    public function log();
}

interface Serializable {
    public function serialize();
}

interface Cacheable {
    public function cache();
}

// Accept either (Loggable AND Serializable) OR Cacheable
function process((Loggable&Serializable)|Cacheable $obj): void {
    if ($obj instanceof Loggable) {
        $obj->log();
    }
    if ($obj instanceof Cacheable) {
        $obj->cache();
    }
}

class LogSerializable implements Loggable, Serializable {
    public function log() { echo "Logged\n"; }
    public function serialize() { return "serialized"; }
}

class Cache implements Cacheable {
    public function cache() { echo "Cached\n"; }
}

process(new LogSerializable());  // OK: matches (Loggable&Serializable)
process(new Cache());            // OK: matches Cacheable
```

**With Return Types:**

```php
<?php
interface I1 {}
interface I2 {}
interface I3 {}

class A implements I1, I2 {}

function get(): (I1&I2)|I3 {
    return new A();  // Returns object matching (I1&I2)
}

$result = get();
echo $result instanceof I1 ? "yes" : "no";  // yes
```

**Multiple Intersection Groups:**

```php
<?php
interface A {}
interface B {}
interface C {}
interface D {}

class AB implements A, B {}
class CD implements C, D {}

function test((A&B)|(C&D) $obj): string {
    return get_class($obj);
}

echo test(new AB());  // AB
echo test(new CD());  // CD
```

**Notes:**

- **DNF Form Required**: Must be in Disjunctive Normal Form: `(A&B)|C` is valid, but `A&(B|C)` is not
- **Parentheses Required**: Intersections in unions must use parentheses: `(A&B)|C`
- **Type Mixing**: Can mix single types with intersections: `(A&B)|C|D`
- **Validation**: At runtime, the value must match at least one "branch" of the union
- **Intersection Matching**: To match an intersection branch, must implement ALL types in that intersection
- **Class/Interface Types Only**: Only class/interface types can be in intersections (not int, string, etc.)
- **Inheritance Support**: Subclasses that implement required interfaces match the type

**Error Handling:**

If a value doesn't match any branch of the DNF type, a TypeError is thrown:

```php
<?php
interface A {}
interface B {}
interface C {}

class OnlyA implements A {}

function process((A&B)|C $obj): void {
    echo "OK";
}

// TypeError: Expected type (A&B)|C, got OnlyA
// OnlyA only implements A, not both A and B, and not C
process(new OnlyA());
```

### Return Type Declarations

Specify the type a function returns using `: type` after the parameter list.

```php
<?php
function add(int $a, int $b): int {
    return $a + $b;
}

function getUser(): array {
    return ["name" => "Alice", "age" => 30];
}
```

**Special return types:**
- `void` - Function returns nothing (PHP 7.1)
- `never` - Function never returns (throws or exits) (PHP 8.1)
- `static` - Returns instance of called class (PHP 8.0)
- `self` - Returns instance of declaring class
- `parent` - Returns instance of parent class

```php
<?php
function log(string $message): void {
    echo $message;
    // No return statement needed
}

function fail(): never {
    throw new Exception("Always fails");
}
```

### Class Type Hints

Use class names as type hints:

```php
<?php
class User {
    public $name;
}

function processUser(User $user): string {
    return $user->name;
}
```

### Method Type Hints

Type hints work the same way in class methods:

```php
<?php
class Calculator {
    public function add(int $a, int $b): int {
        return $a + $b;
    }
    
    public function divide(float $a, float $b): ?float {
        if ($b === 0.0) {
            return null;
        }
        return $a / $b;
    }
}
```

### Runtime Type Validation

**Type validation is fully implemented:**
- ✅ Type declarations are parsed and stored in the AST
- ✅ All PHP 7.0-8.1 type syntax is supported
- ✅ Types are validated at runtime for parameters and return values
- ✅ Descriptive TypeErrors are thrown for mismatches

**Example:**
```php
<?php
function add(int $a, int $b): int {
    return $a + $b;
}

echo add(5, 10);        // Output: 15
echo add("hello", "world"); // TypeError: Expected int, got string
```

**Supported validations:**
- Simple types: int, string, float, bool, array, object, callable, iterable, mixed
- Nullable types: ?int, ?string, etc.
- Union types: int|string, int|float|null
- Class types: User, Exception, etc.
- Return types: void, never, static

### Best Practices

- **Use type hints** for runtime safety and better code documentation
- **Prefer specific types** over `mixed` when possible
- **Use nullable types** (`?int`) instead of union with null when accepting a single type or null
- **Handle type errors** gracefully with try/catch when needed
- **Test edge cases** to ensure your type hints match actual usage

## Declare Directive - Strict Types (PHP 7.0)

The `declare` directive controls code execution aspects. The most important use is `declare(strict_types=1)` which enables strict type checking for function calls.

### What is Strict Types Mode?

By default, PHP uses **coercive type checking** - it attempts to convert values to match type hints:
- `"42"` (string) is automatically converted to `42` (int)
- `1.9` (float) is automatically converted to `1` (int)

With `declare(strict_types=1)`, PHP uses **strict type checking** - type mismatches are rejected:
- Passing a string where int is expected throws a TypeError
- Passing a float where int is expected throws a TypeError (except: int can widen to float)

### Basic Syntax

```php
<?php
declare(strict_types=1);

function add(int $a, int $b): int {
    return $a + $b;
}

echo add(1, 2);      // OK: 3
echo add(1.5, 2);    // TypeError: must be of type int, float given
echo add("1", 2);    // TypeError: must be of type int, string given
```

### Without Strict Types (Default Coercive Mode)

```php
<?php
function add(int $a, int $b): int {
    return $a + $b;
}

echo add("1", "2");  // OK: 3 (strings coerced to integers)
echo add(1.9, 2.1);  // OK: 3 (floats coerced to integers: 1 + 2)
```

### File-Scope Declaration

`declare(strict_types=1)` must be the **first statement** in a file (except for an opening PHP tag):

```php
<?php
declare(strict_types=1);  // Must be first!

function test(int $x): void {
    echo $x;
}

test(123);     // OK
test("123");   // TypeError
```

**Invalid positions:**
```php
<?php
echo "hello";
declare(strict_types=1);  // Error: must be first statement
```

### Block-Scoped Declaration

You can also use `declare` with a block to apply strict types to specific code:

```php
<?php
function outer(int $n) {
    return $n;
}

declare(strict_types=1) {
    function inner(int $n) {
        return $n;
    }
    // inner() uses strict types
}

// outer() uses coercive types
echo outer("42");  // OK: 42 (coerced)
```

**Alternative syntax:**
```php
<?php
declare(strict_types=1):
    // Code here
enddeclare;
```

### Type Widening Exception

Even in strict mode, **int can be passed where float is expected** (widening is allowed):

```php
<?php
declare(strict_types=1);

function half(float $n): float {
    return $n / 2;
}

echo half(10);    // OK: 5 (int widened to float)
echo half(10.0);  // OK: 5
echo half("10");  // TypeError: string cannot be passed as float
```

### Caller vs Callee

**Important:** The strict_types mode is determined by the **calling file**, not the function definition file.

If file A has `declare(strict_types=1)` and calls a function in file B (without strict_types), the call uses **strict checking** because it's happening in file A.

### Multiple Directives

You can declare multiple directives at once:

```php
<?php
declare(strict_types=1, encoding='UTF-8');
```

**Supported directives:**
- `strict_types` - Enable/disable strict type checking (0 or 1)
- `encoding` - Character encoding (mostly ignored in modern PHP)
- `ticks` - For `register_tick_function()` (advanced, rarely used)

### Validation Rules

**Strict mode type matching:**

| Hint Type | Value Type | Allowed? |
|-----------|------------|----------|
| `int` | `int` | ✅ Yes |
| `int` | `float` | ❌ No |
| `int` | `string` | ❌ No |
| `float` | `int` | ✅ Yes (widening) |
| `float` | `float` | ✅ Yes |
| `float` | `string` | ❌ No |
| `string` | `string` | ✅ Yes |
| `string` | `int` | ❌ No |
| `bool` | `bool` | ✅ Yes |
| `bool` | `int` | ❌ No |
| `array` | `array` | ✅ Yes |
| `mixed` | any | ✅ Yes |

### Use Cases

**When to use strict_types:**
- ✅ New projects where type safety is a priority
- ✅ Library code that needs predictable behavior
- ✅ Code that works with sensitive data (financial, security)
- ✅ When you want to catch type bugs early

**When not to use strict_types:**
- ⚠️ Legacy codebases that rely on type coercion
- ⚠️ Code that processes user input (strings) as numbers
- ⚠️ Quick scripts where flexibility is more important than strictness

### Examples

**Strict validation for parameters:**
```php
<?php
declare(strict_types=1);

function processUser(string $name, int $age, bool $active): void {
    echo "$name is $age years old";
    echo $active ? " (active)" : " (inactive)";
}

processUser("Alice", 30, true);  // OK
processUser("Bob", "25", true);  // TypeError: age must be int
processUser("Carol", 40, 1);     // TypeError: active must be bool
```

**Strict validation for return types:**
```php
<?php
declare(strict_types=1);

function getCount(): int {
    return 42;     // OK
}

function getBadCount(): int {
    return "42";   // TypeError: return value must be int, string given
}
```

**Mixed strict and coercive:**
```php
<?php
// File: strict.php
declare(strict_types=1);

require 'coercive.php';

// This call uses strict checking (caller's mode)
coerciveFunction(10);    // OK
coerciveFunction("10");  // TypeError

// File: coercive.php
function coerciveFunction(int $x) {
    echo $x;
}
```

### Key Points

- `declare(strict_types=1)` affects **parameter** and **return type** validation
- It must be the **first statement** in the file
- It's determined by the **calling file**, not the function definition
- **int → float widening** is allowed even in strict mode
- **No narrowing** is allowed (float → int, string → int, etc.)
- Applies to **user-defined functions** and **built-in functions** with type hints
- Cannot be changed at runtime (compile-time directive)

