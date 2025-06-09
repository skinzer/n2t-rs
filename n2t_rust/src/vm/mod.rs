// VM module - placeholder for future implementation

pub mod vm;
pub mod memory;
pub mod builtins;

pub use vm::VirtualMachine;
pub use memory::VmMemory;
pub use builtins::VmBuiltins;