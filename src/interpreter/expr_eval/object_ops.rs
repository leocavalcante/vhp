//! Object/OOP operations for expression evaluation
//!
//! Handles:
//! - Object instantiation (new ClassName)
//! - Property access and assignment
//! - Method calls
//! - Static method calls

use crate::ast::{Argument, Expr};
use crate::interpreter::value::Value;
use crate::interpreter::Interpreter;
use std::io::Write;

/// Evaluate object instantiation (new ClassName(...))
pub(crate) fn eval_new<W: Write>(
    interpreter: &mut Interpreter<W>,
    class_name: &str,
    args: &[Argument],
) -> Result<Value, String> {
    // Delegate to the existing eval_new in objects.rs
    interpreter.eval_new(class_name, args)
}

/// Evaluate property access ($obj->property)
pub(crate) fn eval_property_access<W: Write>(
    interpreter: &mut Interpreter<W>,
    object: &Expr,
    property: &str,
) -> Result<Value, String> {
    // Delegate to the existing eval_property_access in objects.rs
    interpreter.eval_property_access(object, property)
}

/// Evaluate property assignment ($obj->property = value)
pub(crate) fn eval_property_assign<W: Write>(
    interpreter: &mut Interpreter<W>,
    object_expr: &Expr,
    property: &str,
    value_expr: &Expr,
) -> Result<Value, String> {
    // Delegate to the existing eval_property_assign in objects.rs
    interpreter.eval_property_assign(object_expr, property, value_expr)
}

/// Evaluate method call ($obj->method(...))
pub(crate) fn eval_method_call<W: Write>(
    interpreter: &mut Interpreter<W>,
    object: &Expr,
    method: &str,
    args: &[Argument],
) -> Result<Value, String> {
    // Delegate to the existing eval_method_call in objects.rs
    interpreter.eval_method_call(object, method, args)
}

/// Evaluate static method call (ClassName::method(...))
pub(crate) fn eval_static_method_call<W: Write>(
    interpreter: &mut Interpreter<W>,
    class_name: &str,
    method: &str,
    args: &[Argument],
) -> Result<Value, String> {
    // Delegate to the existing eval_static_method_call in objects.rs
    interpreter.eval_static_method_call(class_name, method, args)
}

/// Evaluate anonymous class instantiation (new class(...) { ... })
pub(crate) fn eval_new_anonymous_class<W: Write>(
    interpreter: &mut Interpreter<W>,
    constructor_args: &[Argument],
    parent: &Option<String>,
    interfaces: &[String],
    traits: &[crate::ast::TraitUse],
    properties: &[crate::ast::Property],
    methods: &[crate::ast::Method],
) -> Result<Value, String> {
    // Generate unique class name
    interpreter.anonymous_class_counter += 1;
    let class_name = format!("class@anonymous${}", interpreter.anonymous_class_counter);
    
    // Create class definition
    let mut class_methods = std::collections::HashMap::new();
    let mut method_visibility = std::collections::HashMap::new();
    
    for method in methods {
        let user_func = crate::interpreter::UserFunction {
            params: method.params.clone(),
            body: method.body.clone(),
            is_abstract: method.is_abstract,
            is_final: method.is_final,
            attributes: method.attributes.clone(),
        };
        class_methods.insert(method.name.to_lowercase(), user_func);
        method_visibility.insert(method.name.to_lowercase(), method.visibility.clone());
    }
    
    let class_def = crate::interpreter::ClassDefinition {
        name: class_name.clone(),
        is_abstract: false,
        is_final: true, // Anonymous classes are implicitly final
        readonly: false,
        parent: parent.clone(),
        properties: properties.to_vec(),
        methods: class_methods,
        method_visibility,
        attributes: vec![],
    };
    
    // Register class
    interpreter.classes.insert(class_name.to_lowercase(), class_def);
    
    // TODO: Handle traits and validate interface implementation
    // For now, just log warnings if they're used
    if !traits.is_empty() {
        eprintln!("Warning: Traits in anonymous classes not yet fully supported");
    }
    if !interfaces.is_empty() {
        eprintln!("Warning: Interface validation in anonymous classes not yet fully supported");
    }
    
    // Now instantiate the class with constructor arguments
    interpreter.eval_new(&class_name, constructor_args)
}
