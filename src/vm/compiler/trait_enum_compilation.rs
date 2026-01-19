use super::Compiler;

use crate::ast::{Attribute, Expr, Method};
use crate::vm::opcode::Opcode;
use std::sync::Arc;

impl Compiler {
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

            // Set trait context for __TRAIT__ magic constant
            method_compiler.current_trait = Some(name.to_string());

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
            method_compiler.function.return_type = method
                .return_type
                .as_ref()
                .map(|t| method_compiler.resolve_type_hint(t));

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
            method_compiler.function.return_type = method
                .return_type
                .as_ref()
                .map(|t| method_compiler.resolve_type_hint(t));

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
