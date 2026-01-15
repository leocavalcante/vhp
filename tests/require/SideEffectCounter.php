<?php
class SideEffectCounter {
    public static int $sideEffects = 0;

    public function __construct() {
        self::$sideEffects++;
    }

    public static function getSideEffects(): int {
        return self::$sideEffects;
    }
}

SideEffectCounter::$sideEffects++;
return SideEffectCounter::$sideEffects;
