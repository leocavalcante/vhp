use super::Compiler;

impl Compiler {
    /// Compile a statement (internal implementation)
    pub(crate) fn compile_stmt_internal(&mut self, stmt: &crate::ast::Stmt) -> Result<(), String> {
        match stmt {
            crate::ast::Stmt::Echo(exprs) => {
                for expr in exprs {
                    self.compile_expr(expr)?;
                    self.emit(crate::vm::opcode::Opcode::Echo);
                }
            }
            crate::ast::Stmt::Expression(expr) => {
                self.compile_expr(expr)?;
                self.emit(crate::vm::opcode::Opcode::Pop);
            }
            crate::ast::Stmt::Return(expr) => {
                if let Some(expr) = expr {
                    self.compile_expr(expr)?;
                    self.emit(crate::vm::opcode::Opcode::Return);
                } else {
                    self.emit(crate::vm::opcode::Opcode::ReturnNull);
                }
            }
            crate::ast::Stmt::If {
                condition,
                then_branch,
                elseif_branches,
                else_branch,
            } => {
                self.compile_if(condition, then_branch, elseif_branches, else_branch)?;
            }
            crate::ast::Stmt::While { condition, body } => {
                self.compile_while(condition, body)?;
            }
            crate::ast::Stmt::DoWhile { body, condition } => {
                self.compile_do_while(body, condition)?;
            }
            crate::ast::Stmt::For {
                init,
                condition,
                update,
                body,
            } => {
                self.compile_for(init, condition, update, body)?;
            }
            crate::ast::Stmt::Foreach {
                array,
                key,
                value,
                body,
            } => {
                self.compile_foreach(array, key, value, body)?;
            }
            crate::ast::Stmt::Break => {
                self.emit(crate::vm::opcode::Opcode::Break);
            }
            crate::ast::Stmt::Continue => {
                self.emit(crate::vm::opcode::Opcode::Continue);
            }
            crate::ast::Stmt::Function {
                name,
                params,
                return_type,
                body,
                attributes,
            } => {
                self.compile_function(name, params, return_type, body, attributes)?;
            }
            crate::ast::Stmt::Switch {
                expr,
                cases,
                default,
            } => {
                self.compile_switch(expr, cases, default)?;
            }
            crate::ast::Stmt::Html(content) => {
                let idx = self.intern_string(content.clone());
                self.emit(crate::vm::opcode::Opcode::PushString(idx));
                self.emit(crate::vm::opcode::Opcode::Echo);
            }
            crate::ast::Stmt::Declare { directives, body } => {
                for directive in directives {
                    if let crate::ast::DeclareDirective::StrictTypes(enabled) = directive {
                        self.strict_types = *enabled;
                    }
                }
                if let Some(stmts) = body {
                    for stmt in stmts {
                        self.compile_stmt(stmt)?;
                    }
                }
            }
            crate::ast::Stmt::Namespace { name, body } => {
                let prev_namespace = self.current_namespace.clone();
                let prev_use_aliases = self.use_aliases.clone();

                self.current_namespace = name.as_ref().map(|n| n.parts.join("\\"));
                self.use_aliases.clear();

                match body {
                    crate::ast::NamespaceBody::Braced(stmts) => {
                        for stmt in stmts {
                            self.compile_stmt(stmt)?;
                        }
                        self.current_namespace = prev_namespace;
                        self.use_aliases = prev_use_aliases;
                    }
                    crate::ast::NamespaceBody::Unbraced => {
                        // For unbraced namespaces, the namespace applies to subsequent statements
                    }
                }
            }
            crate::ast::Stmt::Use(use_clauses) => {
                for clause in use_clauses {
                    let full_name = clause.name.parts.join("\\");
                    let alias = clause
                        .alias
                        .clone()
                        .unwrap_or_else(|| clause.name.last().cloned().unwrap_or_default());
                    self.use_aliases.insert(alias, full_name);
                }
            }
            crate::ast::Stmt::GroupUse(group_use) => {
                let prefix = group_use.prefix.parts.join("\\");
                for clause in &group_use.items {
                    let full_name = if prefix.is_empty() {
                        clause.name.parts.join("\\")
                    } else {
                        format!("{}\\{}", prefix, clause.name.parts.join("\\"))
                    };
                    let alias = clause
                        .alias
                        .clone()
                        .unwrap_or_else(|| clause.name.last().cloned().unwrap_or_default());
                    self.use_aliases.insert(alias, full_name);
                }
            }
            crate::ast::Stmt::Throw(expr) => {
                self.compile_expr(expr)?;
                self.emit(crate::vm::opcode::Opcode::Throw);
            }
            crate::ast::Stmt::TryCatch {
                try_body,
                catch_clauses,
                finally_body,
            } => {
                self.compile_try_catch(try_body, catch_clauses, finally_body)?;
            }
            crate::ast::Stmt::Class {
                name,
                is_abstract,
                is_final,
                readonly,
                parent,
                interfaces,
                trait_uses,
                properties,
                methods,
                attributes,
            } => {
                self.compile_class(
                    name,
                    *is_abstract,
                    *is_final,
                    *readonly,
                    parent,
                    interfaces,
                    trait_uses,
                    properties,
                    methods,
                    attributes,
                )?;
            }
            crate::ast::Stmt::Interface {
                name,
                parents,
                methods,
                constants,
                attributes,
            } => {
                self.compile_interface(name, parents, methods, constants, attributes)?;
            }
            crate::ast::Stmt::Trait {
                name,
                uses,
                properties,
                methods,
                attributes,
            } => {
                self.compile_trait(name, uses, properties, methods, attributes)?;
            }
            crate::ast::Stmt::Enum {
                name,
                backing_type,
                cases,
                methods,
                attributes,
            } => {
                self.compile_enum(name, backing_type, cases, methods, attributes)?;
            }
        }
        Ok(())
    }
}
