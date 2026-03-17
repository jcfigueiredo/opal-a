pub mod eval;
pub mod loader;
mod string_methods;
mod dict_methods;

pub use eval::{EvalError, Interpreter};
