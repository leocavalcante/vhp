<?php
// Test class for require_once() functionality

class OnceCounter {
    private static int $count = 0;

    public function __construct() {
        self::$count++;
    }

    public static function getCount(): int {
        return self::$count;
    }
}
