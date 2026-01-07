<?php
// String operations benchmark

$text = "Hello World! ";
$result = "";

// String concatenation
for ($i = 0; $i < 1000; $i++) {
    $result .= $text;
}

// String manipulation
$upper = strtoupper($result);
$lower = strtolower($upper);
$length = strlen($lower);

// String searching
$pos = strpos($lower, "world");

// String replacement
$replaced = str_replace("hello", "hi", $lower);

// String splitting
$parts = explode(" ", $text);
$joined = implode("-", $parts);

echo $length . "\n";
echo $pos . "\n";
