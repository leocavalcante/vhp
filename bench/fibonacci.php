<?php
// Recursive Fibonacci benchmark
function fibonacci($n) {
    if ($n <= 1) {
        return $n;
    }
    return fibonacci($n - 1) + fibonacci($n - 2);
}

$result = fibonacci(30);
echo $result . "\n";
