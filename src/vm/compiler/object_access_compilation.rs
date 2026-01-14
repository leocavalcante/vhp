use super::Compiler;

use crate::ast::{Argument, Expr};
use crate::vm::class::{CompiledClass, CompiledProperty};
use crate::vm::opcode::Opcode;
use std::sync::Arc;

impl Compiler {
    pub(crate) fn compile_new_object(
        &mut self,
        class_name: &str,
        args: &[Argument],
    ) -> Result<(), String> {
        let qualified_name = self.qualify_class_name(class_name);
        let class_idx = self.intern_string(qualified_name);
        self.emit(Opcode::NewObject(class_idx));

        let has_named = args.iter().any(|arg| arg.name.is_some());

        if has_named {
            for (idx, arg) in args.iter().enumerate() {
                if let Some(ref param_name) = arg.name {
                    let name_idx = self.intern_string(param_name.clone());
                    self.emit(Opcode::PushString(name_idx));
                    self.compile_expr(&arg.value)?;
                } else {
                    self.emit(Opcode::PushInt(idx as i64));
                    self.compile_expr(&arg.value)?;
                }
            }
            self.emit(Opcode::NewArray(args.len() as u16));
            self.emit(Opcode::CallConstructorNamed);
        } else {
            for arg in args {
                self.compile_expr(&arg.value)?;
            }
            self.emit(Opcode::CallConstructor(args.len() as u8));
        }

        Ok(())
    }

    pub(crate) fn compile_property_access(
        &mut self,
        object: &Expr,
        property: &str,
    ) -> Result<(), String> {
        self.compile_expr(object)?;
        let prop_idx = self.intern_string(property.to_string());
        self.emit(Opcode::LoadProperty(prop_idx));
        Ok(())
    }

    pub(crate) fn compile_property_assign(
        &mut self,
        object: &Expr,
        property: &str,
        value: &Expr,
    ) -> Result<(), String> {
        if matches!(object, Expr::This) {
            self.compile_expr(value)?;
            let prop_idx = self.intern_string(property.to_string());
            self.emit(Opcode::StoreThisProperty(prop_idx));
        } else if let Expr::Variable(var_name) = object {
            self.compile_expr(object)?;
            self.compile_expr(value)?;
            let prop_idx = self.intern_string(property.to_string());
            self.emit(Opcode::StoreProperty(prop_idx));
            if let Some(&slot) = self.locals.get(var_name) {
                self.emit(Opcode::StoreFast(slot));
            } else {
                let var_idx = self.intern_string(var_name.clone());
                self.emit(Opcode::StoreVar(var_idx));
            }
        } else {
            self.compile_expr(object)?;
            self.compile_expr(value)?;
            let prop_idx = self.intern_string(property.to_string());
            self.emit(Opcode::StoreProperty(prop_idx));
        }
        Ok(())
    }

    pub(crate) fn compile_method_call(
        &mut self,
        object: &Expr,
        method: &str,
        args: &[Argument],
    ) -> Result<(), String> {
        let method_idx = self.intern_string(method.to_string());

        match object {
            Expr::Variable(var_name) => {
                for arg in args {
                    self.compile_expr(&arg.value)?;
                }

                if let Some(&slot) = self.locals.get(var_name) {
                    self.emit(Opcode::CallMethodOnLocal(
                        slot,
                        method_idx,
                        args.len() as u8,
                    ));
                } else {
                    let var_idx = self.intern_string(var_name.clone());
                    self.emit(Opcode::CallMethodOnGlobal(
                        var_idx,
                        method_idx,
                        args.len() as u8,
                    ));
                }
            }
            _ => {
                self.compile_expr(object)?;
                for arg in args {
                    self.compile_expr(&arg.value)?;
                }
                self.emit(Opcode::CallMethod(method_idx, args.len() as u8));
            }
        }

        Ok(())
    }

    pub(crate) fn compile_static_method_call(
        &mut self,
        class_name: &str,
        method: &str,
        args: &[Argument],
    ) -> Result<(), String> {
        let has_named = args.iter().any(|arg| arg.name.is_some());

        if has_named {
            for (idx, arg) in args.iter().enumerate() {
                if let Some(ref param_name) = arg.name {
                    let name_idx = self.intern_string(param_name.clone());
                    self.emit(Opcode::PushString(name_idx));
                    self.compile_expr(&arg.value)?;
                } else {
                    self.emit(Opcode::PushInt(idx as i64));
                    self.compile_expr(&arg.value)?;
                }
            }
            self.emit(Opcode::NewArray(args.len() as u16));
            let class_idx = self.intern_string(class_name.to_string());
            let method_idx = self.intern_string(method.to_string());
            self.emit(Opcode::CallStaticMethodNamed(class_idx, method_idx));
        } else {
            for arg in args {
                self.compile_expr(&arg.value)?;
            }
            let class_idx = self.intern_string(class_name.to_string());
            let method_idx = self.intern_string(method.to_string());
            self.emit(Opcode::CallStaticMethod(
                class_idx,
                method_idx,
                args.len() as u8,
            ));
        }

        Ok(())
    }

    pub(crate) fn compile_static_property_access(
        &mut self,
        class: &str,
        property: &str,
    ) -> Result<(), String> {
        let class_idx = self.intern_string(class.to_string());
        let prop_idx = self.intern_string(property.to_string());
        self.emit(Opcode::LoadStaticProp(class_idx, prop_idx));
        Ok(())
    }

    pub(crate) fn compile_static_property_assign(
        &mut self,
        class: &str,
        property: &str,
        value: &Expr,
    ) -> Result<(), String> {
        self.compile_expr(value)?;
        let class_idx = self.intern_string(class.to_string());
        let prop_idx = self.intern_string(property.to_string());
        self.emit(Opcode::StoreStaticProp(class_idx, prop_idx));
        Ok(())
    }

    pub(crate) fn compile_anonymous_class(
        &mut self,
        constructor_args: &[Argument],
        parent: &Option<String>,
        properties: &[crate::ast::Property],
        methods: &[crate::ast::Method],
    ) -> Result<(), String> {
        let anon_name = format!("__anon_class_{}", self.classes.len());
        let mut anon_class = CompiledClass::new(anon_name.clone());

        anon_class.parent = parent.clone();

        for prop in properties {
            let compiled_prop = CompiledProperty::from_ast(prop, false);
            anon_class.properties.push(compiled_prop);
        }

        for method in methods {
            let method_name = format!("{}::{}", anon_name, method.name);
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
            anon_class.methods.insert(method.name.clone(), compiled);
        }

        self.classes.insert(anon_name.clone(), Arc::new(anon_class));

        let class_idx = self.intern_string(anon_name);
        self.emit(Opcode::NewObject(class_idx));

        let arg_count = constructor_args.len();
        for arg in constructor_args {
            self.compile_expr(&arg.value)?;
        }

        self.emit(Opcode::CallConstructor(arg_count as u8));

        Ok(())
    }

    pub(crate) fn compile_clone_with(
        &mut self,
        object: &Expr,
        modifications: &[crate::ast::PropertyModification],
    ) -> Result<(), String> {
        self.compile_expr(object)?;
        self.emit(Opcode::Clone);

        for modification in modifications {
            self.compile_expr(&modification.value)?;
            let prop_idx = self.intern_string(modification.property.clone());
            self.emit(Opcode::StoreCloneProperty(prop_idx));
        }

        Ok(())
    }
}
