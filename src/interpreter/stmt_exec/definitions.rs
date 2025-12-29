//! Type definition handlers for functions, classes, interfaces, traits, and enums
//!
//! This module implements the registration and validation of user-defined types
//! including function, class, interface, trait, and enum declarations.

use crate::ast::{Property, Stmt};
use crate::interpreter::value::Value;
use crate::interpreter::{
    ClassDefinition, ControlFlow, EnumDefinition, InterfaceDefinition, Interpreter,
    TraitDefinition, UserFunction,
};
use std::collections::HashMap;
use std::io::Write;

impl<W: Write> Interpreter<W> {
    /// Handle function declarations
    pub(super) fn handle_function_decl(
        &mut self,
        name: &str,
        params: &[crate::ast::FunctionParam],
        body: &[Stmt],
        attributes: &[crate::ast::Attribute],
    ) -> std::io::Result<ControlFlow> {
        self.functions.insert(
            name.to_string(),
            UserFunction {
                params: params.to_vec(),
                body: body.to_vec(),
                attributes: attributes.to_vec(),
            },
        );
        Ok(ControlFlow::None)
    }

    /// Handle class declarations
    pub(super) fn handle_class_decl(
        &mut self,
        name: &str,
        readonly: bool,
        parent: &Option<String>,
        interfaces: &[String],
        trait_uses: &[crate::ast::TraitUse],
        properties: &[Property],
        methods: &[crate::ast::Method],
        attributes: &[crate::ast::Attribute],
    ) -> std::io::Result<ControlFlow> {
        // Validate all implemented interfaces exist
        for iface_name in interfaces {
            if !self.interfaces.contains_key(&iface_name.to_lowercase()) {
                return Err(std::io::Error::other(format!(
                    "Interface '{}' not found",
                    iface_name
                )));
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
                return Err(std::io::Error::other(format!(
                    "Parent class '{}' not found",
                    parent_name
                )));
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
                            return Err(std::io::Error::other(
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
        all_properties.extend(properties.iter().cloned());

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
                        promoted_statements.push(Stmt::Expression(
                            crate::ast::Expr::PropertyAssign {
                                object: Box::new(crate::ast::Expr::This),
                                property: param.name.clone(),
                                value: Box::new(crate::ast::Expr::Variable(param.name.clone())),
                            },
                        ));
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
                            return Err(std::io::Error::other(
                                format!("Method '{}' in class '{}' has {} parameters but interface '{}' expects {}",
                                    method_name, name, params.len(), iface_name, method_params.len()),
                            ));
                        }
                    } else {
                        return Err(std::io::Error::other(format!(
                            "Class '{}' does not implement method '{}' from interface '{}'",
                            name, method_name, iface_name
                        )));
                    }
                }
            }
        }

        let class_def = ClassDefinition {
            name: name.to_string(),
            readonly,
            parent: parent.clone(),
            properties: all_properties,
            methods: methods_map,
            method_visibility: visibility_map,
            attributes: attributes.to_vec(),
        };

        // Store class definition (case-insensitive)
        self.classes.insert(name.to_lowercase(), class_def);
        Ok(ControlFlow::None)
    }

    /// Handle interface declarations
    pub(super) fn handle_interface_decl(
        &mut self,
        name: &str,
        parents: &[String],
        methods: &[crate::ast::InterfaceMethodSignature],
        constants: &[crate::ast::InterfaceConstant],
        attributes: &[crate::ast::Attribute],
    ) -> std::io::Result<ControlFlow> {
        // Validate parent interfaces exist
        for parent_name in parents {
            if !self.interfaces.contains_key(&parent_name.to_lowercase()) {
                return Err(std::io::Error::other(format!(
                    "Parent interface '{}' not found",
                    parent_name
                )));
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
            let value = self
                .eval_expr(&constant.value)
                .map_err(std::io::Error::other)?;
            const_map.insert(constant.name.clone(), value);
        }

        let iface_def = InterfaceDefinition {
            name: name.to_string(),
            parents: parents.to_vec(),
            methods: all_methods,
            constants: const_map,
            attributes: attributes.to_vec(),
        };

        // Store interface definition (case-insensitive)
        self.interfaces.insert(name.to_lowercase(), iface_def);
        Ok(ControlFlow::None)
    }

    /// Handle trait declarations
    pub(super) fn handle_trait_decl(
        &mut self,
        name: &str,
        uses: &[String],
        properties: &[Property],
        methods: &[crate::ast::Method],
        attributes: &[crate::ast::Attribute],
    ) -> std::io::Result<ControlFlow> {
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
        all_properties.extend(properties.iter().cloned());

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
            name: name.to_string(),
            uses: uses.to_vec(),
            properties: all_properties,
            methods: methods_map,
            method_visibility: visibility_map,
            attributes: attributes.to_vec(),
        };

        // Store trait definition (case-insensitive)
        self.traits.insert(name.to_lowercase(), trait_def);
        Ok(ControlFlow::None)
    }

    /// Handle enum declarations
    pub(super) fn handle_enum_decl(
        &mut self,
        name: &str,
        backing_type: crate::ast::EnumBackingType,
        cases: &[crate::ast::EnumCase],
        methods: &[crate::ast::Method],
        attributes: &[crate::ast::Attribute],
    ) -> std::io::Result<ControlFlow> {
        // Validate cases
        let mut case_values: HashMap<String, Value> = HashMap::new();
        let mut case_list: Vec<(String, Option<Value>)> = Vec::new();

        for case in cases {
            // Check for duplicate case names
            if case_values.contains_key(&case.name) {
                return Err(std::io::Error::other(format!(
                    "Duplicate case name '{}' in enum '{}'",
                    case.name, name
                )));
            }

            // Evaluate case value for backed enums
            let value = if let Some(ref value_expr) = case.value {
                let val = self.eval_expr(value_expr).map_err(std::io::Error::other)?;

                // Validate backing type matches
                match backing_type {
                    crate::ast::EnumBackingType::Int => {
                        if !matches!(val, Value::Integer(_)) {
                            return Err(std::io::Error::other(format!(
                                "Enum case '{}::{}' must have int backing value",
                                name, case.name
                            )));
                        }
                    }
                    crate::ast::EnumBackingType::String => {
                        if !matches!(val, Value::String(_)) {
                            return Err(std::io::Error::other(format!(
                                "Enum case '{}::{}' must have string backing value",
                                name, case.name
                            )));
                        }
                    }
                    crate::ast::EnumBackingType::None => {
                        return Err(std::io::Error::other("Pure enum cannot have case values"));
                    }
                }

                // Check for duplicate values
                for (_, existing_val) in &case_list {
                    if let Some(existing) = existing_val {
                        if self.values_identical(existing, &val) {
                            return Err(std::io::Error::other(format!(
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
            name: name.to_string(),
            backing_type,
            cases: case_list,
            methods: method_map,
            method_visibility: visibility_map,
            attributes: attributes.to_vec(),
        };

        self.enums.insert(name.to_lowercase(), enum_def);
        Ok(ControlFlow::None)
    }
}
