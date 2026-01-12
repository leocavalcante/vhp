//! Built-in functions module

pub mod array;
pub mod fileio;
pub mod json;
pub mod math;
pub mod output;
pub mod string;
pub mod types;

pub use json::{json_decode, json_encode};

#[allow(unused_imports)]
pub use array::{
    array_filter, array_first, array_key_exists, array_keys, array_last, array_map, array_merge,
    array_pop, array_push, array_reduce, array_reverse, array_search, array_shift, array_sum,
    array_unique, array_unshift, array_values, count, in_array, range,
};

pub use math::{
    abs, ceil, cos, exp, floor, log, log10, max, min, pi, pow, rand, round, sin, sqrt, tan,
};

pub use fileio::{
    file_exists, file_get_contents, file_put_contents, filemtime, filesize, is_dir, is_file,
    is_readable, is_writable, unlink,
};
