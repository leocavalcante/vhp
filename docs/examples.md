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

## Fibers (Cooperative Multitasking)

```php
<?php
function worker($name) {
    echo "Worker $name starting\n";
    
    for ($i = 1; $i <= 3; $i++) {
        echo "Worker $name: Step $i\n";
        
        // Suspend execution and pass control back
        $data = Fiber::suspend("step_$i");
        
        if ($data) {
            echo "Worker $name received: $data\n";
        }
    }
    
    return "Worker $name completed";
}

$fiber = new Fiber('worker');

// Start the fiber
$result = $fiber->start('Alice');
echo "Fiber returned: $result\n";

// Resume multiple times
$result = $fiber->resume('resume_1');  
echo "Fiber returned: $result\n";

$result = $fiber->resume('resume_2');
echo "Fiber returned: $result\n"; 

$result = $fiber->resume('resume_3');
echo "Final result: $result\n";

// Check final state
echo "Terminated: " . ($fiber->isTerminated() ? "Yes" : "No") . "\n";
```

## Exception Handling

```php
<?php
class ValidationException extends Exception {}
class DatabaseException extends Exception {}

function processUser($username, $age) {
    // Validate input
    if (empty($username)) {
        throw new ValidationException("Username cannot be empty");
    }
    
    if ($age < 0 || $age > 150) {
        throw new ValidationException("Invalid age: " . $age);
    }
    
    // Simulate database operation
    if ($username === "admin") {
        throw new DatabaseException("Cannot modify admin user");
    }
    
    return "User $username (age $age) processed successfully";
}

// Example 1: Catch specific exception
try {
    echo processUser("", 25);
} catch (ValidationException $e) {
    echo "Validation Error: " . $e->getMessage() . "\n";
} catch (DatabaseException $e) {
    echo "Database Error: " . $e->getMessage() . "\n";
}
// Output: Validation Error: Username cannot be empty

// Example 2: Multi-catch (PHP 7.1+)
try {
    echo processUser("admin", 30);
} catch (ValidationException | DatabaseException $e) {
    echo "Error: " . $e->getMessage() . "\n";
}
// Output: Error: Cannot modify admin user

// Example 3: Finally block always executes
function attemptOperation() {
    try {
        echo "Opening connection\n";
        throw new Exception("Connection failed");
    } catch (Exception $e) {
        echo "Error: " . $e->getMessage() . "\n";
    } finally {
        echo "Closing connection\n";
    }
}

attemptOperation();
// Output:
// Opening connection
// Error: Connection failed
// Closing connection

// Example 4: Throw as expression (PHP 8.0+)
function getConfig($key) {
    $config = ["timeout" => 30, "retries" => 3];
    return $config[$key] ?? throw new Exception("Config key '$key' not found");
}

try {
    echo "Timeout: " . getConfig("timeout") . "\n";
    echo getConfig("invalid");
} catch (Exception $e) {
    echo "Error: " . $e->getMessage();
}
// Output:
// Timeout: 30
// Error: Config key 'invalid' not found
```

## DNF Types (PHP 8.2)

DNF (Disjunctive Normal Form) types allow complex type declarations that combine union and intersection types.

```php
<?php
// Example 1: Basic DNF type - (A&B)|C
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

// Example 2: Multiple intersection groups - (A&B)|(C&D)
interface Iterator {}
interface Countable {}
interface ArrayAccess {}
interface Traversable {}

class CountableIterator implements Iterator, Countable {}
class TraversableArray implements ArrayAccess, Traversable {}

function handle((Iterator&Countable)|(ArrayAccess&Traversable) $collection): string {
    return get_class($collection);
}

echo handle(new CountableIterator()) . "\n";   // CountableIterator
echo handle(new TraversableArray()) . "\n";    // TraversableArray

// Example 3: DNF with return types
interface Readable {
    public function read();
}

interface Writable {
    public function write($data);
}

interface Stream {
    public function stream();
}

class File implements Readable, Writable {
    public function read() { return "data"; }
    public function write($data) {}
}

function openResource(): (Readable&Writable)|Stream {
    return new File();
}

$resource = openResource();
echo $resource->read();  // data

// Example 4: Type validation with DNF
interface A {}
interface B {}
interface C {}

class AB implements A, B {}
class OnlyA implements A {}

function test((A&B)|C $obj): void {
    echo "Valid type\n";
}

test(new AB());      // Valid type
// test(new OnlyA());  // TypeError: Expected type (A&B)|C
// OnlyA only implements A, not both A and B, and not C

// Example 5: Complex business logic with DNF
interface PaymentMethod {
    public function charge($amount);
}

interface Refundable {
    public function refund($amount);
}

interface Recurring {
    public function setupRecurring($interval);
}

class CreditCard implements PaymentMethod, Refundable {
    public function charge($amount) { return "Charged $amount"; }
    public function refund($amount) { return "Refunded $amount"; }
}

class Subscription implements Recurring {
    public function setupRecurring($interval) { return "Recurring: $interval"; }
}

function processPayment((PaymentMethod&Refundable)|Recurring $method): string {
    if ($method instanceof PaymentMethod) {
        return $method->charge(100);
    } elseif ($method instanceof Recurring) {
        return $method->setupRecurring("monthly");
    }
    return "Unknown";
}

echo processPayment(new CreditCard()) . "\n";    // Charged 100
echo processPayment(new Subscription()) . "\n";  // Recurring: monthly
```

## Declare Strict Types (PHP 7.0)

The `declare(strict_types=1)` directive enables strict type checking, preventing automatic type coercion.

```php
<?php
// Example 1: Strict types enabled
declare(strict_types=1);

function add(int $a, int $b): int {
    return $a + $b;
}

echo add(5, 10);     // OK: 15
echo add(5.0, 10);   // TypeError: must be of type int, float given
echo add("5", 10);   // TypeError: must be of type int, string given

// Example 2: Without strict types (default coercive mode)
function multiply(int $a, int $b): int {
    return $a * $b;
}

echo multiply("5", "10");  // OK: 50 (strings coerced to integers)
echo multiply(5.9, 2.1);   // OK: 10 (floats truncated: 5 * 2)

// Example 3: Type widening exception - int to float is allowed
declare(strict_types=1);

function divide(float $a, float $b): float {
    return $a / $b;
}

echo divide(10, 2);      // OK: 5 (int widened to float)
echo divide(10.0, 2.0);  // OK: 5
echo divide("10", 2);    // TypeError: string cannot be passed as float

// Example 4: Strict validation for multiple types
declare(strict_types=1);

function processUser(string $name, int $age, bool $active): string {
    $status = $active ? "active" : "inactive";
    return "$name is $age years old ($status)";
}

echo processUser("Alice", 30, true) . "\n";  // OK
echo processUser("Bob", "25", true);         // TypeError: age must be int

// Example 5: Block-scoped strict types
function coercive(int $x): int {
    return $x;
}

declare(strict_types=1) {
    function strict(int $x): int {
        return $x;
    }

    // strict() enforces strict types
    // echo strict("10");  // TypeError
}

// coercive() uses default type coercion
echo coercive("10");  // OK: 10
```

## Property Hooks (PHP 8.4)

Property hooks provide a clean way to add custom logic to property access without explicit getter/setter methods.

```php
<?php
// Example 1: Computed property with get hook
class Circle {
    public float $radius = 5.0;

    public float $diameter {
        get => $this->radius * 2;
    }

    public float $area {
        get => 3.14159 * $this->radius ** 2;
    }
}

$c = new Circle();
echo "Radius: " . $c->radius . "\n";      // 5
echo "Diameter: " . $c->diameter . "\n";  // 10
echo "Area: " . $c->area . "\n";          // 78.53975

$c->radius = 10.0;
echo "New diameter: " . $c->diameter . "\n";  // 20 (automatically updated)

// Example 2: Temperature converter with get and set hooks
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
echo "Fahrenheit: " . $t->fahrenheit . "\n";  // 212
echo "Celsius: " . $t->celsius . "\n";        // 100

// Example 3: Validation and logging with set hook
class User {
    private string $rawEmail = "";

    public string $email {
        get => $this->rawEmail;
        set {
            if (!str_contains($value, "@")) {
                throw new Exception("Invalid email format");
            }
            $this->rawEmail = strtolower($value);
            echo "Email set to: " . $this->rawEmail . "\n";
        }
    }
}

$user = new User();
$user->email = "USER@EXAMPLE.COM";  // Email set to: user@example.com
echo "Current: " . $user->email . "\n";     // user@example.com

// Example 4: Read-only computed property
class Rectangle {
    public float $width = 10.0;
    public float $height = 5.0;

    public float $area {
        get => $this->width * $this->height;
    }
}

$rect = new Rectangle();
echo "Area: " . $rect->area . "\n";  // 50
// $rect->area = 100;  // Error: Cannot set property with only get hook

// Example 5: Side-effects with set hooks
class EventLogger {
    private array $events = [];

    public string $event {
        set {
            $this->events[] = [
                "message" => $value,
                "timestamp" => date("Y-m-d H:i:s")
            ];
            echo "Logged event: " . $value . "\n";
        }
    }
}

$logger = new EventLogger();
$logger->event = "User logged in";   // Logged event: User logged in
$logger->event = "Data processed";   // Logged event: Data processed
```

## Static Properties and Late Static Binding (PHP 5.0/5.3)

Static properties are class-level variables shared across all instances. Late static binding allows proper inheritance behavior with the `static::` keyword.

```php
<?php
// Example 1: Basic static property counter
class PageView {
    public static $count = 0;

    public static function recordView() {
        self::$count++;
        echo "Total views: " . self::$count . "\n";
    }
}

PageView::recordView();  // Total views: 1
PageView::recordView();  // Total views: 2
PageView::recordView();  // Total views: 3

// Example 2: Configuration management with static properties
class Config {
    private static $settings = [
        "debug" => false,
        "cache" => true
    ];

    public static function get($key) {
        return self::$settings[$key] ?? null;
    }

    public static function set($key, $value) {
        self::$settings[$key] = $value;
    }
}

echo "Debug mode: " . (Config::get("debug") ? "on" : "off") . "\n";  // off
Config::set("debug", true);
echo "Debug mode: " . (Config::get("debug") ? "on" : "off") . "\n";  // on

// Example 3: Late static binding with static::
class Animal {
    protected static $species = "Unknown";

    public static function getSpecies() {
        // static:: refers to the called class (late binding)
        return static::$species;
    }

    public static function identify() {
        echo "I am a " . static::getSpecies() . "\n";
    }
}

class Dog extends Animal {
    protected static $species = "Canine";
}

class Cat extends Animal {
    protected static $species = "Feline";
}

Animal::identify();  // I am a Unknown
Dog::identify();     // I am a Canine
Cat::identify();     // I am a Feline

// Example 4: Difference between self:: and static::
class Counter {
    protected static $name = "Base Counter";

    public static function showWithSelf() {
        return "self:: -> " . self::$name;
    }

    public static function showWithStatic() {
        return "static:: -> " . static::$name;
    }
}

class SpecialCounter extends Counter {
    protected static $name = "Special Counter";
}

echo Counter::showWithSelf() . "\n";         // self:: -> Base Counter
echo Counter::showWithStatic() . "\n";       // static:: -> Base Counter

echo SpecialCounter::showWithSelf() . "\n";  // self:: -> Base Counter (wrong!)
echo SpecialCounter::showWithStatic() . "\n"; // static:: -> Special Counter (correct!)

// Example 5: Singleton pattern with static properties
class Database {
    private static $instance = null;
    private $connectionString;

    private function __construct($connStr) {
        $this->connectionString = $connStr;
        echo "Database connected to: " . $connStr . "\n";
    }

    public static function getInstance($connStr = "localhost:5432") {
        if (self::$instance === null) {
            self::$instance = new Database($connStr);
        }
        return self::$instance;
    }

    public function query($sql) {
        echo "Executing: " . $sql . "\n";
    }
}

$db1 = Database::getInstance("production.db");
$db1->query("SELECT * FROM users");

$db2 = Database::getInstance("another.db");  // Reuses existing instance
$db2->query("SELECT * FROM posts");

// Example 6: Static property with array operations
class Registry {
    public static $data = [];

    public static function register($key, $value) {
        self::$data[$key] = $value;
    }

    public static function get($key) {
        return self::$data[$key] ?? null;
    }
}

Registry::register("app_name", "VHP Framework");
Registry::register("version", "1.0.0");

echo "App: " . Registry::get("app_name") . "\n";  // App: VHP Framework
echo "Version: " . Registry::get("version") . "\n";  // Version: 1.0.0
```

## Asymmetric Visibility (PHP 8.4)

Asymmetric visibility allows public read access with restricted write access, eliminating the need for getter/setter boilerplate.

```php
<?php
// Example 1: Basic asymmetric visibility - public read, private write
class User {
    public private(set) string $id;
    public private(set) string $email;
    public string $name;  // Symmetric: public read/write

    public function __construct(string $id, string $email, string $name) {
        $this->id = $id;      // OK: write inside class
        $this->email = $email;
        $this->name = $name;
    }

    public function updateEmail(string $newEmail) {
        // Validation logic here
        $this->email = $newEmail;  // OK: write inside class
    }
}

$user = new User("123", "user@example.com", "Alice");
echo $user->id . "\n";     // OK: public read - 123
echo $user->email . "\n";  // OK: public read - user@example.com

$user->name = "Bob";       // OK: public write (symmetric visibility)
// $user->id = "456";      // Error: Cannot modify private property
// $user->email = "new@example.com";  // Error: Cannot modify private property

$user->updateEmail("new@example.com");  // OK: write through method
echo $user->email . "\n";  // new@example.com

// Example 2: Public read, protected write - inheritance pattern
class Counter {
    public protected(set) int $count = 0;
    public protected(set) int $maxValue = 100;

    public function getValue(): int {
        return $this->count;
    }
}

class ResettableCounter extends Counter {
    public function increment() {
        if ($this->count < $this->maxValue) {
            $this->count++;  // OK: protected write in subclass
        }
    }

    public function reset() {
        $this->count = 0;  // OK: protected write in subclass
    }
}

$counter = new ResettableCounter();
echo $counter->count . "\n";      // OK: public read - 0
echo $counter->maxValue . "\n";   // OK: public read - 100

$counter->increment();
$counter->increment();
echo $counter->count . "\n";      // OK: public read - 2

// $counter->count = 10;          // Error: Cannot modify protected property
// $counter->maxValue = 200;      // Error: Cannot modify protected property

$counter->reset();
echo $counter->count . "\n";      // OK: public read - 0

// Example 3: Immutable value objects
readonly class Point {
    public function __construct(
        public private(set) float $x,
        public private(set) float $y
    ) {}

    public function distanceFrom(Point $other): float {
        $dx = $this->x - $other->x;
        $dy = $this->y - $other->y;
        return sqrt($dx * $dx + $dy * $dy);
    }
}

$p1 = new Point(0.0, 0.0);
$p2 = new Point(3.0, 4.0);

echo "P1: (" . $p1->x . ", " . $p1->y . ")\n";  // P1: (0, 0)
echo "P2: (" . $p2->x . ", " . $p2->y . ")\n";  // P2: (3, 4)
echo "Distance: " . $p1->distanceFrom($p2) . "\n";  // Distance: 5

// $p1->x = 1.0;  // Error: Cannot modify private property
// $p1->y = 1.0;  // Error: Cannot modify private property

// Example 4: Static properties with asymmetric visibility
class AppConfig {
    public private(set) static string $environment = "development";
    public private(set) static bool $debug = true;
    public private(set) static array $settings = [];

    public static function initialize(string $env) {
        self::$environment = $env;  // OK: write inside class
        self::$debug = ($env === "development");
        self::$settings = [
            "timeout" => 30,
            "retries" => 3
        ];
    }

    public static function setSetting(string $key, $value) {
        self::$settings[$key] = $value;  // OK: write inside class
    }
}

// Public read access
echo "Environment: " . AppConfig::$environment . "\n";  // development
echo "Debug: " . (AppConfig::$debug ? "on" : "off") . "\n";  // on

// AppConfig::$environment = "production";  // Error: Cannot modify private property
// AppConfig::$debug = false;               // Error: Cannot modify private property

// Must use method to modify
AppConfig::initialize("production");
echo "Environment: " . AppConfig::$environment . "\n";  // production
echo "Debug: " . (AppConfig::$debug ? "on" : "off") . "\n";  // off

// Example 5: Protected read, private write
class Authenticator {
    protected private(set) string $token = "";
    protected private(set) bool $authenticated = false;

    private function setToken(string $token) {
        $this->token = $token;  // OK: private write inside class
        $this->authenticated = true;
    }

    public function login(string $username, string $password): bool {
        // Authentication logic here
        if ($username === "admin" && $password === "secret") {
            $this->setToken("token_" . uniqid());
            return true;
        }
        return false;
    }

    public function isAuthenticated(): bool {
        return $this->authenticated;
    }
}

class ExtendedAuthenticator extends Authenticator {
    public function getAuthStatus(): string {
        // OK: protected read in subclass
        if ($this->authenticated) {
            return "Authenticated with token: " . substr($this->token, 0, 10) . "...";
        }
        return "Not authenticated";
    }

    public function forceLogout() {
        // $this->authenticated = false;  // Error: Cannot modify private property
        // $this->token = "";              // Error: Cannot modify private property
        // Must use public interface
    }
}

$auth = new ExtendedAuthenticator();
echo $auth->getAuthStatus() . "\n";  // Not authenticated

if ($auth->login("admin", "secret")) {
    echo $auth->getAuthStatus() . "\n";  // Authenticated with token: token_...
}

// Example 6: Practical use case - Event sourcing
class Event {
    public private(set) string $id;
    public private(set) string $type;
    public private(set) array $data;
    public private(set) int $timestamp;

    public function __construct(string $type, array $data) {
        $this->id = uniqid("evt_", true);
        $this->type = $type;
        $this->data = $data;
        $this->timestamp = time();
    }

    public function toArray(): array {
        return [
            "id" => $this->id,
            "type" => $this->type,
            "data" => $this->data,
            "timestamp" => $this->timestamp
        ];
    }
}

$event = new Event("user.created", ["username" => "alice", "email" => "alice@example.com"]);

// Public read access to all properties
echo "Event ID: " . $event->id . "\n";
echo "Event Type: " . $event->type . "\n";
echo "Timestamp: " . $event->timestamp . "\n";

// Cannot modify immutable event properties
// $event->id = "evt_new";         // Error: Cannot modify private property
// $event->type = "user.updated";  // Error: Cannot modify private property
// $event->timestamp = time();     // Error: Cannot modify private property
```

