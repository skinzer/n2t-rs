// Computer-level components - components needed for the complete computer system

pub mod rom32k;
pub mod screen;
pub mod keyboard;

// Re-export computer-level chips
pub use rom32k::Rom32kChip;
pub use screen::{ScreenChip, SCREEN_SIZE, SCREEN_OFFSET};
pub use keyboard::{KeyboardChip, KEYBOARD_OFFSET};