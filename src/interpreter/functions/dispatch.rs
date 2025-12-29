//! Built-in function dispatcher
//!
//! This module handles routing calls to built-in functions organized by category.
//! It dispatches to the appropriate builtin submodule based on function name.

use crate::interpreter::builtins;
use crate::interpreter::value::Value;
use crate::interpreter::Interpreter;
use std::io::Write;

impl<W: Write> Interpreter<W> {
    /// Dispatch a built-in function call by name
    pub(super) fn dispatch_builtin(
        &mut self,
        name: &str,
        arg_values: &[Value],
    ) -> Result<Value, String> {
        let lower_name = name.to_lowercase();
        match lower_name.as_str() {
            // String functions
            "strlen" => builtins::string::strlen(arg_values),
            "substr" => builtins::string::substr(arg_values),
            "strtoupper" => builtins::string::strtoupper(arg_values),
            "strtolower" => builtins::string::strtolower(arg_values),
            "trim" => builtins::string::trim(arg_values),
            "ltrim" => builtins::string::ltrim(arg_values),
            "rtrim" => builtins::string::rtrim(arg_values),
            "str_repeat" => builtins::string::str_repeat(arg_values),
            "str_replace" => builtins::string::str_replace(arg_values),
            "strpos" => builtins::string::strpos(arg_values),
            "str_contains" => builtins::string::str_contains(arg_values),
            "str_starts_with" => builtins::string::str_starts_with(arg_values),
            "str_ends_with" => builtins::string::str_ends_with(arg_values),
            "ucfirst" => builtins::string::ucfirst(arg_values),
            "lcfirst" => builtins::string::lcfirst(arg_values),
            "ucwords" => builtins::string::ucwords(arg_values),
            "strrev" => builtins::string::strrev(arg_values),
            "str_pad" => builtins::string::str_pad(arg_values),
            "explode" => builtins::string::explode(arg_values),
            "implode" | "join" => builtins::string::implode(arg_values),
            "sprintf" => builtins::string::sprintf(arg_values),
            "printf" => builtins::output::printf(&mut self.output, arg_values),
            "chr" => builtins::string::chr(arg_values),
            "ord" => builtins::string::ord(arg_values),

            // Math functions
            "abs" => builtins::math::abs(arg_values),
            "ceil" => builtins::math::ceil(arg_values),
            "floor" => builtins::math::floor(arg_values),
            "round" => builtins::math::round(arg_values),
            "max" => builtins::math::max(arg_values),
            "min" => builtins::math::min(arg_values),
            "pow" => builtins::math::pow(arg_values),
            "sqrt" => builtins::math::sqrt(arg_values),
            "rand" | "mt_rand" => builtins::math::rand(arg_values),

            // Type functions
            "intval" => builtins::types::intval(arg_values),
            "floatval" | "doubleval" => builtins::types::floatval(arg_values),
            "strval" => builtins::types::strval(arg_values),
            "boolval" => builtins::types::boolval(arg_values),
            "gettype" => builtins::types::gettype(arg_values),
            "is_null" => builtins::types::is_null(arg_values),
            "is_bool" => builtins::types::is_bool(arg_values),
            "is_int" | "is_integer" | "is_long" => builtins::types::is_int(arg_values),
            "is_float" | "is_double" | "is_real" => builtins::types::is_float(arg_values),
            "is_string" => builtins::types::is_string(arg_values),
            "is_array" => builtins::types::is_array(arg_values),
            "is_numeric" => builtins::types::is_numeric(arg_values),
            "isset" => builtins::types::isset(arg_values),
            "empty" => builtins::types::empty(arg_values),

            // Output functions
            "print" => builtins::output::print(&mut self.output, arg_values),
            "var_dump" => builtins::output::var_dump(&mut self.output, arg_values),
            "print_r" => builtins::output::print_r(&mut self.output, arg_values),

            // Array functions
            "count" | "sizeof" => builtins::array::count(arg_values),
            "array_push" => builtins::array::array_push(arg_values),
            "array_pop" => builtins::array::array_pop(arg_values),
            "array_shift" => builtins::array::array_shift(arg_values),
            "array_unshift" => builtins::array::array_unshift(arg_values),
            "array_keys" => builtins::array::array_keys(arg_values),
            "array_values" => builtins::array::array_values(arg_values),
            "in_array" => builtins::array::in_array(arg_values),
            "array_search" => builtins::array::array_search(arg_values),
            "array_reverse" => builtins::array::array_reverse(arg_values),
            "array_merge" => builtins::array::array_merge(arg_values),
            "array_key_exists" => builtins::array::array_key_exists(arg_values),
            "range" => builtins::array::range(arg_values),
            "array_first" => builtins::array::array_first(arg_values),
            "array_last" => builtins::array::array_last(arg_values),

            // Reflection functions (PHP 8.0 attributes)
            "get_class_attributes" => {
                builtins::reflection::get_class_attributes(arg_values, &self.classes)
            }
            "get_method_attributes" => {
                builtins::reflection::get_method_attributes(arg_values, &self.classes)
            }
            "get_property_attributes" => {
                builtins::reflection::get_property_attributes(arg_values, &self.classes)
            }
            "get_function_attributes" => {
                builtins::reflection::get_function_attributes(arg_values, &self.functions)
            }
            "get_parameter_attributes" => {
                builtins::reflection::get_parameter_attributes(arg_values, &self.functions)
            }
            "get_method_parameter_attributes" => {
                builtins::reflection::get_method_parameter_attributes(arg_values, &self.classes)
            }
            "get_interface_attributes" => {
                builtins::reflection::get_interface_attributes(arg_values, &self.interfaces)
            }
            "get_trait_attributes" => {
                builtins::reflection::get_trait_attributes(arg_values, &self.traits)
            }

            // Not a built-in function
            _ => Err(format!("Unknown built-in function: {}", name)),
        }
    }
}
