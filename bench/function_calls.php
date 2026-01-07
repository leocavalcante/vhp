<?php
// Function call overhead benchmark
function add($a, $b) {
    return $a + $b;
}

function multiply($a, $b) {
    return $a * $b;
}

function compute($x, $y, $z) {
    return add($x, $y) + multiply($y, $z);
}

$result = 0;
for ($i = 0; $i < 10000; $i++) {
    $result = compute($i, $i + 1, $i + 2);
}

echo $result . "\n";
