use crate::runtime::Value;

fn normalize_class_name(name: &str) -> String {
    if let Some(stripped) = name.strip_prefix('\\') {
        stripped.to_string()
    } else {
        name.to_string()
    }
}

pub fn execute_new_object<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    class_name: String,
) -> Result<(), String> {
    let class_name = normalize_class_name(&class_name);
    let class_def = vm
        .classes
        .get(&class_name)
        .ok_or_else(|| format!("Class '{}' not found", class_name))?
        .clone();

    if class_def.is_abstract {
        return Err(format!("Cannot instantiate abstract class {}", class_name));
    }

    let mut instance = crate::runtime::ObjectInstance::with_hierarchy(
        class_name.clone(),
        class_def.parent.clone(),
        class_def.interfaces.clone(),
    );

    let mut parent_chain = Vec::new();
    let mut current_parent = class_def.parent.as_ref();
    while let Some(parent_name) = current_parent {
        if let Some(parent_def) = vm.classes.get(parent_name) {
            parent_chain.push(parent_def.clone());
            current_parent = parent_def.parent.as_ref();
        } else {
            break;
        }
    }

    for parent_def in parent_chain.iter().rev() {
        for prop in &parent_def.properties {
            let default_val = prop.default.clone().unwrap_or(Value::Null);
            instance
                .properties
                .insert(prop.name.clone(), default_val.clone());
            if prop.readonly {
                instance.readonly_properties.insert(prop.name.clone());
                if prop.default.is_some() {
                    instance.initialized_readonly.insert(prop.name.clone());
                }
            }
        }
    }

    for prop in &class_def.properties {
        let default_val = prop.default.clone().unwrap_or(Value::Null);
        instance
            .properties
            .insert(prop.name.clone(), default_val.clone());
        if prop.readonly {
            instance.readonly_properties.insert(prop.name.clone());
            if prop.default.is_some() {
                instance.initialized_readonly.insert(prop.name.clone());
            }
        }
    }

    vm.stack.push(Value::Object(instance));
    Ok(())
}

pub fn execute_instance_of<W: std::io::Write>(vm: &mut super::super::VM<W>, class_name: String) {
    let class_name = normalize_class_name(&class_name);
    let object = vm.stack.pop().unwrap();

    let result = match object {
        Value::Object(instance) => {
            instance.class_name == class_name
                || instance.parent_class.as_ref() == Some(&class_name)
                || instance.interfaces.contains(&class_name)
        }
        _ => false,
    };
    vm.stack.push(Value::Bool(result));
}

pub fn execute_clone<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let object = vm.stack.pop().ok_or("Stack underflow")?;
    match object {
        Value::Object(instance) => {
            let cloned = instance.clone();
            vm.stack.push(Value::Object(cloned));
        }
        _ => return Err("__clone method called on non-object".to_string()),
    }
    Ok(())
}

pub fn execute_load_enum_case<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    enum_name: String,
    case_name: String,
) -> Result<(), String> {
    let enum_name = normalize_class_name(&enum_name);
    let enum_def = vm
        .enums
        .get(&enum_name)
        .ok_or_else(|| format!("Enum '{}' not found", enum_name))?
        .clone();

    let backing_value = enum_def
        .cases
        .get(&case_name)
        .ok_or_else(|| format!("Undefined case '{}' for enum '{}'", case_name, enum_name))?
        .clone()
        .map(Box::new);

    vm.stack.push(Value::EnumCase {
        enum_name,
        case_name,
        backing_value,
    });
    Ok(())
}

pub fn execute_load_this<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let frame = vm.current_frame();
    let this = frame
        .locals
        .first()
        .cloned()
        .ok_or("No $this available in current context")?;
    vm.stack.push(this);
    Ok(())
}
