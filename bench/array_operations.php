<?php
// Array operations benchmark

// Create large array
$arr = range(1, 10000);

// Array operations
$sum = 0;
foreach ($arr as $val) {
    $sum += $val;
}

// Array manipulation
array_push($arr, 10001);
array_pop($arr);
array_unshift($arr, 0);
array_shift($arr);

// Array search
$found = in_array(5000, $arr);

// Array merge
$arr2 = range(10001, 15000);
$merged = array_merge($arr, $arr2);

echo $sum . "\n";
echo count($merged) . "\n";
