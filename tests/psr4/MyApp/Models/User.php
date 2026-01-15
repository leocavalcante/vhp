<?php
// PSR-4 test: MyApp\Models\User class

namespace MyApp\Models;

class User {
    public string $name;
    public string $email;

    public function __construct(string $name, string $email) {
        $this->name = $name;
        $this->email = $email;
    }

    public function getGreeting(): string {
        return "Hello, I am " . $this->name . " (" . $this->email . ")";
    }

    public static function createGuest(): User {
        return new User("Guest", "guest@example.com");
    }
}
