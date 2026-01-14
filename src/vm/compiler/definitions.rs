use super::Compiler;

use crate::ast::{Attribute, Expr, Method, QualifiedName, TraitUse};
use crate::vm::opcode::Opcode;
use std::sync::Arc;

impl Compiler {
    pub(crate) fn compile_class_internal(
        &mut self,
        name: &str,
        is_abstract: bool,
        is_final: bool,
        readonly: bool,
        parent: &Option<QualifiedName>,
        interfaces: &[QualifiedName],
        trait_uses: &[TraitUse],
        properties: &[crate::ast::Property],
        methods: &[Method],
        attributes: &[Attribute],
    ) -> Result<(), String> {
        use crate::vm::class::CompiledClass;

        let qualified_name = if let Some(ref ns) = self.current_namespace {
            format!("{}\\{}", ns, name)
        } else {
            name.to_string()
        };

        let resolved_parent = parent.as_ref().map(|p| self.resolve_qualified_name(p));

        if let Some(ref parent_name) = resolved_parent {
            let is_builtin = matches!(
                parent_name.as_str(),
                "Exception"
                    | "Error"
                    | "TypeError"
                    | "InvalidArgumentException"
                    | "UnhandledMatchError"
            );

            if let Some(parent_class) = self.classes.get(parent_name) {
                if parent_class.is_final {
                    return Err(format!("cannot extend final class {}", parent_name));
                }
            } else if !is_builtin {
                return Err(format!("Parent class '{}' not found", parent_name));
            }
        }

        let resolved_interfaces: Vec<String> = interfaces
            .iter()
            .map(|i| self.resolve_qualified_name(i))
            .collect();

        let mut compiled_class = CompiledClass::new(qualified_name.clone());
        compiled_class.is_abstract = is_abstract;
        compiled_class.is_final = is_final;
        compiled_class.readonly = readonly;
        compiled_class.parent = resolved_parent.clone();
        compiled_class.interfaces = resolved_interfaces.clone();
        compiled_class.traits = trait_uses.iter().flat_map(|t| t.traits.clone()).collect();
        compiled_class.attributes = attributes.to_vec();

        for iface_name in &resolved_interfaces {
            if !self.interfaces.contains_key(iface_name) {
                return Err(format!("Interface '{}' not found", iface_name));
            }
        }

        for prop in properties {
            let mut compiled_prop = crate::vm::class::CompiledProperty::from_ast(prop, readonly);

            for hook in &prop.hooks {
                let hook_method_name = match hook.hook_type {
                    crate::ast::PropertyHookType::Get => {
                        format!("{}::__prop_get_{}", qualified_name, prop.name)
                    }
                    crate::ast::PropertyHookType::Set => {
                        format!("{}::__prop_set_{}", qualified_name, prop.name)
                    }
                };

                let mut hook_compiler = Compiler::new(hook_method_name.clone());

                hook_compiler.locals.insert("this".to_string(), 0);
                hook_compiler.function.local_names.push("this".to_string());
                hook_compiler.next_local = 1;

                if matches!(hook.hook_type, crate::ast::PropertyHookType::Set) {
                    hook_compiler.locals.insert("value".to_string(), 1);
                    hook_compiler.function.local_names.push("value".to_string());
                    hook_compiler.next_local = 2;
                    hook_compiler.function.param_count = 1;
                    hook_compiler.function.required_param_count = 1;
                }

                hook_compiler.function.local_count = hook_compiler.next_local;

                match &hook.body {
                    crate::ast::PropertyHookBody::Expression(expr) => {
                        hook_compiler.compile_expr(expr)?;
                        if matches!(hook.hook_type, crate::ast::PropertyHookType::Get) {
                            hook_compiler.emit(Opcode::Return);
                        } else {
                            hook_compiler.emit(Opcode::Pop);
                            hook_compiler.emit(Opcode::ReturnNull);
                        }
                    }
                    crate::ast::PropertyHookBody::Block(stmts) => {
                        for stmt in stmts {
                            hook_compiler.compile_stmt(stmt)?;
                        }
                        hook_compiler.emit(Opcode::ReturnNull);
                    }
                }

                for (inner_name, inner_func) in hook_compiler.functions.drain() {
                    self.functions.insert(inner_name, inner_func);
                }

                let compiled_hook = Arc::new(hook_compiler.function);

                match hook.hook_type {
                    crate::ast::PropertyHookType::Get => {
                        compiled_prop.get_hook = Some(format!("__prop_get_{}", prop.name));
                        compiled_class
                            .methods
                            .insert(format!("__prop_get_{}", prop.name), compiled_hook);
                    }
                    crate::ast::PropertyHookType::Set => {
                        compiled_prop.set_hook = Some(format!("__prop_set_{}", prop.name));
                        compiled_class
                            .methods
                            .insert(format!("__prop_set_{}", prop.name), compiled_hook);
                    }
                }
            }

            if prop.is_static {
                let default_value = compiled_prop
                    .default
                    .clone()
                    .unwrap_or(crate::runtime::Value::Null);
                compiled_class
                    .static_properties
                    .insert(prop.name.clone(), default_value);
                if prop.readonly || readonly {
                    compiled_class
                        .readonly_static_properties
                        .insert(prop.name.clone());
                }
            }
            compiled_class.properties.push(compiled_prop);
        }

        for method in methods {
            if method.name == "__construct" {
                for param in &method.params {
                    if param.visibility.is_some() {
                        let promoted_prop = crate::vm::class::CompiledProperty {
                            name: param.name.clone(),
                            visibility: param.visibility.unwrap(),
                            write_visibility: None,
                            default: None,
                            readonly: param.readonly || readonly,
                            is_static: false,
                            type_hint: None,
                            attributes: param.attributes.clone(),
                            get_hook: None,
                            set_hook: None,
                        };
                        compiled_class.properties.push(promoted_prop);
                    }
                }
            }
        }

        for trait_name in &compiled_class.traits {
            if let Some(trait_def) = self.traits.get(trait_name) {
                for trait_prop in &trait_def.properties {
                    if !compiled_class
                        .properties
                        .iter()
                        .any(|p| p.name == trait_prop.name)
                    {
                        compiled_class.properties.push(trait_prop.clone());
                    }
                }
            }
        }

        let mut trait_methods: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for trait_name in &compiled_class.traits {
            if let Some(trait_def) = self.traits.get(trait_name) {
                for method_name in trait_def.methods.keys() {
                    trait_methods
                        .entry(method_name.clone())
                        .or_default()
                        .push(trait_name.clone());
                }
            }
        }

        for (method_name, defining_traits) in &trait_methods {
            if defining_traits.len() > 1 {
                let class_defines_method = methods.iter().any(|m| &m.name == method_name);
                if !class_defines_method {
                    return Err(format!(
                        "Trait method conflict: {} is defined in multiple traits ({})",
                        method_name,
                        defining_traits.join(", ")
                    ));
                }
            }
        }

        let interfaces_to_check: Vec<&str> =
            resolved_interfaces.iter().map(|s| s.as_str()).collect();
        let mut interfaces: Vec<String> = interfaces_to_check
            .iter()
            .filter_map(|n| {
                if let Some(iface) = self.interfaces.get(*n) {
                    Some(iface.name.clone())
                } else {
                    None
                }
            })
            .collect();

        let mut parent_interfaces: Vec<String> = Vec::new();
        for iface_name in &interfaces {
            if let Some(iface) = self.interfaces.get(iface_name) {
                for parent in &iface.parents {
                    if !parent_interfaces.contains(parent) {
                        parent_interfaces.push(parent.clone());
                    }
                }
            }
        }

        for parent_iface in &parent_interfaces {
            if !interfaces.contains(parent_iface) {
                interfaces.push(parent_iface.clone());
            }
        }

        compiled_class.interfaces = interfaces.clone();

        for method in methods {
            if method.is_abstract {
                if !is_abstract {
                    return Err(format!(
                        "Cannot declare method {}::{} as abstract if class is not abstract",
                        name, method.name
                    ));
                }
            }

            if method.is_final && !is_abstract {
                compiled_class
                    .method_finals
                    .insert(method.name.clone(), true);
            }

            if let Some(parent_name) = compiled_class.parent.as_ref() {
                if let Some(parent_class) = self.classes.get(parent_name) {
                    let has_method = parent_class.methods.contains_key(&method.name)
                        || parent_class.static_methods.contains_key(&method.name);
                    if has_method
                        && parent_class
                            .method_finals
                            .get(&method.name)
                            .copied()
                            .unwrap_or(false)
                    {
                        return Err(format!(
                            "Cannot override final method {}::{}",
                            parent_name, method.name
                        ));
                    }
                }
            }

            let has_override_attr = method
                .attributes
                .iter()
                .any(|attr| attr.name == "Override" || attr.name == "\\Override");

            if has_override_attr {
                let mut found_parent_method = false;

                let mut current_parent = resolved_parent.clone();
                while let Some(parent_name) = current_parent {
                    if let Some(parent_class) = self.classes.get(&parent_name) {
                        if parent_class.methods.contains_key(&method.name)
                            || parent_class.static_methods.contains_key(&method.name)
                        {
                            found_parent_method = true;
                            break;
                        }
                        current_parent = parent_class.parent.clone();
                    } else {
                        break;
                    }
                }

                if !found_parent_method {
                    for iface_name in &compiled_class.interfaces {
                        if let Some(iface_def) = self.interfaces.get(iface_name) {
                            if iface_def
                                .method_signatures
                                .iter()
                                .any(|(name, _)| name == &method.name)
                            {
                                found_parent_method = true;
                                break;
                            }
                        }
                    }
                }

                if !found_parent_method {
                    for trait_name in &compiled_class.traits {
                        if let Some(trait_def) = self.traits.get(trait_name) {
                            if trait_def.methods.contains_key(&method.name) {
                                found_parent_method = true;
                                break;
                            }
                        }
                    }
                }

                if !found_parent_method {
                    return Err(format!(
                        "{}::{} has #[\\Override] attribute, but no matching parent method exists",
                        name, method.name
                    ));
                }
            }

            let method_name = format!("{}::{}", qualified_name, method.name);
            let mut method_compiler = Compiler::new(method_name.clone());

            if !method.is_static {
                method_compiler.locals.insert("this".to_string(), 0);
                method_compiler
                    .function
                    .local_names
                    .push("this".to_string());
                method_compiler.next_local = 1;
            }

            let param_start = method_compiler.next_local;
            for (i, param) in method.params.iter().enumerate() {
                let slot = param_start + i as u16;
                method_compiler.locals.insert(param.name.clone(), slot);
                method_compiler
                    .function
                    .local_names
                    .push(param.name.clone());
            }
            method_compiler.next_local = param_start + method.params.len() as u16;
            method_compiler.function.local_count = method_compiler.next_local;
            method_compiler.function.param_count = method.params.len() as u8;
            method_compiler.function.required_param_count = method
                .params
                .iter()
                .filter(|p| p.default.is_none() && !p.is_variadic)
                .count() as u8;
            method_compiler.function.return_type = method.return_type.clone();
            method_compiler.function.is_variadic = method.params.iter().any(|p| p.is_variadic);

            method_compiler.function.parameters = method.params.clone();
            method_compiler.function.attributes = method.attributes.clone();

            for param in &method.params {
                method_compiler
                    .function
                    .param_types
                    .push(param.type_hint.clone());
            }

            for (i, param) in method.params.iter().enumerate() {
                if let Some(default_expr) = &param.default {
                    let slot = param_start + i as u16;
                    method_compiler.emit(Opcode::LoadFast(slot));
                    let skip_jump = method_compiler.emit_jump(Opcode::JumpIfNotNull(0));
                    method_compiler.emit(Opcode::Pop);
                    method_compiler.compile_expr(default_expr)?;
                    method_compiler.emit(Opcode::StoreFast(slot));
                    let end_jump = method_compiler.emit_jump(Opcode::Jump(0));
                    method_compiler.patch_jump(skip_jump);
                    method_compiler.emit(Opcode::Pop);
                    method_compiler.patch_jump(end_jump);
                }
            }

            if method.name == "__construct" && !method.is_static {
                for (i, param) in method.params.iter().enumerate() {
                    if param.visibility.is_some() {
                        let slot = param_start + i as u16;
                        method_compiler.emit(Opcode::LoadFast(slot));
                        let prop_idx = method_compiler.intern_string(param.name.clone());
                        method_compiler.emit(Opcode::StoreThisProperty(prop_idx));
                    }
                }
            }

            for stmt in &method.body {
                method_compiler.compile_stmt(stmt)?;
            }

            method_compiler.emit(Opcode::ReturnNull);

            for (inner_name, inner_func) in method_compiler.functions.drain() {
                self.functions.insert(inner_name, inner_func);
            }

            let compiled = Arc::new(method_compiler.function);
            compiled_class
                .method_visibility
                .insert(method.name.clone(), method.visibility);
            compiled_class
                .method_finals
                .insert(method.name.clone(), method.is_final);
            compiled_class
                .method_abstracts
                .insert(method.name.clone(), method.is_abstract);

            if method.is_static {
                compiled_class
                    .static_methods
                    .insert(method.name.clone(), compiled);
            } else {
                compiled_class.methods.insert(method.name.clone(), compiled);
            }
        }

        if !is_abstract {
            if let Some(ref parent_name) = resolved_parent {
                if let Some(parent_class) = self.classes.get(parent_name) {
                    for (method_name, is_abs) in &parent_class.method_abstracts {
                        if *is_abs {
                            let has_method = compiled_class.methods.contains_key(method_name)
                                || compiled_class.static_methods.contains_key(method_name);
                            if !has_method {
                                return Err(format!(
                                    "Class '{}' must implement abstract method '{}' from class '{}'",
                                    name, method_name, parent_name
                                ));
                            }
                        }
                    }
                }
            }
        }

        if !is_abstract {
            for interface in interfaces {
                if let Some(iface_def) = self.interfaces.get(&interface) {
                    for (method_name, _param_count) in &iface_def.method_signatures {
                        let has_method = compiled_class.methods.contains_key(method_name)
                            || compiled_class.static_methods.contains_key(method_name);
                        if !has_method {
                            return Err(format!(
                                "Class '{}' does not implement method '{}' from interface '{}'",
                                name, method_name, interface
                            ));
                        }
                    }
                }
            }
        }

        self.classes
            .insert(qualified_name, Arc::new(compiled_class));
        Ok(())
    }

    pub(crate) fn compile_interface_internal(
        &mut self,
        name: &str,
        parents: &[QualifiedName],
        methods: &[crate::ast::InterfaceMethodSignature],
        constants: &[crate::ast::InterfaceConstant],
        attributes: &[Attribute],
    ) -> Result<(), String> {
        use crate::vm::class::CompiledInterface;

        let qualified_name = if let Some(ref ns) = self.current_namespace {
            format!("{}\\{}", ns, name)
        } else {
            name.to_string()
        };

        let mut compiled_interface = CompiledInterface::new(qualified_name.clone());
        compiled_interface.parents = parents
            .iter()
            .map(|p| self.resolve_qualified_name(p))
            .collect();
        compiled_interface.attributes = attributes.to_vec();

        for method in methods {
            compiled_interface
                .method_signatures
                .push((method.name.clone(), method.params.len() as u8));
        }

        for constant in constants {
            let _ = constant;
        }

        self.interfaces
            .insert(qualified_name, Arc::new(compiled_interface));
        Ok(())
    }

    pub(crate) fn compile_trait_internal(
        &mut self,
        name: &str,
        uses: &[String],
        properties: &[crate::ast::Property],
        methods: &[Method],
        attributes: &[Attribute],
    ) -> Result<(), String> {
        use crate::vm::class::{CompiledProperty, CompiledTrait};

        let mut compiled_trait = CompiledTrait::new(name.to_string());
        compiled_trait.uses = uses.to_vec();
        compiled_trait.attributes = attributes.to_vec();

        for prop in properties {
            let compiled_prop = CompiledProperty::from_ast(prop, false);
            compiled_trait.properties.push(compiled_prop);
        }

        for method in methods {
            let method_name = format!("{}::{}", name, method.name);
            let mut method_compiler = Compiler::new(method_name.clone());

            if !method.is_static {
                method_compiler.locals.insert("this".to_string(), 0);
                method_compiler
                    .function
                    .local_names
                    .push("this".to_string());
                method_compiler.next_local = 1;
            }

            let param_start = method_compiler.next_local;
            for (i, param) in method.params.iter().enumerate() {
                let slot = param_start + i as u16;
                method_compiler.locals.insert(param.name.clone(), slot);
                method_compiler
                    .function
                    .local_names
                    .push(param.name.clone());
            }
            method_compiler.next_local = param_start + method.params.len() as u16;
            method_compiler.function.local_count = method_compiler.next_local;
            method_compiler.function.param_count = method.params.len() as u8;
            method_compiler.function.required_param_count = method
                .params
                .iter()
                .filter(|p| p.default.is_none() && !p.is_variadic)
                .count() as u8;
            method_compiler.function.return_type = method.return_type.clone();

            method_compiler.function.parameters = method.params.clone();
            method_compiler.function.attributes = method.attributes.clone();

            for param in &method.params {
                method_compiler
                    .function
                    .param_types
                    .push(param.type_hint.clone());
            }

            for stmt in &method.body {
                method_compiler.compile_stmt(stmt)?;
            }

            method_compiler.emit(Opcode::ReturnNull);

            for (inner_name, inner_func) in method_compiler.functions.drain() {
                self.functions.insert(inner_name, inner_func);
            }

            let compiled = Arc::new(method_compiler.function);
            compiled_trait.methods.insert(method.name.clone(), compiled);
        }

        self.traits
            .insert(name.to_string(), Arc::new(compiled_trait));
        Ok(())
    }

    pub(crate) fn compile_enum_internal(
        &mut self,
        name: &str,
        backing_type: &crate::ast::EnumBackingType,
        cases: &[crate::ast::EnumCase],
        methods: &[Method],
        attributes: &[Attribute],
    ) -> Result<(), String> {
        use crate::ast::EnumBackingType;
        use crate::runtime::Value;
        use crate::vm::class::CompiledEnum;

        let mut compiled_enum = CompiledEnum::new(name.to_string(), *backing_type);
        compiled_enum.attributes = attributes.to_vec();

        let mut seen_values: std::collections::HashSet<String> = std::collections::HashSet::new();
        for case in cases {
            let backing_value = if let Some(expr) = &case.value {
                match expr {
                    Expr::Integer(n) => Some(Value::Integer(*n)),
                    Expr::Float(n) => Some(Value::Float(*n)),
                    Expr::String(s) => Some(Value::String(s.clone())),
                    _ => None,
                }
            } else {
                None
            };

            if let Some(ref val) = backing_value {
                let type_matches = matches!(
                    (backing_type, val),
                    (EnumBackingType::Int, Value::Integer(_))
                        | (EnumBackingType::String, Value::String(_))
                        | (EnumBackingType::None, _)
                );
                if !type_matches {
                    let expected_type = match backing_type {
                        EnumBackingType::Int => "int",
                        EnumBackingType::String => "string",
                        EnumBackingType::None => "none",
                    };
                    return Err(format!(
                        "Enum case '{}::{}' must have {} backing value",
                        name, case.name, expected_type
                    ));
                }

                let val_str = format!("{:?}", val);
                if !seen_values.insert(val_str) {
                    return Err("Duplicate case value in backed enum".to_string());
                }
            }

            compiled_enum.cases.insert(case.name.clone(), backing_value);
            compiled_enum.case_order.push(case.name.clone());
        }

        for method in methods {
            let method_name = format!("{}::{}", name, method.name);
            let mut method_compiler = Compiler::new(method_name.clone());

            if !method.is_static {
                method_compiler.locals.insert("this".to_string(), 0);
                method_compiler
                    .function
                    .local_names
                    .push("this".to_string());
                method_compiler.next_local = 1;
            }

            let param_start = method_compiler.next_local;
            for (i, param) in method.params.iter().enumerate() {
                let slot = param_start + i as u16;
                method_compiler.locals.insert(param.name.clone(), slot);
                method_compiler
                    .function
                    .local_names
                    .push(param.name.clone());
            }
            method_compiler.next_local = param_start + method.params.len() as u16;
            method_compiler.function.local_count = method_compiler.next_local;
            method_compiler.function.param_count = method.params.len() as u8;
            method_compiler.function.required_param_count = method
                .params
                .iter()
                .filter(|p| p.default.is_none() && !p.is_variadic)
                .count() as u8;
            method_compiler.function.return_type = method.return_type.clone();

            method_compiler.function.parameters = method.params.clone();
            method_compiler.function.attributes = method.attributes.clone();

            for param in &method.params {
                method_compiler
                    .function
                    .param_types
                    .push(param.type_hint.clone());
            }

            for stmt in &method.body {
                method_compiler.compile_stmt(stmt)?;
            }

            method_compiler.emit(Opcode::ReturnNull);

            for (inner_name, inner_func) in method_compiler.functions.drain() {
                self.functions.insert(inner_name, inner_func);
            }

            let compiled = Arc::new(method_compiler.function);
            compiled_enum.methods.insert(method.name.clone(), compiled);
        }

        // Add built-in static methods for backed enums: from() and tryFrom()
        if *backing_type != EnumBackingType::None {
            // Compile from() method (throws if not found)
            let from_method_name = format!("{}::from", name);
            let mut from_compiler = Compiler::new(from_method_name);

            // Parameter: the value to search for
            from_compiler.locals.insert("value".to_string(), 0);
            from_compiler.function.local_names.push("value".to_string());
            from_compiler.next_local = 1;
            from_compiler.function.local_count = 1;
            from_compiler.function.param_count = 1;
            from_compiler.function.required_param_count = 1;

            // Load the parameter
            from_compiler.emit(Opcode::LoadFast(0));

            // Add enum name to string pool and emit EnumFromValue opcode
            let enum_name_idx = from_compiler.intern_string(name.to_string());
            from_compiler.emit(Opcode::EnumFromValue(enum_name_idx));

            from_compiler.emit(Opcode::Return);

            for (inner_name, inner_func) in from_compiler.functions.drain() {
                self.functions.insert(inner_name, inner_func);
            }
            let from_compiled = Arc::new(from_compiler.function);
            compiled_enum
                .static_methods
                .insert("from".to_string(), from_compiled);

            // Compile tryFrom() method (returns null if not found)
            let try_from_method_name = format!("{}::tryFrom", name);
            let mut try_from_compiler = Compiler::new(try_from_method_name);

            // Parameter: the value to search for
            try_from_compiler.locals.insert("value".to_string(), 0);
            try_from_compiler
                .function
                .local_names
                .push("value".to_string());
            try_from_compiler.next_local = 1;
            try_from_compiler.function.local_count = 1;
            try_from_compiler.function.param_count = 1;
            try_from_compiler.function.required_param_count = 1;

            // Load the parameter
            try_from_compiler.emit(Opcode::LoadFast(0));

            // Add enum name to string pool and emit EnumTryFromValue opcode
            let enum_name_idx = try_from_compiler.intern_string(name.to_string());
            try_from_compiler.emit(Opcode::EnumTryFromValue(enum_name_idx));

            try_from_compiler.emit(Opcode::Return);

            for (inner_name, inner_func) in try_from_compiler.functions.drain() {
                self.functions.insert(inner_name, inner_func);
            }
            let try_from_compiled = Arc::new(try_from_compiler.function);
            compiled_enum
                .static_methods
                .insert("tryFrom".to_string(), try_from_compiled);
        }

        self.enums.insert(name.to_string(), Arc::new(compiled_enum));
        Ok(())
    }
}
