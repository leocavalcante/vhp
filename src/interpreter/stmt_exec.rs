//! Statement execution implementation for the interpreter
//!
//! This module handles executing all statement types including:
//! - Echo and expression statements
//! - Control flow (if, while, for, foreach, switch)
//! - Function, class, interface, trait, and enum definitions
//! - Break, continue, and return

use crate::ast::{Program, Property, Stmt, SwitchCase};
use crate::interpreter::value::Value;
use crate::interpreter::{
    ClassDefinition, ControlFlow, EnumDefinition, InterfaceDefinition, Interpreter, TraitDefinition,
    UserFunction,
};
use std::collections::HashMap;
use std::io;
use std::io::Write;

impl<W: Write> Interpreter<W> {
    pub(super) fn execute_stmt(&mut self, stmt: &Stmt) -> io::Result<ControlFlow> {
        match stmt {
            Stmt::Echo(exprs) => {
                for expr in exprs {
                    let value = self.eval_expr(expr).map_err(|e| {
                        io::Error::other(e)
                    })?;
                    write!(self.output, "{}", value.to_output_string())?;
                }
                Ok(ControlFlow::None)
            }
            Stmt::Expression(expr) => {
                self.eval_expr(expr).map_err(|e| {
                    io::Error::other(e)
                })?;
                Ok(ControlFlow::None)
            }
            Stmt::Html(html) => {
                write!(self.output, "{}", html)?;
                Ok(ControlFlow::None)
            }
            Stmt::If {
                condition,
                then_branch,
                elseif_branches,
                else_branch,
            } => {
                let cond_value = self.eval_expr(condition).map_err(|e| {
                    io::Error::other(e)
                })?;

                if cond_value.to_bool() {
                    for stmt in then_branch {
                        let cf = self.execute_stmt(stmt)?;
                        if cf != ControlFlow::None {
                            return Ok(cf);
                        }
                    }
                } else {
                    let mut executed = false;
                    for (elseif_cond, elseif_body) in elseif_branches {
                        let elseif_value = self.eval_expr(elseif_cond).map_err(|e| {
                            io::Error::other(e)
                        })?;
                        if elseif_value.to_bool() {
                            for stmt in elseif_body {
                                let cf = self.execute_stmt(stmt)?;
                                if cf != ControlFlow::None {
                                    return Ok(cf);
                                }
                            }
                            executed = true;
                            break;
                        }
                    }

                    if !executed {
                        if let Some(else_body) = else_branch {
                            for stmt in else_body {
                                let cf = self.execute_stmt(stmt)?;
                                if cf != ControlFlow::None {
                                    return Ok(cf);
                                }
                            }
                        }
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::While { condition, body } => {
                loop {
                    let cond_value = self.eval_expr(condition).map_err(|e| {
                        io::Error::other(e)
                    })?;

                    if !cond_value.to_bool() {
                        break;
                    }

                    for stmt in body {
                        let cf = self.execute_stmt(stmt)?;
                        match cf {
                            ControlFlow::Break => return Ok(ControlFlow::None),
                            ControlFlow::Continue => break,
                            ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                            ControlFlow::None => {}
                        }
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::DoWhile { body, condition } => {
                loop {
                    let mut should_break = false;
                    let mut return_val = None;
                    for stmt in body {
                        let cf = self.execute_stmt(stmt)?;
                        match cf {
                            ControlFlow::Break => {
                                should_break = true;
                                break;
                            }
                            ControlFlow::Continue => break,
                            ControlFlow::Return(v) => {
                                return_val = Some(v);
                                break;
                            }
                            ControlFlow::None => {}
                        }
                    }

                    if let Some(v) = return_val {
                        return Ok(ControlFlow::Return(v));
                    }

                    if should_break {
                        break;
                    }

                    let cond_value = self.eval_expr(condition).map_err(|e| {
                        io::Error::other(e)
                    })?;

                    if !cond_value.to_bool() {
                        break;
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::For {
                init,
                condition,
                update,
                body,
            } => {
                if let Some(init_expr) = init {
                    self.eval_expr(init_expr).map_err(|e| {
                        io::Error::other(e)
                    })?;
                }

                loop {
                    if let Some(cond_expr) = condition {
                        let cond_value = self.eval_expr(cond_expr).map_err(|e| {
                            io::Error::other(e)
                        })?;
                        if !cond_value.to_bool() {
                            break;
                        }
                    }

                    let mut should_break = false;
                    let mut return_val = None;
                    for stmt in body {
                        let cf = self.execute_stmt(stmt)?;
                        match cf {
                            ControlFlow::Break => {
                                should_break = true;
                                break;
                            }
                            ControlFlow::Continue => break,
                            ControlFlow::Return(v) => {
                                return_val = Some(v);
                                break;
                            }
                            ControlFlow::None => {}
                        }
                    }

                    if let Some(v) = return_val {
                        return Ok(ControlFlow::Return(v));
                    }

                    if should_break {
                        break;
                    }

                    if let Some(update_expr) = update {
                        self.eval_expr(update_expr).map_err(|e| {
                            io::Error::other(e)
                        })?;
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::Foreach {
                array,
                key,
                value,
                body,
            } => {
                let array_val = self.eval_expr(array).map_err(|e| {
                    io::Error::other(e)
                })?;

                match array_val {
                    Value::Array(arr) => {
                        for (k, v) in arr {
                            // Bind key if specified
                            if let Some(key_name) = key {
                                self.variables.insert(key_name.clone(), k.to_value());
                            }

                            // Bind value
                            self.variables.insert(value.clone(), v);

                            // Execute body
                            for stmt in body {
                                let cf = self.execute_stmt(stmt)?;
                                match cf {
                                    ControlFlow::Break => return Ok(ControlFlow::None),
                                    ControlFlow::Continue => break,
                                    ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                                    ControlFlow::None => {}
                                }
                            }
                        }
                        Ok(ControlFlow::None)
                    }
                    _ => {
                        // PHP would emit a warning here, we just skip
                        Ok(ControlFlow::None)
                    }
                }
            }
            Stmt::Switch {
                expr,
                cases,
                default,
            } => {
                let switch_value = self.eval_expr(expr).map_err(|e| {
                    io::Error::other(e)
                })?;

                let mut matched = false;
                let mut fall_through = false;

                for SwitchCase { value, body } in cases {
                    if !matched && !fall_through {
                        let case_value = self.eval_expr(value).map_err(|e| {
                            io::Error::other(e)
                        })?;
                        if switch_value.loose_equals(&case_value) {
                            matched = true;
                        }
                    }

                    if matched || fall_through {
                        for stmt in body {
                            let cf = self.execute_stmt(stmt)?;
                            match cf {
                                ControlFlow::Break => return Ok(ControlFlow::None),
                                ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                                _ => {}
                            }
                        }
                        fall_through = true;
                    }
                }

                if !matched && !fall_through {
                    if let Some(default_body) = default {
                        for stmt in default_body {
                            let cf = self.execute_stmt(stmt)?;
                            match cf {
                                ControlFlow::Break => return Ok(ControlFlow::None),
                                ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                                _ => {}
                            }
                        }
                    }
                }

                Ok(ControlFlow::None)
            }
            Stmt::Break => Ok(ControlFlow::Break),
            Stmt::Continue => Ok(ControlFlow::Continue),
            Stmt::Function { name, params, body, attributes } => {
                self.functions.insert(
                    name.clone(),
                    UserFunction {
                        params: params.clone(),
                        body: body.clone(),
                        attributes: attributes.clone(),
                    },
                );
                Ok(ControlFlow::None)
            }
            Stmt::Return(expr) => {
                let value = if let Some(e) = expr {
                    self.eval_expr(e).map_err(|e| {
                        io::Error::other(e)
                    })?
                } else {
                    Value::Null
                };
                Ok(ControlFlow::Return(value))
            }
            Stmt::Class {
                name,
                readonly,
                parent,
                interfaces,
                trait_uses,
                properties,
                methods,
                attributes,
            } => {
                // Validate all implemented interfaces exist
                for iface_name in interfaces {
                    if !self.interfaces.contains_key(&iface_name.to_lowercase()) {
                        return Err(io::Error::other(
                            format!("Interface '{}' not found", iface_name),
                        ));
                    }
                }

                // Build methods map
                let mut methods_map = HashMap::new();
                let mut visibility_map = HashMap::new();
                let mut all_properties = Vec::new();

                // If there's a parent class, inherit its properties and methods
                if let Some(parent_name) = parent {
                    let parent_name_lower = parent_name.to_lowercase();
                    if let Some(parent_class) = self.classes.get(&parent_name_lower).cloned() {
                        // Inherit parent properties
                        all_properties.extend(parent_class.properties.clone());

                        // Inherit parent methods
                        for (method_name, method_func) in parent_class.methods.iter() {
                            methods_map.insert(method_name.clone(), method_func.clone());
                        }
                        for (method_name, visibility) in parent_class.method_visibility.iter() {
                            visibility_map.insert(method_name.clone(), *visibility);
                        }
                    } else {
                        return Err(io::Error::other(
                            format!("Parent class '{}' not found", parent_name),
                        ));
                    }
                }

                // Add properties from traits
                for trait_use in trait_uses {
                    for trait_name in &trait_use.traits {
                        if let Some(trait_def) = self.traits.get(&trait_name.to_lowercase()).cloned() {
                            // Add trait properties
                            all_properties.extend(trait_def.properties.clone());

                            // Add trait methods (checking for conflicts)
                            for (method_name, method_func) in trait_def.methods.iter() {
                                if methods_map.contains_key(method_name) {
                                    // Conflict: method already exists from another trait or class
                                    return Err(io::Error::other(
                                        format!("Trait method '{}' conflicts with other trait or class method in '{}'",
                                            method_name, name),
                                    ));
                                }
                                methods_map.insert(method_name.clone(), method_func.clone());
                            }
                            for (method_name, visibility) in trait_def.method_visibility.iter() {
                                if !visibility_map.contains_key(method_name) {
                                    visibility_map.insert(method_name.clone(), *visibility);
                                }
                            }
                        }
                    }
                }

                // Add current class properties (can override parent/trait properties)
                all_properties.extend(properties.clone());

                // Add current class methods (can override parent/trait methods)
                for method in methods {
                    let mut method_body = method.body.clone();

                    // Handle constructor property promotion (PHP 8.0)
                    if method.name.to_lowercase() == "__construct" {
                        let mut promoted_statements = Vec::new();

                        // Extract promoted properties and prepend assignments
                        for param in &method.params {
                            if let Some(visibility) = param.visibility {
                                // Add promoted property
                                all_properties.push(Property {
                                    name: param.name.clone(),
                                    visibility,
                                    default: param.default.clone(),
                                    readonly: param.readonly,
                                    attributes: param.attributes.clone(),
                                });

                                // Prepend assignment: $this->param_name = $param_name
                                promoted_statements.push(Stmt::Expression(crate::ast::Expr::PropertyAssign {
                                    object: Box::new(crate::ast::Expr::This),
                                    property: param.name.clone(),
                                    value: Box::new(crate::ast::Expr::Variable(param.name.clone())),
                                }));
                            }
                        }

                        // Prepend promoted property assignments to constructor body
                        promoted_statements.extend(method_body);
                        method_body = promoted_statements;
                    }

                    let func = UserFunction {
                        params: method.params.clone(),
                        body: method_body,
                        attributes: method.attributes.clone(),
                    };
                    let method_name_lower = method.name.to_lowercase();
                    methods_map.insert(method_name_lower.clone(), func);
                    visibility_map.insert(method_name_lower, method.visibility);
                }

                // Verify all interface methods are implemented
                for iface_name in interfaces {
                    if let Some(iface_def) = self.interfaces.get(&iface_name.to_lowercase()) {
                        for (method_name, method_params) in &iface_def.methods {
                            let method_name_lower = method_name.to_lowercase();
                            if let Some(UserFunction { params, .. }) = methods_map.get(&method_name_lower) {
                                // Verify parameter count matches
                                if params.len() != method_params.len() {
                                    return Err(io::Error::other(
                                        format!("Method '{}' in class '{}' has {} parameters but interface '{}' expects {}",
                                            method_name, name, params.len(), iface_name, method_params.len()),
                                    ));
                                }
                            } else {
                                return Err(io::Error::other(
                                    format!("Class '{}' does not implement method '{}' from interface '{}'",
                                        name, method_name, iface_name),
                                ));
                            }
                        }
                    }
                }

                let class_def = ClassDefinition {
                    name: name.clone(),
                    readonly: *readonly,
                    parent: parent.clone(),
                    properties: all_properties,
                    methods: methods_map,
                    method_visibility: visibility_map,
                    attributes: attributes.clone(),
                };

                // Store class definition (case-insensitive)
                self.classes.insert(name.to_lowercase(), class_def);
                Ok(ControlFlow::None)
            }
            Stmt::Interface {
                name,
                parents,
                methods,
                constants,
                attributes,
            } => {
                // Validate parent interfaces exist
                for parent_name in parents {
                    if !self.interfaces.contains_key(&parent_name.to_lowercase()) {
                        return Err(io::Error::other(
                            format!("Parent interface '{}' not found", parent_name),
                        ));
                    }
                }

                // Collect all methods from parent interfaces
                let mut all_methods = Vec::new();
                for parent_name in parents {
                    if let Some(parent_iface) = self.interfaces.get(&parent_name.to_lowercase()).cloned() {
                        all_methods.extend(parent_iface.methods.clone());
                    }
                }

                // Add current interface methods
                for method in methods {
                    all_methods.push((method.name.clone(), method.params.clone()));
                }

                // Evaluate constants
                let mut const_map = HashMap::new();
                for constant in constants {
                    let value = self.eval_expr(&constant.value).map_err(|e| {
                        io::Error::other(e)
                    })?;
                    const_map.insert(constant.name.clone(), value);
                }

                let iface_def = InterfaceDefinition {
                    name: name.clone(),
                    parents: parents.clone(),
                    methods: all_methods,
                    constants: const_map,
                    attributes: attributes.clone(),
                };

                // Store interface definition (case-insensitive)
                self.interfaces.insert(name.to_lowercase(), iface_def);
                Ok(ControlFlow::None)
            }
            Stmt::Trait {
                name,
                uses,
                properties,
                methods,
                attributes,
            } => {
                // Build methods map
                let mut methods_map = HashMap::new();
                let mut visibility_map = HashMap::new();
                let mut all_properties = Vec::new();

                // Add properties from used traits
                for trait_name in uses {
                    if let Some(trait_def) = self.traits.get(&trait_name.to_lowercase()).cloned() {
                        // Add trait properties
                        all_properties.extend(trait_def.properties.clone());

                        // Add trait methods
                        for (method_name, method_func) in trait_def.methods.iter() {
                            methods_map.insert(method_name.clone(), method_func.clone());
                        }
                        for (method_name, visibility) in trait_def.method_visibility.iter() {
                            visibility_map.insert(method_name.clone(), *visibility);
                        }
                    }
                }

                // Add current trait properties
                all_properties.extend(properties.clone());

                // Add current trait methods (override used trait methods)
                for method in methods {
                    let func = UserFunction {
                        params: method.params.clone(),
                        body: method.body.clone(),
                        attributes: method.attributes.clone(),
                    };
                    let method_name_lower = method.name.to_lowercase();
                    methods_map.insert(method_name_lower.clone(), func);
                    visibility_map.insert(method_name_lower, method.visibility);
                }

                let trait_def = TraitDefinition {
                    name: name.clone(),
                    uses: uses.clone(),
                    properties: all_properties,
                    methods: methods_map,
                    method_visibility: visibility_map,
                    attributes: attributes.clone(),
                };

                // Store trait definition (case-insensitive)
                self.traits.insert(name.to_lowercase(), trait_def);
                Ok(ControlFlow::None)
            }
            Stmt::Enum {
                name,
                backing_type,
                cases,
                methods,
                attributes,
            } => {
                // Validate cases
                let mut case_values: HashMap<String, Value> = HashMap::new();
                let mut case_list: Vec<(String, Option<Value>)> = Vec::new();

                for case in cases {
                    // Check for duplicate case names
                    if case_values.contains_key(&case.name) {
                        return Err(io::Error::other(format!(
                            "Duplicate case name '{}' in enum '{}'",
                            case.name, name
                        )));
                    }

                    // Evaluate case value for backed enums
                    let value = if let Some(ref value_expr) = case.value {
                        let val = self.eval_expr(value_expr).map_err(io::Error::other)?;

                        // Validate backing type matches
                        match backing_type {
                            crate::ast::EnumBackingType::Int => {
                                if !matches!(val, Value::Integer(_)) {
                                    return Err(io::Error::other(format!(
                                        "Enum case '{}::{}' must have int backing value",
                                        name, case.name
                                    )));
                                }
                            }
                            crate::ast::EnumBackingType::String => {
                                if !matches!(val, Value::String(_)) {
                                    return Err(io::Error::other(format!(
                                        "Enum case '{}::{}' must have string backing value",
                                        name, case.name
                                    )));
                                }
                            }
                            crate::ast::EnumBackingType::None => {
                                return Err(io::Error::other(
                                    "Pure enum cannot have case values",
                                ));
                            }
                        }

                        // Check for duplicate values
                        for (_, existing_val) in &case_list {
                            if let Some(existing) = existing_val {
                                if self.values_identical(existing, &val) {
                                    return Err(io::Error::other(format!(
                                        "Duplicate case value in backed enum '{}'",
                                        name
                                    )));
                                }
                            }
                        }

                        Some(val)
                    } else {
                        None
                    };

                    case_values.insert(case.name.clone(), value.clone().unwrap_or(Value::Null));
                    case_list.push((case.name.clone(), value));
                }

                // Store methods
                let mut method_map = HashMap::new();
                let mut visibility_map = HashMap::new();

                for method in methods {
                    let method_name_lower = method.name.to_lowercase();
                    method_map.insert(
                        method_name_lower.clone(),
                        UserFunction {
                            params: method.params.clone(),
                            body: method.body.clone(),
                            attributes: method.attributes.clone(),
                        },
                    );
                    visibility_map.insert(method_name_lower, method.visibility);
                }

                // Store enum definition
                let enum_def = EnumDefinition {
                    name: name.clone(),
                    backing_type: *backing_type,
                    cases: case_list,
                    methods: method_map,
                    method_visibility: visibility_map,
                    attributes: attributes.clone(),
                };

                self.enums.insert(name.to_lowercase(), enum_def);
                Ok(ControlFlow::None)
            }
        }
    }

    /// Check if two values are identical (===)
    pub(super) fn values_identical(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            _ => false,
        }
    }

    pub fn execute(&mut self, program: &Program) -> io::Result<()> {
        for stmt in &program.statements {
            let _ = self.execute_stmt(stmt)?;
        }
        self.output.flush()?;
        Ok(())
    }
}
