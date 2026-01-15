<?php
spl_autoload_register_psr4('MyApp\\', '/home/leo/projects/vhp/tests/psr4/MyApp/');
load_psr4_class('MyApp\\Models\\User');
$user = new MyApp\Models\User('Test', 'test@example.com');
echo $user->getGreeting();
