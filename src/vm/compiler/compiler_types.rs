//! Type and name resolution utilities for the compiler

use super::Compiler;
use crate::ast::{QualifiedName, TypeHint};

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

    /// Resolve a TypeHint to fully qualified class names
    /// Simple type names like "User" are converted to Class with qualified name
    pub fn resolve_type_hint(&self, type_hint: &TypeHint) -> TypeHint {
        match type_hint {
            TypeHint::Simple(name) => {
                let scalar_types = [
                    "int", "string", "float", "bool", "array", "object", "callable", "mixed",
                    "null", "iterable", "false", "true", "void", "never",
                ];
                if scalar_types.contains(&name.as_str()) {
                    TypeHint::Simple(name.clone())
                } else {
                    TypeHint::Class(self.qualify_class_name(name))
                }
            }
            TypeHint::Nullable(inner) => {
                TypeHint::Nullable(Box::new(self.resolve_type_hint(inner)))
            }
            TypeHint::Union(types) => {
                TypeHint::Union(types.iter().map(|t| self.resolve_type_hint(t)).collect())
            }
            TypeHint::Intersection(types) => {
                TypeHint::Intersection(types.iter().map(|t| self.resolve_type_hint(t)).collect())
            }
            TypeHint::DNF(groups) => TypeHint::DNF(
                groups
                    .iter()
                    .map(|group| group.iter().map(|t| self.resolve_type_hint(t)).collect())
                    .collect(),
            ),
            TypeHint::Class(name) => {
                if name.starts_with('\\') {
                    TypeHint::Class(name.strip_prefix('\\').unwrap().to_string())
                } else {
                    TypeHint::Class(self.qualify_class_name(name))
                }
            }
            TypeHint::Void => TypeHint::Void,
            TypeHint::Never => TypeHint::Never,
            TypeHint::Static => TypeHint::Static,
            TypeHint::SelfType => TypeHint::SelfType,
            TypeHint::ParentType => TypeHint::ParentType,
        }
    }
}
