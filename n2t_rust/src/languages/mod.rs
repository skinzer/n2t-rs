// Language parsers module - placeholder for future implementation

pub mod hdl;
pub mod assembly;
pub mod vm_lang;
pub mod jack;
pub mod tst;

pub use hdl::HdlParser;
pub use assembly::AssemblyParser;
pub use vm_lang::VmParser;
pub use jack::JackParser;
pub use tst::TstParser;