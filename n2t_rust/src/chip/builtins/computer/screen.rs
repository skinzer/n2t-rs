use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::chip::{ChipInterface, Clock, Bus, Pin};
use crate::chip::pin::{Voltage, HIGH};
use crate::error::Result;
use tokio::sync::broadcast;
use super::super::sequential::{ClockedChip, Memory};

pub const SCREEN_SIZE: usize = 8192; // 2^13 = 8192 registers (512x256 pixels / 16 pixels per word)
pub const SCREEN_OFFSET: usize = 16384; // Screen starts at address 16384 in memory map

/// Screen - 8192-register screen memory using 13-bit address
/// Screen is memory-mapped starting at address 16384
#[derive(Debug)]
pub struct ScreenChip {
    name: String,
    input_pins: HashMap<String, Rc<RefCell<dyn Pin>>>,
    output_pins: HashMap<String, Rc<RefCell<dyn Pin>>>,
    internal_pins: HashMap<String, Rc<RefCell<dyn Pin>>>,
    clock_subscriber: Option<broadcast::Receiver<crate::chip::clock::ClockTick>>,
    memory: Memory,
    // Internal state for clocked operation
    next_data: u16,
    current_address: usize,
}

impl ScreenChip {
    pub fn new() -> Self {
        let mut input_pins = HashMap::new();
        let mut output_pins = HashMap::new();
        
        // Create pins with trait object casting
        input_pins.insert("in".to_string(), Rc::new(RefCell::new(Bus::new("in".to_string(), 16))) as Rc<RefCell<dyn Pin>>);
        input_pins.insert("load".to_string(), Rc::new(RefCell::new(Bus::new("load".to_string(), 1))) as Rc<RefCell<dyn Pin>>);
        input_pins.insert("address".to_string(), Rc::new(RefCell::new(Bus::new("address".to_string(), 13))) as Rc<RefCell<dyn Pin>>);
        output_pins.insert("out".to_string(), Rc::new(RefCell::new(Bus::new("out".to_string(), 16))) as Rc<RefCell<dyn Pin>>);
        
        Self {
            name: "Screen".to_string(),
            input_pins,
            output_pins,
            internal_pins: HashMap::new(),
            clock_subscriber: None,
            memory: Memory::new(SCREEN_SIZE),
            next_data: 0,
            current_address: 0,
        }
    }
    
    pub fn subscribe_to_clock(&mut self, clock: &Clock) {
        self.clock_subscriber = Some(clock.subscribe());
    }
    
    pub fn memory(&self) -> &Memory {
        &self.memory
    }
    
    /// Get pixel state for a given x, y coordinate
    /// Each memory word represents 16 pixels horizontally
    /// Screen is 512x256 pixels
    pub fn get_pixel(&self, x: usize, y: usize) -> bool {
        if x >= 512 || y >= 256 {
            return false; // Out of bounds
        }
        
        let word_address = (y * 32) + (x / 16); // 32 words per row (512/16)
        let bit_position = x % 16;
        let word_value = self.memory.get(word_address);
        
        (word_value >> bit_position) & 1 == 1
    }
    
    /// Set pixel state for a given x, y coordinate
    pub fn set_pixel(&mut self, x: usize, y: usize, value: bool) {
        if x >= 512 || y >= 256 {
            return; // Out of bounds
        }
        
        let word_address = (y * 32) + (x / 16);
        let bit_position = x % 16;
        let mut word_value = self.memory.get(word_address);
        
        if value {
            word_value |= 1 << bit_position;
        } else {
            word_value &= !(1 << bit_position);
        }
        
        self.memory.set(word_address, word_value);
    }
    
    /// Clear the entire screen
    pub fn clear_screen(&mut self) {
        for address in 0..SCREEN_SIZE {
            self.memory.set(address, 0);
        }
    }
    
    /// Fill the entire screen
    pub fn fill_screen(&mut self) {
        for address in 0..SCREEN_SIZE {
            self.memory.set(address, 0xFFFF);
        }
    }
}

impl ChipInterface for ScreenChip {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn input_pins(&self) -> &HashMap<String, Rc<RefCell<dyn Pin>>> {
        &self.input_pins
    }
    
    fn output_pins(&self) -> &HashMap<String, Rc<RefCell<dyn Pin>>> {
        &self.output_pins
    }
    
    fn internal_pins(&self) -> &HashMap<String, Rc<RefCell<dyn Pin>>> {
        &self.internal_pins
    }
    
    fn get_pin(&self, name: &str) -> Result<Rc<RefCell<dyn Pin>>> {
        if let Some(pin) = self.input_pins.get(name) {
            return Ok(pin.clone());
        }
        if let Some(pin) = self.output_pins.get(name) {
            return Ok(pin.clone());
        }
        Err(crate::error::SimulatorError::PinNotFound {
            pin: name.to_string(),
            chip: self.name.clone(),
        }.into())
    }
    
    fn is_input_pin(&self, name: &str) -> bool {
        self.input_pins.contains_key(name)
    }
    
    fn is_output_pin(&self, name: &str) -> bool {
        self.output_pins.contains_key(name)
    }
    
    fn eval(&mut self) -> Result<()> {
        // Combinatorial read: output current value at address
        let address = self.input_pins["address"].borrow().bus_voltage() as usize;
        let address = address & 0b1111111111111; // Mask to 13 bits for Screen
        let value = self.memory.get(address);
        self.output_pins["out"].borrow_mut().set_bus_voltage(value);
        Ok(())
    }
    
    fn reset(&mut self) -> Result<()> {
        self.memory.reset();
        self.next_data = 0;
        self.current_address = 0;
        self.output_pins["out"].borrow_mut().set_bus_voltage(0);
        Ok(())
    }
}

impl ClockedChip for ScreenChip {
    fn tick(&mut self, _clock_level: Voltage) -> Result<()> {
        // Rising edge: sample inputs and conditionally write to memory
        let load = self.input_pins["load"].borrow().voltage(None)?;
        let address = self.input_pins["address"].borrow().bus_voltage() as usize;
        self.current_address = address & 0b1111111111111; // Mask to 13 bits for Screen
        
        if load == HIGH {
            self.next_data = self.input_pins["in"].borrow().bus_voltage();
            self.memory.set(self.current_address, self.next_data);
        }
        
        Ok(())
    }
    
    fn tock(&mut self, _clock_level: Voltage) -> Result<()> {
        // Falling edge: update output with current memory value
        let value = self.memory.get(self.current_address);
        self.output_pins["out"].borrow_mut().set_bus_voltage(value);
        Ok(())
    }
}

impl Default for ScreenChip {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chip::pin::{HIGH, LOW};
    
    #[test]
    fn test_screen_basic_structure() {
        let screen = ScreenChip::new();
        
        // Test basic properties
        assert_eq!(screen.name(), "Screen");
        assert!(screen.get_pin("in").is_ok());
        assert!(screen.get_pin("address").is_ok());
        assert!(screen.get_pin("load").is_ok());
        assert!(screen.get_pin("out").is_ok());
        
        // Test memory size
        assert_eq!(screen.memory().size(), SCREEN_SIZE);
        assert_eq!(SCREEN_SIZE, 8192);
        assert_eq!(SCREEN_OFFSET, 16384);
    }
    
    #[test]
    fn test_screen_memory_operations() {
        let mut screen = ScreenChip::new();
        
        // Test write operation at address 0
        screen.get_pin("address").unwrap().borrow_mut().set_bus_voltage(0);
        screen.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0x1234);
        screen.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        
        // Simulate clock cycle
        screen.tick(HIGH).unwrap();
        screen.tock(LOW).unwrap();
        
        // Verify write worked
        screen.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
        screen.eval().unwrap();
        let output = screen.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x1234, "Screen[0] should contain written value");
    }
    
    #[test]
    fn test_screen_pixel_operations() {
        let mut screen = ScreenChip::new();
        
        // Test setting individual pixels
        screen.set_pixel(0, 0, true);   // Top-left pixel
        screen.set_pixel(15, 0, true);  // 16th pixel in first row (end of first word)
        screen.set_pixel(16, 0, true);  // 17th pixel in first row (start of second word)
        screen.set_pixel(511, 255, true); // Bottom-right pixel
        
        // Verify pixels were set
        assert!(screen.get_pixel(0, 0), "Pixel (0,0) should be set");
        assert!(screen.get_pixel(15, 0), "Pixel (15,0) should be set");
        assert!(screen.get_pixel(16, 0), "Pixel (16,0) should be set");
        assert!(screen.get_pixel(511, 255), "Pixel (511,255) should be set");
        
        // Test clearing pixels
        screen.set_pixel(0, 0, false);
        assert!(!screen.get_pixel(0, 0), "Pixel (0,0) should be cleared");
        
        // Test out-of-bounds pixels
        assert!(!screen.get_pixel(512, 0), "Out-of-bounds pixel should be false");
        assert!(!screen.get_pixel(0, 256), "Out-of-bounds pixel should be false");
    }
    
    #[test]
    fn test_screen_memory_word_layout() {
        let mut screen = ScreenChip::new();
        
        // Test that pixels are packed correctly into memory words
        // Set all 16 pixels in the first word
        for x in 0..16 {
            screen.set_pixel(x, 0, true);
        }
        
        // The first memory word should be 0xFFFF
        let first_word = screen.memory().get(0);
        assert_eq!(first_word, 0xFFFF, "First word should have all 16 bits set");
        
        // Test specific bit patterns
        screen.clear_screen();
        screen.set_pixel(1, 0, true);  // Bit 1
        screen.set_pixel(8, 0, true);  // Bit 8
        screen.set_pixel(15, 0, true); // Bit 15
        
        let word_value = screen.memory().get(0);
        let expected = (1 << 1) | (1 << 8) | (1 << 15);
        assert_eq!(word_value, expected, "Word should have bits 1, 8, and 15 set");
    }
    
    #[test]
    fn test_screen_clear_and_fill() {
        let mut screen = ScreenChip::new();
        
        // Set some pixels
        screen.set_pixel(100, 100, true);
        screen.set_pixel(200, 200, true);
        assert!(screen.get_pixel(100, 100));
        
        // Clear screen
        screen.clear_screen();
        assert!(!screen.get_pixel(100, 100), "Pixel should be cleared");
        assert!(!screen.get_pixel(200, 200), "Pixel should be cleared");
        
        // Verify all memory is cleared
        for address in 0..SCREEN_SIZE {
            assert_eq!(screen.memory().get(address), 0, "Memory[{}] should be 0", address);
        }
        
        // Fill screen
        screen.fill_screen();
        assert!(screen.get_pixel(100, 100), "Pixel should be set");
        assert!(screen.get_pixel(200, 200), "Pixel should be set");
        assert!(screen.get_pixel(0, 0), "Corner pixel should be set");
        assert!(screen.get_pixel(511, 255), "Corner pixel should be set");
        
        // Verify all memory is filled
        for address in 0..SCREEN_SIZE {
            assert_eq!(screen.memory().get(address), 0xFFFF, "Memory[{}] should be 0xFFFF", address);
        }
    }
    
    #[test]
    fn test_screen_address_masking() {
        let mut screen = ScreenChip::new();
        
        // Test that addresses are properly masked to 13 bits
        // Address 8192 (0b10000000000000) should be masked to 0 (0b0000000000000)
        screen.get_pin("address").unwrap().borrow_mut().set_bus_voltage(8192);
        screen.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0x9999);
        screen.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        
        screen.tick(HIGH).unwrap();
        screen.tock(LOW).unwrap();
        
        // Check that value was written to address 0 (8192 & 0b1111111111111 = 0)
        screen.get_pin("address").unwrap().borrow_mut().set_bus_voltage(0);
        screen.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
        screen.eval().unwrap();
        let output = screen.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x9999, "Address 8192 should be masked to 0");
    }
}