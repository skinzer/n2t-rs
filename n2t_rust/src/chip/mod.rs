pub mod bus;
pub mod chip;
pub mod clock;
pub mod pin;
pub mod builder;
pub mod builtins;
pub mod subbus;

#[cfg(test)]
mod tests;

pub use bus::Bus;
pub use chip::{Chip, ChipInterface, Connection, PinSide, WireError};
pub use pin::{Pin, Voltage, HIGH, LOW};
pub use builder::ChipBuilder;
pub use builtins::{ClockedChip, DffChip, BitChip, RegisterChip, PcChip};
pub use builtins::{Memory, Ram8Chip, Ram64Chip, Ram512Chip, Ram4kChip, Ram16kChip};
pub use builtins::{Rom32kChip, ScreenChip, KeyboardChip, SCREEN_SIZE, SCREEN_OFFSET, KEYBOARD_OFFSET};
pub use builtins::{NandChip, NotChip, AndChip, OrChip, XorChip};
pub use builtins::{MuxChip, DMuxChip, DMux4WayChip, DMux8WayChip};
pub use builtins::{Not16Chip, And16Chip, Or16Chip};
pub use builtins::{Mux16Chip, Mux4Way16Chip, Mux8Way16Chip};
pub use builtins::{Add16Chip, Inc16Chip};
pub use builtins::{HalfAdderChip, FullAdderChip};
pub use builtins::{AluChip, AluFlags};
pub use clock::Clock;
pub use subbus::{InSubBus, OutSubBus, PinRange, parse_pin_range, create_input_subbus, create_output_subbus};