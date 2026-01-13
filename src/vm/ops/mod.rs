//! Opcode execution modules
//!
//! This directory contains modules for executing specific categories of bytecode opcodes.
//! Each module implements functions as methods on `VM<W>` via `impl<W: Write> super::super::VM<W>`.

mod arithmetic;
mod comparison;
mod strings;
mod logical_bitwise;
mod control_flow;
mod arrays;
mod object_ops;
mod functions;
mod exceptions;
mod closures;
mod misc;

pub use arithmetic::*;
pub use comparison::*;
pub use strings::*;
pub use logical_bitwise::*;
pub use control_flow::*;
pub use arrays::*;
pub use object_ops::*;
pub use functions::*;
pub use exceptions::*;
pub use closures::*;
pub use misc::*;
