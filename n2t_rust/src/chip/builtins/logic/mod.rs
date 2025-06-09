// Logic gate implementations

pub mod nand;
pub mod not;
pub mod and;
pub mod or;
pub mod xor;
pub mod mux;
pub mod dmux;
pub mod dmux_multi;

// Re-export all logic chips
pub use nand::NandChip;
pub use not::NotChip;
pub use and::AndChip;
pub use or::OrChip;
pub use xor::XorChip;
pub use mux::MuxChip;
pub use dmux::DMuxChip;
pub use dmux_multi::{DMux4WayChip, DMux8WayChip};