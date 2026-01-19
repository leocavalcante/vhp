# Plan: Feature Implementation and Code Quality
Created: 2026-01-16T20:36:54Z
Cycle: 2

## Context
VHP is a PHP 8.x compatible bytecode VM with 566 tests (557 passing, 3 failed, 6 errors). Critical issues identified:
- Missing time functions (time, mktime, strtotime) blocking 6 tests
- Missing Exception backtrace methods (getTrace, getTraceAsString) blocking 2 tests
- interface_exists() always returns false, blocking Traversable test
- vm/mod.rs exceeds 500-line hard limit (508 lines)
- 35 unwrap() calls remaining (target: <30)

## Tasks
- [x] Task 1: Implement time(), mktime(), strtotime() built-in functions in src/runtime/builtins/datetime_timestamp.rs
- [x] Task 2: Implement Exception::getTrace() and getTraceAsString() methods in src/runtime/value/object_instance.rs
- [ ] Task 3: Implement interface_exists() to check against actual interface registry in src/runtime/builtins/type_extra.rs
- [ ] Task 4: Register SPL built-in interfaces during VM initialization in src/vm/mod.rs and src/vm/spl_interfaces.rs
- [ ] Task 5: Refactor src/vm/mod.rs (508 lines) by extracting opcode dispatcher to vm/execution_dispatcher.rs
- [ ] Task 6: Reduce unsafe unwrap() calls from 35 to <30 by replacing with proper error handling
- [ ] Task 7: Add integration tests for time functions and exception backtrace methods
- [ ] Task 8: Verify all test failures resolved and achieve 100% test pass rate

## Notes
Priority: Fix failing tests first (high impact), then file size violation, then code quality improvements.

Dependencies:
- Task 3 (interface_exists) depends on Task 4 (interface registration)
- Task 2 (Exception backtrace) requires tracking call frames during execution
- Time functions need Rust chrono library or custom date parsing

Metrics:
- Current test coverage: 557/566 passing (98.4%)
- Files >500 lines: 1 (vm/mod.rs: 508)
- Unsafe unwrap count: 35


