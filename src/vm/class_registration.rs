use crate::runtime::Value;
use crate::vm::class::CompiledClass;
use crate::vm::class::CompiledProperty;
use crate::vm::opcode::Opcode;
use std::collections::HashMap;
use std::sync::Arc;

pub fn register_builtin_classes(classes: &mut HashMap<String, Arc<CompiledClass>>) {
    register_exception_class(classes);
    register_error_class(classes);
    register_type_error(classes);
    register_invalid_argument_exception(classes);
    register_unhandled_match_error(classes);
    register_fiber_class(classes);
}

fn register_exception_class(classes: &mut std::collections::HashMap<String, Arc<CompiledClass>>) {
    let mut exception = CompiledClass::new("Exception".to_string());

    exception.properties.push(CompiledProperty {
        name: "message".to_string(),
        visibility: crate::ast::Visibility::Private,
        write_visibility: None,
        default: Some(Value::String(String::new())),
        readonly: false,
        is_static: false,
        type_hint: None,
        attributes: Vec::new(),
        get_hook: None,
        set_hook: None,
    });

    exception.properties.push(CompiledProperty {
        name: "code".to_string(),
        visibility: crate::ast::Visibility::Private,
        write_visibility: None,
        default: Some(Value::Integer(0)),
        readonly: false,
        is_static: false,
        type_hint: None,
        attributes: Vec::new(),
        get_hook: None,
        set_hook: None,
    });

    let mut construct = CompiledFunction::new("Exception::__construct".to_string());
    construct.param_count = 2;
    construct.required_param_count = 0;
    construct.local_count = 3;
    construct.local_names = vec![
        "this".to_string(),
        "message".to_string(),
        "code".to_string(),
    ];
    construct.bytecode.push(Opcode::LoadFast(1));
    construct.strings.push("message".to_string());
    construct.bytecode.push(Opcode::StoreThisProperty(0));
    construct.bytecode.push(Opcode::LoadFast(2));
    construct.strings.push("code".to_string());
    construct.bytecode.push(Opcode::StoreThisProperty(1));
    construct.bytecode.push(Opcode::ReturnNull);
    exception
        .methods
        .insert("__construct".to_string(), Arc::new(construct));

    let mut get_message = CompiledFunction::new("Exception::getMessage".to_string());
    get_message.param_count = 0;
    get_message.local_count = 1;
    get_message.local_names = vec!["this".to_string()];
    get_message.strings.push("message".to_string());
    get_message.bytecode.push(Opcode::LoadThis);
    get_message.bytecode.push(Opcode::LoadProperty(0));
    get_message.bytecode.push(Opcode::Return);
    exception
        .methods
        .insert("getMessage".to_string(), Arc::new(get_message));

    let mut get_code = CompiledFunction::new("Exception::getCode".to_string());
    get_code.param_count = 0;
    get_code.local_count = 1;
    get_code.local_names = vec!["this".to_string()];
    get_code.strings.push("code".to_string());
    get_code.bytecode.push(Opcode::LoadThis);
    get_code.bytecode.push(Opcode::LoadProperty(0));
    get_code.bytecode.push(Opcode::Return);
    exception
        .methods
        .insert("getCode".to_string(), Arc::new(get_code));

    classes.insert("Exception".to_string(), Arc::new(exception));
}

fn register_error_class(classes: &mut std::collections::HashMap<String, Arc<CompiledClass>>) {
    let mut error = CompiledClass::new("Error".to_string());

    error.properties.push(CompiledProperty {
        name: "message".to_string(),
        visibility: crate::ast::Visibility::Private,
        write_visibility: None,
        default: Some(Value::String(String::new())),
        readonly: false,
        is_static: false,
        type_hint: None,
        attributes: Vec::new(),
        get_hook: None,
        set_hook: None,
    });

    error.properties.push(CompiledProperty {
        name: "code".to_string(),
        visibility: crate::ast::Visibility::Private,
        write_visibility: None,
        default: Some(Value::Integer(0)),
        readonly: false,
        is_static: false,
        type_hint: None,
        attributes: Vec::new(),
        get_hook: None,
        set_hook: None,
    });

    let mut error_construct = CompiledFunction::new("Error::__construct".to_string());
    error_construct.param_count = 2;
    error_construct.required_param_count = 0;
    error_construct.local_count = 3;
    error_construct.local_names = vec![
        "this".to_string(),
        "message".to_string(),
        "code".to_string(),
    ];
    error_construct.bytecode.push(Opcode::LoadFast(1));
    error_construct.strings.push("message".to_string());
    error_construct.bytecode.push(Opcode::StoreThisProperty(0));
    error_construct.bytecode.push(Opcode::LoadFast(2));
    error_construct.strings.push("code".to_string());
    error_construct.bytecode.push(Opcode::StoreThisProperty(1));
    error_construct.bytecode.push(Opcode::ReturnNull);
    error
        .methods
        .insert("__construct".to_string(), Arc::new(error_construct));

    let mut error_get_message = CompiledFunction::new("Error::getMessage".to_string());
    error_get_message.param_count = 0;
    error_get_message.local_count = 1;
    error_get_message.local_names = vec!["this".to_string()];
    error_get_message.strings.push("message".to_string());
    error_get_message.bytecode.push(Opcode::LoadThis);
    error_get_message.bytecode.push(Opcode::LoadProperty(0));
    error_get_message.bytecode.push(Opcode::Return);
    error
        .methods
        .insert("getMessage".to_string(), Arc::new(error_get_message));

    let mut error_get_code = CompiledFunction::new("Error::getCode".to_string());
    error_get_code.param_count = 0;
    error_get_code.local_count = 1;
    error_get_code.local_names = vec!["this".to_string()];
    error_get_code.strings.push("code".to_string());
    error_get_code.bytecode.push(Opcode::LoadThis);
    error_get_code.bytecode.push(Opcode::LoadProperty(0));
    error_get_code.bytecode.push(Opcode::Return);
    error
        .methods
        .insert("getCode".to_string(), Arc::new(error_get_code));

    classes.insert("Error".to_string(), Arc::new(error));
}

fn register_type_error(classes: &mut std::collections::HashMap<String, Arc<CompiledClass>>) {
    let mut type_error = CompiledClass::new("TypeError".to_string());
    type_error.parent = Some("Error".to_string());
    classes.insert("TypeError".to_string(), Arc::new(type_error));
}

fn register_invalid_argument_exception(
    classes: &mut std::collections::HashMap<String, Arc<CompiledClass>>,
) {
    let mut invalid_arg = CompiledClass::new("InvalidArgumentException".to_string());
    invalid_arg.parent = Some("Exception".to_string());
    classes.insert(
        "InvalidArgumentException".to_string(),
        Arc::new(invalid_arg),
    );
}

fn register_unhandled_match_error(
    classes: &mut std::collections::HashMap<String, Arc<CompiledClass>>,
) {
    let mut unhandled_match = CompiledClass::new("UnhandledMatchError".to_string());
    unhandled_match.parent = Some("Error".to_string());
    classes.insert("UnhandledMatchError".to_string(), Arc::new(unhandled_match));
}

fn register_fiber_class(classes: &mut std::collections::HashMap<String, Arc<CompiledClass>>) {
    let mut fiber = CompiledClass::new("Fiber".to_string());

    fiber.properties.push(CompiledProperty {
        name: "__callback".to_string(),
        visibility: crate::ast::Visibility::Private,
        write_visibility: None,
        default: Some(Value::Null),
        readonly: false,
        is_static: false,
        type_hint: None,
        attributes: Vec::new(),
        get_hook: None,
        set_hook: None,
    });

    fiber.properties.push(CompiledProperty {
        name: "__started".to_string(),
        visibility: crate::ast::Visibility::Private,
        write_visibility: None,
        default: Some(Value::Bool(false)),
        readonly: false,
        is_static: false,
        type_hint: None,
        attributes: Vec::new(),
        get_hook: None,
        set_hook: None,
    });

    fiber.properties.push(CompiledProperty {
        name: "__suspended".to_string(),
        visibility: crate::ast::Visibility::Private,
        write_visibility: None,
        default: Some(Value::Bool(false)),
        readonly: false,
        is_static: false,
        type_hint: None,
        attributes: Vec::new(),
        get_hook: None,
        set_hook: None,
    });

    fiber.properties.push(CompiledProperty {
        name: "__terminated".to_string(),
        visibility: crate::ast::Visibility::Private,
        write_visibility: None,
        default: Some(Value::Bool(false)),
        readonly: false,
        is_static: false,
        type_hint: None,
        attributes: Vec::new(),
        get_hook: None,
        set_hook: None,
    });

    fiber.properties.push(CompiledProperty {
        name: "__return_value".to_string(),
        visibility: crate::ast::Visibility::Private,
        write_visibility: None,
        default: Some(Value::Null),
        readonly: false,
        is_static: false,
        type_hint: None,
        attributes: Vec::new(),
        get_hook: None,
        set_hook: None,
    });

    let mut construct = CompiledFunction::new("Fiber::__construct".to_string());
    construct.param_count = 1;
    construct.required_param_count = 1;
    construct.local_count = 2;
    construct.local_names = vec!["this".to_string(), "callback".to_string()];
    construct.bytecode.push(Opcode::LoadFast(1));
    construct.strings.push("__callback".to_string());
    construct.bytecode.push(Opcode::StoreThisProperty(0));
    construct.bytecode.push(Opcode::ReturnNull);
    fiber
        .methods
        .insert("__construct".to_string(), Arc::new(construct));

    let mut start = CompiledFunction::new("Fiber::start".to_string());
    start.param_count = 0;
    start.local_count = 1;
    start.local_names = vec!["this".to_string()];
    start.strings.push("__started".to_string());
    start.bytecode.push(Opcode::PushTrue);
    start.bytecode.push(Opcode::LoadThis);
    start.bytecode.push(Opcode::StoreThisProperty(0));
    start.strings.push("__terminated".to_string());
    start.bytecode.push(Opcode::PushTrue);
    start.bytecode.push(Opcode::LoadThis);
    start.bytecode.push(Opcode::StoreThisProperty(1));
    start.strings.push("__callback".to_string());
    start.bytecode.push(Opcode::LoadThis);
    start.bytecode.push(Opcode::LoadProperty(2));
    // Set current fiber before calling callback
    start.bytecode.push(Opcode::Dup);
    start.bytecode.push(Opcode::SetCurrentFiber);
    start.bytecode.push(Opcode::CallCallable(0));
    start.bytecode.push(Opcode::Dup);
    start.bytecode.push(Opcode::LoadFast(0));
    start.bytecode.push(Opcode::Swap);
    start.strings.push("__return_value".to_string());
    start.bytecode.push(Opcode::StoreThisProperty(3));
    start.bytecode.push(Opcode::LoadFast(0));
    start.bytecode.push(Opcode::LoadProperty(3));
    start.bytecode.push(Opcode::Return);
    fiber.methods.insert("start".to_string(), Arc::new(start));

    let mut get_return = CompiledFunction::new("Fiber::getReturn".to_string());
    get_return.param_count = 0;
    get_return.local_count = 1;
    get_return.local_names = vec!["this".to_string()];
    get_return.strings.push("__return_value".to_string());
    get_return.bytecode.push(Opcode::LoadThis);
    get_return.bytecode.push(Opcode::LoadProperty(0));
    get_return.bytecode.push(Opcode::Return);
    fiber
        .methods
        .insert("getReturn".to_string(), Arc::new(get_return));

    let mut is_started = CompiledFunction::new("Fiber::isStarted".to_string());
    is_started.param_count = 0;
    is_started.local_count = 1;
    is_started.local_names = vec!["this".to_string()];
    is_started.strings.push("__started".to_string());
    is_started.bytecode.push(Opcode::LoadThis);
    is_started.bytecode.push(Opcode::LoadProperty(0));
    is_started.bytecode.push(Opcode::Return);
    fiber
        .methods
        .insert("isStarted".to_string(), Arc::new(is_started));

    let mut is_suspended = CompiledFunction::new("Fiber::isSuspended".to_string());
    is_suspended.param_count = 0;
    is_suspended.local_count = 1;
    is_suspended.local_names = vec!["this".to_string()];
    is_suspended.strings.push("__suspended".to_string());
    is_suspended.bytecode.push(Opcode::LoadThis);
    is_suspended.bytecode.push(Opcode::LoadProperty(0));
    is_suspended.bytecode.push(Opcode::Return);
    fiber
        .methods
        .insert("isSuspended".to_string(), Arc::new(is_suspended));

    let mut is_terminated = CompiledFunction::new("Fiber::isTerminated".to_string());
    is_terminated.param_count = 0;
    is_terminated.local_count = 1;
    is_terminated.local_names = vec!["this".to_string()];
    is_terminated.strings.push("__terminated".to_string());
    is_terminated.bytecode.push(Opcode::LoadThis);
    is_terminated.bytecode.push(Opcode::LoadProperty(0));
    is_terminated.bytecode.push(Opcode::Return);
    fiber
        .methods
        .insert("isTerminated".to_string(), Arc::new(is_terminated));

    let mut get_current = CompiledFunction::new("Fiber::getCurrent".to_string());
    get_current.param_count = 0;
    get_current.local_count = 0;
    get_current.bytecode.push(Opcode::GetCurrentFiber);
    get_current.bytecode.push(Opcode::Return);
    fiber
        .static_methods
        .insert("getCurrent".to_string(), Arc::new(get_current));

    let mut suspend = CompiledFunction::new("Fiber::suspend".to_string());
    suspend.param_count = 1;
    suspend.required_param_count = 1;
    suspend.local_count = 1;
    suspend.local_names = vec!["this".to_string(), "value".to_string()];
    suspend.bytecode.push(Opcode::PushNull);
    suspend.bytecode.push(Opcode::Return);
    fiber
        .methods
        .insert("suspend".to_string(), Arc::new(suspend));

    let mut resume = CompiledFunction::new("Fiber::resume".to_string());
    resume.param_count = 1;
    resume.required_param_count = 0;
    resume.local_count = 1;
    resume.local_names = vec!["this".to_string(), "value".to_string()];
    resume.bytecode.push(Opcode::PushNull);
    resume.bytecode.push(Opcode::Return);
    fiber.methods.insert("resume".to_string(), Arc::new(resume));

    let mut throw = CompiledFunction::new("Fiber::throw".to_string());
    throw.param_count = 1;
    throw.required_param_count = 1;
    throw.local_count = 1;
    throw.local_names = vec!["this".to_string(), "exception".to_string()];
    throw.bytecode.push(Opcode::PushNull);
    throw.bytecode.push(Opcode::Return);
    fiber.methods.insert("throw".to_string(), Arc::new(throw));

    classes.insert("Fiber".to_string(), Arc::new(fiber));
}

use crate::vm::opcode::CompiledFunction;
