<?php
// PSR-4 autoloading test
// PSR-4: prefix "MyApp\" maps to base_dir/MyApp/, so MyApp\Models\User -> base_dir/MyApp/Models/User.php

// Register PSR-4 namespace prefix (prefix ends with \)
spl_autoload_register_psr4('MyApp\\', '/home/leo/projects/vhp/tests/psr4/MyApp/');

// Test load_psr4_class directly
$loaded = load_psr4_class('MyApp\\Models\\User');
echo "Direct load: " . ($loaded ? "OK" : "FAILED") . "\n";

// Check if class is now available
$user = new MyApp\Models\User("Alice", "alice@example.com");
echo $user->getGreeting() . "\n";


