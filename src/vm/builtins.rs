//! Built-in function bridge for the VM
//!
//! This module provides a bridge between the VM and the interpreter's
//! built-in function implementations.

use crate::interpreter::builtins;
use crate::interpreter::Value;
use std::io::Write;

/// List of all built-in function names (lowercase for case-insensitive matching)
pub const BUILTIN_FUNCTIONS: &[&str] = &[
    // String functions
    "strlen",
    "substr",
    "strtoupper",
    "strtolower",
    "trim",
    "ltrim",
    "rtrim",
    "str_repeat",
    "str_replace",
    "strpos",
    "str_contains",
    "str_starts_with",
    "str_ends_with",
    "ucfirst",
    "lcfirst",
    "ucwords",
    "strrev",
    "str_pad",
    "explode",
    "implode",
    "join",
    "sprintf",
    "chr",
    "ord",
    // Math functions
    "abs",
    "ceil",
    "floor",
    "round",
    "max",
    "min",
    "pow",
    "sqrt",
    "rand",
    "mt_rand",
    // Type functions
    "intval",
    "floatval",
    "doubleval",
    "strval",
    "boolval",
    "gettype",
    "is_null",
    "is_bool",
    "is_int",
    "is_integer",
    "is_long",
    "is_float",
    "is_double",
    "is_real",
    "is_string",
    "is_array",
    "is_numeric",
    "isset",
    "empty",
    "unset",
    // Array functions
    "count",
    "sizeof",
    "array_push",
    "array_pop",
    "array_shift",
    "array_unshift",
    "array_keys",
    "array_values",
    "in_array",
    "array_search",
    "array_reverse",
    "array_merge",
    "array_key_exists",
    "range",
    "array_first",
    "array_last",
    // Output functions (handled separately since they need writer)
    "print",
    "var_dump",
    "print_r",
    "printf",
    // Reflection functions (handled in VM)
    "get_class_attributes",
    "get_property_attributes",
    "get_method_attributes",
    "get_method_parameter_attributes",
    "get_function_attributes",
    "get_parameter_attributes",
    "get_interface_attributes",
    "get_trait_attributes",
];

/// Check if a function name is a built-in function
pub fn is_builtin(name: &str) -> bool {
    let lower = name.to_lowercase();
    BUILTIN_FUNCTIONS.contains(&lower.as_str())
}

/// Call a built-in function with the given arguments
/// Returns the result value or an error message
pub fn call_builtin<W: Write>(name: &str, args: &[Value], output: &mut W) -> Result<Value, String> {
    let lower_name = name.to_lowercase();
    match lower_name.as_str() {
        // String functions
        "strlen" => builtins::string::strlen(args),
        "substr" => builtins::string::substr(args),
        "strtoupper" => builtins::string::strtoupper(args),
        "strtolower" => builtins::string::strtolower(args),
        "trim" => builtins::string::trim(args),
        "ltrim" => builtins::string::ltrim(args),
        "rtrim" => builtins::string::rtrim(args),
        "str_repeat" => builtins::string::str_repeat(args),
        "str_replace" => builtins::string::str_replace(args),
        "strpos" => builtins::string::strpos(args),
        "str_contains" => builtins::string::str_contains(args),
        "str_starts_with" => builtins::string::str_starts_with(args),
        "str_ends_with" => builtins::string::str_ends_with(args),
        "ucfirst" => builtins::string::ucfirst(args),
        "lcfirst" => builtins::string::lcfirst(args),
        "ucwords" => builtins::string::ucwords(args),
        "strrev" => builtins::string::strrev(args),
        "str_pad" => builtins::string::str_pad(args),
        "explode" => builtins::string::explode(args),
        "implode" | "join" => builtins::string::implode(args),
        "sprintf" => builtins::string::sprintf(args),
        "chr" => builtins::string::chr(args),
        "ord" => builtins::string::ord(args),

        // Math functions
        "abs" => builtins::math::abs(args),
        "ceil" => builtins::math::ceil(args),
        "floor" => builtins::math::floor(args),
        "round" => builtins::math::round(args),
        "max" => builtins::math::max(args),
        "min" => builtins::math::min(args),
        "pow" => builtins::math::pow(args),
        "sqrt" => builtins::math::sqrt(args),
        "rand" | "mt_rand" => builtins::math::rand(args),

        // Type functions
        "intval" => builtins::types::intval(args),
        "floatval" | "doubleval" => builtins::types::floatval(args),
        "strval" => builtins::types::strval(args),
        "boolval" => builtins::types::boolval(args),
        "gettype" => builtins::types::gettype(args),
        "is_null" => builtins::types::is_null(args),
        "is_bool" => builtins::types::is_bool(args),
        "is_int" | "is_integer" | "is_long" => builtins::types::is_int(args),
        "is_float" | "is_double" | "is_real" => builtins::types::is_float(args),
        "is_string" => builtins::types::is_string(args),
        "is_array" => builtins::types::is_array(args),
        "is_numeric" => builtins::types::is_numeric(args),
        "isset" => builtins::types::isset(args),
        "empty" => builtins::types::empty(args),
        "unset" => builtins::types::unset(args),

        // Array functions
        "count" | "sizeof" => builtins::array::count(args),
        "array_push" => builtins::array::array_push(args),
        "array_pop" => builtins::array::array_pop(args),
        "array_shift" => builtins::array::array_shift(args),
        "array_unshift" => builtins::array::array_unshift(args),
        "array_keys" => builtins::array::array_keys(args),
        "array_values" => builtins::array::array_values(args),
        "in_array" => builtins::array::in_array(args),
        "array_search" => builtins::array::array_search(args),
        "array_reverse" => builtins::array::array_reverse(args),
        "array_merge" => builtins::array::array_merge(args),
        "array_key_exists" => builtins::array::array_key_exists(args),
        "range" => builtins::array::range(args),
        "array_first" => builtins::array::array_first(args),
        "array_last" => builtins::array::array_last(args),

        // Output functions (need writer)
        "print" => builtins::output::print(output, args),
        "var_dump" => builtins::output::var_dump(output, args),
        "print_r" => builtins::output::print_r(output, args),
        "printf" => builtins::output::printf(output, args),

        _ => Err(format!("Unknown built-in function: {}", name)),
    }
}
