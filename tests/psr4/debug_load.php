<?php
// Debug PSR-4 loading
spl_autoload_register_psr4('MyApp\\', '/home/leo/projects/vhp/tests/psr4/MyApp/');
$loaded = load_psr4_class('MyApp\\Models\\User');
echo "Loaded: " . ($loaded ? "OK" : "FAIL") . "\n";

// Try to create an instance
$user = new MyApp\Models\User("Alice", "alice@example.com");
echo $user->getGreeting() . "\n";

