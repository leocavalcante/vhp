<?php
// PSR-4 autoloading test - debug version

// First, check if PSR-4 registration works
// PSR-4: prefix "MyApp\" maps to base_dir, so MyApp\Models\User -> base_dir/Models/User.php
// But we want MyApp\Models\User -> base_dir/MyApp/Models/User.php
// So the prefix should be "" (empty) and base_dir should include MyApp
$result = spl_autoload_register_psr4('MyApp\\', '/home/leo/projects/vhp/tests/psr4/MyApp/');
echo "PSR-4 registration: " . ($result ? "OK" : "FAILED") . "\n";

// Check registered mappings
$mappings = spl_autoload_registered_psr4();
echo "Registered mappings: " . count($mappings) . "\n";

// Test load_psr4_class directly
$loaded = load_psr4_class('MyApp\\Models\\User');
echo "Direct load: " . ($loaded ? "OK" : "FAILED") . "\n";

// Check if class is now available
$user = new MyApp\Models\User("Alice", "alice@example.com");
echo $user->getGreeting();




