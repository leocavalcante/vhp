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
        _return_type: &Option<crate::ast::TypeHint>,
        body: &[Stmt],
        attributes: &[crate::ast::Attribute],
    ) -> std::io::Result<ControlFlow> {
        self.functions.insert(
            name.to_string(),
            UserFunction {
                params: params.to_vec(),
                return_type: _return_type.clone(),
                body: body.to_vec(),
                is_abstract: false, // regular functions are never abstract
                is_final: false,    // regular functions are never final
                attributes: attributes.to_vec(),
            },
        );
        Ok(ControlFlow::None)
    }

    /// Handle class declarations
    #[allow(clippy::too_many_arguments)]
    pub(super) fn handle_class_decl(
        &mut self,
        name: &str,
        is_abstract: bool,
        is_final: bool,
        readonly: bool,
        parent: &Option<crate::ast::QualifiedName>,
        interfaces: &[crate::ast::QualifiedName],
        trait_uses: &[crate::ast::TraitUse],
        properties: &[Property],
        methods: &[crate::ast::Method],
        attributes: &[crate::ast::Attribute],
    ) -> std::io::Result<ControlFlow> {
        // Resolve interface names through namespace context
        let resolved_interfaces: Vec<String> = interfaces
            .iter()
            .map(|iface| self.namespace_context.resolve_class(iface))
            .collect();

        // Validate all implemented interfaces exist
        for iface_name in &resolved_interfaces {
            if !self.interfaces.contains_key(&iface_name.to_lowercase()) {
                return Err(std::io::Error::other(format!(
                    "Interface '{}' not found",
                    iface_name
                )));
            }
        }

        // Resolve parent class name if exists
        let resolved_parent = parent
            .as_ref()
            .map(|p| self.namespace_context.resolve_class(p));

        // If extending a class, check parent isn't final and validate abstract methods
        if let Some(parent_name) = &resolved_parent {
            let parent_name_lower = parent_name.to_lowercase();
            if let Some(parent_class) = self.classes.get(&parent_name_lower).cloned() {
                // Cannot extend final class
                if parent_class.is_final {
                    return Err(std::io::Error::other(format!(
                        "Class {} cannot extend final class {}",
                        name, parent_name
                    )));
                }
                
                // If this class is not abstract, it must implement all abstract methods from parent
                if !is_abstract && parent_class.is_abstract {
                    for (method_name, method_func) in parent_class.methods.iter() {
                        if method_func.is_abstract {
                            // Check if this method is implemented
                            let implemented = methods.iter().any(|m| {
                                m.name.to_lowercase() == method_name.to_lowercase()
                            });
                            if !implemented {
                                return Err(std::io::Error::other(format!(
                                    "Class {} must implement abstract method {}::{}",
                                    name, parent_name, method_name
                                )));
                            }
                        }
                    }
                }
            }
        }

        // Build methods map
        let mut methods_map = HashMap::new();
        let mut visibility_map = HashMap::new();
        let mut all_properties = Vec::new();

        // If there's a parent class, inherit its properties and methods
        if let Some(parent_name) = &resolved_parent {
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
            let method_name_lower = method.name.to_lowercase();
            
            // Check if we're trying to override a final method
            if let Some(existing_method) = methods_map.get(&method_name_lower) {
                if existing_method.is_final {
                    return Err(std::io::Error::other(format!(
                        "Cannot override final method {}",
                        method.name
                    )));
                }
            }
            
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
                            is_static: false, // Promoted properties cannot be static
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
                return_type: method.return_type.clone(),
                body: method_body,
                is_abstract: method.is_abstract,
                is_final: method.is_final,
                attributes: method.attributes.clone(),
            };
            methods_map.insert(method_name_lower.clone(), func);
            visibility_map.insert(method_name_lower, method.visibility);
        }

        // Verify all interface methods are implemented
        for iface_name in &resolved_interfaces {
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

        // Clone values that will be used after moving to ClassDefinition
        let resolved_parent_clone = resolved_parent.clone();
        let all_properties_clone = all_properties.clone();

        let class_def = ClassDefinition {
            name: name.to_string(),
            is_abstract,
            is_final,
            readonly,
            parent: resolved_parent,
            properties: all_properties,
            methods: methods_map,
            method_visibility: visibility_map,
            attributes: attributes.to_vec(),
        };

        // Store class definition with fully qualified name (case-insensitive)
        let fqn = if self.namespace_context.current.is_empty() {
            name.to_lowercase()
        } else {
            format!(
                "{}\\{}",
                self.namespace_context.current.join("\\"),
                name
            )
            .to_lowercase()
        };
        self.classes.insert(fqn.clone(), class_def);

        // Initialize static properties for this class
        let mut static_props = HashMap::new();

        // First, copy parent's static properties if there's inheritance
        if let Some(parent_name) = &resolved_parent_clone {
            let parent_key = parent_name.to_lowercase();
            if let Some(parent_statics) = self.static_properties.get(&parent_key) {
                static_props.extend(parent_statics.clone());
            }
        }

        // Then add/override with this class's static properties
        for prop in &all_properties_clone {
            if prop.is_static {
                let value = if let Some(default_expr) = &prop.default {
                    self.eval_expr(default_expr).map_err(std::io::Error::other)?
                } else {
                    Value::Null
                };
                static_props.insert(prop.name.clone(), value);
            }
        }

        // Store static properties if any exist
        if !static_props.is_empty() {
            self.static_properties.insert(fqn, static_props);
        }

        Ok(ControlFlow::None)
    }

    /// Handle interface declarations
    pub(super) fn handle_interface_decl(
        &mut self,
        name: &str,
        parents: &[crate::ast::QualifiedName],
        methods: &[crate::ast::InterfaceMethodSignature],
        constants: &[crate::ast::InterfaceConstant],
        attributes: &[crate::ast::Attribute],
    ) -> std::io::Result<ControlFlow> {
        // Resolve parent interface names
        let resolved_parents: Vec<String> = parents
            .iter()
            .map(|p| self.namespace_context.resolve_class(p))
            .collect();

        // Validate parent interfaces exist
        for parent_name in &resolved_parents {
            if !self.interfaces.contains_key(&parent_name.to_lowercase()) {
                return Err(std::io::Error::other(format!(
                    "Parent interface '{}' not found",
                    parent_name
                )));
            }
        }

        // Collect all methods from parent interfaces
        let mut all_methods = Vec::new();
        for parent_name in &resolved_parents {
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
            parents: resolved_parents,
            methods: all_methods,
            constants: const_map,
            attributes: attributes.to_vec(),
        };

        // Store interface definition with fully qualified name (case-insensitive)
        let fqn = if self.namespace_context.current.is_empty() {
            name.to_lowercase()
        } else {
            format!(
                "{}\\{}",
                self.namespace_context.current.join("\\"),
                name
            )
            .to_lowercase()
        };
        self.interfaces.insert(fqn, iface_def);
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
                return_type: method.return_type.clone(),
                body: method.body.clone(),
                is_abstract: method.is_abstract,
                is_final: method.is_final,
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
                    return_type: method.return_type.clone(),
                    body: method.body.clone(),
                    is_abstract: method.is_abstract,
                    is_final: method.is_final,
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
