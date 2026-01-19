use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ExceptionValue {
    pub class_name: String,
    pub message: String,
    pub code: i64,
    pub previous: Option<Box<ExceptionValue>>,
}

#[derive(Debug, Clone)]
pub struct ObjectInstance {
    pub class_name: String,
    pub properties: HashMap<String, super::Value>,
    pub readonly_properties: std::collections::HashSet<String>,
    pub initialized_readonly: std::collections::HashSet<String>,
    pub parent_class: Option<String>,
    pub interfaces: Vec<String>,
}

impl ObjectInstance {
    #[allow(dead_code)]
    pub fn new(class_name: String) -> Self {
        Self {
            class_name,
            properties: HashMap::new(),
            readonly_properties: std::collections::HashSet::new(),
            initialized_readonly: std::collections::HashSet::new(),
            parent_class: None,
            interfaces: Vec::new(),
        }
    }

    pub fn with_hierarchy(
        class_name: String,
        parent: Option<String>,
        interfaces: Vec<String>,
    ) -> Self {
        Self {
            class_name,
            properties: HashMap::new(),
            readonly_properties: std::collections::HashSet::new(),
            initialized_readonly: std::collections::HashSet::new(),
            parent_class: parent,
            interfaces,
        }
    }

    pub fn is_instance_of(&self, class_name: &str) -> bool {
        if self.class_name.eq_ignore_ascii_case(class_name) {
            return true;
        }
        if let Some(ref parent) = self.parent_class {
            if parent.eq_ignore_ascii_case(class_name) {
                return true;
            }
        }
        self.interfaces
            .iter()
            .any(|iface| iface.eq_ignore_ascii_case(class_name))
    }
}
