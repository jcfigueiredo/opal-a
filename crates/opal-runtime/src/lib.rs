pub mod closure;
pub mod env;
pub mod function;
pub mod value;

pub use closure::ClosureId;
pub use env::Environment;
pub use function::FunctionId;
pub use value::{
    ActorId, AstId, ClassId, InstanceId, ModuleId, NativeFunctionId, NativeObjectId, Value,
};
