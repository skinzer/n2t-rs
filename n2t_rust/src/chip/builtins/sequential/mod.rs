// Sequential chip implementations - chips with state that respond to clock signals

use crate::chip::ChipInterface;
use crate::chip::pin::Voltage;
use crate::error::Result;

/// Base trait for clocked chips that respond to clock signals
pub trait ClockedChip: ChipInterface {
    /// Called on rising clock edge (HIGH)
    /// This is when sequential chips should sample their inputs
    fn tick(&mut self, clock_level: Voltage) -> Result<()>;
    
    /// Called on falling clock edge (LOW)  
    /// This is when sequential chips should update their outputs
    fn tock(&mut self, clock_level: Voltage) -> Result<()>;
}

pub mod dff;
pub mod bit;
pub mod register;
pub mod pc;
pub mod memory;
pub mod ram8;
pub mod ram64;
pub mod ram512;
pub mod ram4k;
pub mod ram16k;

// Re-export all sequential chips
pub use dff::DffChip;
pub use bit::BitChip;
pub use register::RegisterChip;
pub use pc::PcChip;
pub use memory::Memory;
pub use ram8::Ram8Chip;
pub use ram64::Ram64Chip;
pub use ram512::Ram512Chip;
pub use ram4k::Ram4kChip;
pub use ram16k::Ram16kChip;

// Re-export the ClockedChip trait (already exported above)