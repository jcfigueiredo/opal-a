/// An opaque identifier for a closure stored in the interpreter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClosureId(pub usize);
