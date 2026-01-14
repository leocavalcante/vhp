//! Type and name resolution utilities for the compiler

use super::Compiler;
use crate::ast::QualifiedName;

impl Compiler {
    /// Resolve a QualifiedName to a fully qualified class name string
    pub fn resolve_qualified_name(&self, qname: &QualifiedName) -> String {
        if qname.is_fully_qualified {
            qname.parts.join("\\")
        } else {
            let name = qname.parts.join("\\");
            self.qualify_class_name(&name)
        }
    }

    /// Qualify a class name with the current namespace if needed
    pub fn qualify_class_name(&self, name: &str) -> String {
        if let Some(stripped) = name.strip_prefix('\\') {
            stripped.to_string()
        } else if name.contains('\\') {
            let parts: Vec<&str> = name.splitn(2, '\\').collect();
            let first_segment = parts[0];
            let rest = parts.get(1).unwrap_or(&"");

            if let Some(aliased) = self.use_aliases.get(first_segment) {
                let aliased_str: String = aliased.clone();
                if rest.is_empty() {
                    aliased_str
                } else {
                    format!("{}\\{}", aliased_str, rest)
                }
            } else if let Some(ref ns) = self.current_namespace {
                format!("{}\\{}", ns, name)
            } else {
                name.to_string()
            }
        } else if let Some(aliased) = self.use_aliases.get(name) {
            let aliased_str: String = aliased.clone();
            aliased_str
        } else if let Some(ref ns) = self.current_namespace {
            format!("{}\\{}", ns, name)
        } else {
            name.to_string()
        }
    }
}
