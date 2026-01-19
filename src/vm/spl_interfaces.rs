//! SPL (Standard PHP Library) built-in interfaces
//!
//! This module registers SPL interfaces as built-in interfaces that can be
//! implemented by user code: Traversable, Iterator, IteratorAggregate,
//! Countable, ArrayAccess, and Stringable.

use crate::vm::class::CompiledInterface;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

lazy_static! {
    pub static ref BUILTIN_INTERFACES: Mutex<HashMap<String, Arc<CompiledInterface>>> =
        Mutex::new(HashMap::new());
}

pub fn initialize_builtin_interfaces() {
    let mut interfaces = BUILTIN_INTERFACES.lock().unwrap();
    register_traversable_interface(&mut interfaces);
    register_iterator_interface(&mut interfaces);
    register_iterator_aggregate_interface(&mut interfaces);
    register_countable_interface(&mut interfaces);
    register_array_access_interface(&mut interfaces);
    register_stringable_interface(&mut interfaces);
}

pub fn register_builtin_interfaces(interfaces: &mut HashMap<String, Arc<CompiledInterface>>) {
    let mut builtin = BUILTIN_INTERFACES.lock().unwrap();
    register_traversable_interface(&mut builtin);
    register_iterator_interface(&mut builtin);
    register_iterator_aggregate_interface(&mut builtin);
    register_countable_interface(&mut builtin);
    register_array_access_interface(&mut builtin);
    register_stringable_interface(&mut builtin);
    interfaces.extend(builtin.clone());
}

fn register_traversable_interface(interfaces: &mut HashMap<String, Arc<CompiledInterface>>) {
    let traversable = CompiledInterface::new("Traversable".to_string());
    let traversable_arc = Arc::new(traversable);
    interfaces.insert("Traversable".to_string(), Arc::clone(&traversable_arc));
    interfaces.insert("\\Traversable".to_string(), traversable_arc);
}

fn register_iterator_interface(interfaces: &mut HashMap<String, Arc<CompiledInterface>>) {
    let mut iterator = CompiledInterface::new("Iterator".to_string());
    iterator.parents = vec!["Traversable".to_string()];
    iterator.method_signatures = vec![
        ("current".to_string(), 0),
        ("next".to_string(), 0),
        ("key".to_string(), 0),
        ("valid".to_string(), 0),
        ("rewind".to_string(), 0),
    ];
    let iterator_arc = Arc::new(iterator);
    interfaces.insert("Iterator".to_string(), Arc::clone(&iterator_arc));
    interfaces.insert("\\Iterator".to_string(), iterator_arc);
}

fn register_iterator_aggregate_interface(interfaces: &mut HashMap<String, Arc<CompiledInterface>>) {
    let mut aggregate = CompiledInterface::new("IteratorAggregate".to_string());
    aggregate.parents = vec!["Traversable".to_string()];
    aggregate.method_signatures = vec![("getIterator".to_string(), 0)];
    let aggregate_arc = Arc::new(aggregate);
    interfaces.insert("IteratorAggregate".to_string(), Arc::clone(&aggregate_arc));
    interfaces.insert("\\IteratorAggregate".to_string(), aggregate_arc);
}

fn register_countable_interface(interfaces: &mut HashMap<String, Arc<CompiledInterface>>) {
    let mut countable = CompiledInterface::new("Countable".to_string());
    countable.method_signatures = vec![("count".to_string(), 0)];
    let countable_arc = Arc::new(countable);
    interfaces.insert("Countable".to_string(), Arc::clone(&countable_arc));
    interfaces.insert("\\Countable".to_string(), countable_arc);
}

fn register_array_access_interface(interfaces: &mut HashMap<String, Arc<CompiledInterface>>) {
    let mut access = CompiledInterface::new("ArrayAccess".to_string());
    access.method_signatures = vec![
        ("offsetExists".to_string(), 1),
        ("offsetGet".to_string(), 1),
        ("offsetSet".to_string(), 2),
        ("offsetUnset".to_string(), 1),
    ];
    let access_arc = Arc::new(access);
    interfaces.insert("ArrayAccess".to_string(), Arc::clone(&access_arc));
    interfaces.insert("\\ArrayAccess".to_string(), access_arc);
}

fn register_stringable_interface(interfaces: &mut HashMap<String, Arc<CompiledInterface>>) {
    let mut stringable = CompiledInterface::new("Stringable".to_string());
    stringable.method_signatures = vec![("__toString".to_string(), 0)];
    let stringable_arc = Arc::new(stringable);
    interfaces.insert("Stringable".to_string(), Arc::clone(&stringable_arc));
    interfaces.insert("\\Stringable".to_string(), stringable_arc);
}
