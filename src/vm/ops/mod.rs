//! Opcode execution modules
//!
//! This directory contains modules for executing specific categories of bytecode opcodes.
//! Each module implements functions as methods on `VM<W>` via `impl<W: Write> super::super::VM<W>`.

mod arithmetic;
mod arrays;
mod call_ops;
mod callable_ops;
mod comparison;
mod control_flow;
mod exceptions;
mod fiber;
mod generator;
mod logical_bitwise;
mod method_calls;
mod misc;
mod named_call_ops;
mod object_creation;
mod property_access;
mod property_ops;
mod static_ops;
mod strings;

pub use arithmetic::*;
pub use arrays::*;
pub use call_ops::*;
pub use callable_ops::*;
pub use comparison::*;
pub use control_flow::*;
pub use exceptions::*;
pub use fiber::*;
pub use generator::*;
pub use logical_bitwise::*;
pub use method_calls::*;
pub use misc::*;
pub use named_call_ops::*;
pub use object_creation::*;
pub use property_access::*;
pub use property_ops::*;
pub use static_ops::*;
pub use strings::*;
