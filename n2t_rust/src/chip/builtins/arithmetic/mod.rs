// Arithmetic and 16-bit chip implementations

pub mod not16;
pub mod and16;
pub mod or16;
pub mod mux16;
pub mod add16;
pub mod inc16;
pub mod half_adder;
pub mod full_adder;
pub mod alu;

// Re-export all arithmetic chips
pub use not16::Not16Chip;
pub use and16::And16Chip;
pub use or16::Or16Chip;
pub use mux16::{Mux16Chip, Mux4Way16Chip, Mux8Way16Chip};
pub use add16::Add16Chip;
pub use inc16::Inc16Chip;
pub use half_adder::HalfAdderChip;
pub use full_adder::FullAdderChip;
pub use alu::{AluChip, AluFlags};