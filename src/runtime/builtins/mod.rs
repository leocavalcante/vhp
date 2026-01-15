//! Built-in functions module

pub mod array;
pub mod array_extra;
pub mod fileio;
pub mod json;
pub mod math;
pub mod output;
pub mod spl;
pub mod string;
pub mod string_extra;
pub mod type_extra;
pub mod types;

pub use json::{json_decode, json_encode};

#[allow(unused_imports)]
pub use array::{
    array_filter, array_first, array_key_exists, array_keys, array_last, array_map, array_merge,
    array_pop, array_push, array_reduce, array_reverse, array_search, array_shift, array_slice,
    array_sum, array_unique, array_unshift, array_values, count, in_array, range,
};

#[allow(unused_imports)]
pub use array_extra::{
    array_chunk, array_combine, array_fill, array_fill_keys, array_pad, array_splice,
};

#[allow(unused_imports)]
pub use string::{
    chr, explode, implode, lcfirst, ord, sprintf, str_contains, str_ends_with, str_pad, str_repeat,
    str_replace, str_starts_with, strlen, strpos, strrev, strtolower, strtoupper, substr, trim,
    ucfirst, ucwords,
};

#[allow(unused_imports)]
pub use string_extra::{
    bin2hex, hex2bin, htmlentities, htmlspecialchars, levenshtein, md5, nl2br, number_format, sha1,
    similar_text, strtr,
};

#[allow(unused_imports)]
pub use fileio::{
    file_exists, file_get_contents, file_put_contents, filemtime, filesize, is_dir, is_file,
    is_readable, is_writable, unlink,
};

#[allow(unused_imports)]
pub use spl::{
    get_include_path, set_include_path, spl_autoload_functions, spl_autoload_register,
    spl_autoload_register_psr4, spl_autoload_registered_psr4, spl_autoload_unregister,
};

#[allow(unused_imports)]
pub use types::{
    boolval, empty, floatval, gettype, intval, is_array, is_bool, is_callable, is_float, is_int,
    is_null, is_numeric, is_string, isset, strval, unset,
};

#[allow(unused_imports)]
pub use type_extra::{
    class_alias, class_exists, func_get_arg, func_get_args, func_num_args, get_class,
    get_class_methods, get_class_vars, get_declared_classes, get_declared_interfaces,
    get_declared_traits, get_defined_functions, get_object_vars, get_parent_class,
    interface_exists, is_a, is_subclass_of, method_exists, property_exists, trait_exists,
};
