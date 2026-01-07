<?php
// Loop performance benchmark

$sum = 0;

// Nested loops
for ($i = 0; $i < 100; $i++) {
    for ($j = 0; $j < 100; $j++) {
        $sum += $i * $j;
    }
}

// While loop
$count = 0;
while ($count < 10000) {
    $count++;
}

// Foreach with array
$arr = range(1, 1000);
$total = 0;
foreach ($arr as $val) {
    $total += $val;
}

echo $sum . "\n";
echo $count . "\n";
echo $total . "\n";
