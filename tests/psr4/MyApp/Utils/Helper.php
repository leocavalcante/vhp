<?php
// PSR-4 test: MyApp\Utils\Helper class

namespace MyApp\Utils;

class Helper {
    public string $text;

    public function __construct(string $text) {
        $this->text = $text;
    }

    public static function capitalize(string $text): string {
        return strtoupper($text);
    }

    public function getText(): string {
        return $this->text;
    }
}
