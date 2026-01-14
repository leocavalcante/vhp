//! Opcode execution modules
//!
//! This directory contains modules for executing specific categories of bytecode opcodes.
//! Each module implements functions as methods on `VM<W>` via `impl<W: Write> super::super::VM<W>`.

mod arithmetic;
mod arrays;
mod comparison;
mod control_flow;
mod exceptions;
mod functions;
mod logical_bitwise;
mod method_calls;
mod misc;
mod object_ops;
mod strings;

pub use arithmetic::*;
pub use arrays::*;
pub use comparison::*;
pub use control_flow::*;
pub use exceptions::*;
pub use functions::*;
pub use logical_bitwise::*;
pub use method_calls::*;
pub use misc::*;
pub use object_ops::*;
pub use strings::*;
