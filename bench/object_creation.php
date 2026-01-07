<?php
// Object instantiation benchmark
class SimpleObject {
    public $value;

    public function __construct($v) {
        $this->value = $v;
    }
}

for ($i = 0; $i < 1000; $i++) {
    $obj = new SimpleObject($i);
}

echo "1000\n";
