//! Array built-in functions (re-export module)
//!
//! This module re-exports array functions from the split modules:
//! - array_basic: Basic array access functions
//! - array_search: Search and lookup functions
//! - array_manipulation: Array manipulation functions
//! - array_callbacks: Callback-based array functions

pub use super::array_basic::{
    array_first, array_keys, array_last, array_pop, array_push, array_shift, array_slice,
    array_unshift, array_values, count,
};

pub use super::array_callbacks::{array_filter, array_map, array_reduce, array_sum};

pub use super::array_manipulation::{array_merge, array_reverse, array_unique, range};

pub use super::array_search::{array_key_exists, array_search, in_array};
