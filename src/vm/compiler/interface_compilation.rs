use super::Compiler;

use crate::ast::{Attribute, QualifiedName};
use std::sync::Arc;

impl Compiler {
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
}
