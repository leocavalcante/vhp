---
layout: default
title: Usage
nav_order: 4
---

# Usage

## Running PHP Files

```bash
vhp script.php
```

## Running Inline Code

Use the `-r` flag to execute code directly:

```bash
vhp -r 'echo "Hello, World!";'
```

## Running Tests

VHP includes a built-in test runner for `.vhpt` test files:

```bash
# Run all tests in the default tests/ directory
vhp test

# Verbose output (shows each test name)
vhp test -v

# Run tests from a custom directory
vhp test mydir

# Run a single test file
vhp test path/to/test.vhpt
```

## Command Reference

```
USAGE:
    vhp [OPTIONS] [FILE]
    vhp [SUBCOMMAND]

ARGUMENTS:
    [FILE]    PHP file to execute

OPTIONS:
    -r <CODE>    Run inline PHP code
    -h, --help   Print help information

SUBCOMMANDS:
    test         Run test suite
```

## Examples

### Execute a Script

```bash
$ echo '<?php echo "Hello from VHP!";' > hello.php
$ vhp hello.php
Hello from VHP!
```

### Quick Calculations

```bash
$ vhp -r 'echo 2 ** 10;'
1024
```

### Test String Functions

```bash
$ vhp -r 'echo strtoupper("hello world");'
HELLO WORLD
```
