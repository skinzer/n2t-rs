use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::chip::{ChipInterface, Bus, Pin};
use crate::error::Result;

pub const KEYBOARD_OFFSET: usize = 24576; // Keyboard at address 24576 in memory map

/// Keyboard - Memory-mapped keyboard input device
/// The keyboard is a read-only device that provides the current key code
#[derive(Debug)]
pub struct KeyboardChip {
    name: String,
    input_pins: HashMap<String, Rc<RefCell<dyn Pin>>>,
    output_pins: HashMap<String, Rc<RefCell<dyn Pin>>>,
    internal_pins: HashMap<String, Rc<RefCell<dyn Pin>>>,
    current_key: u16,
}

impl KeyboardChip {
    pub fn new() -> Self {
        let mut output_pins = HashMap::new();
        
        // Keyboard only has output - no input pins
        output_pins.insert("out".to_string(), Rc::new(RefCell::new(Bus::new("out".to_string(), 16))) as Rc<RefCell<dyn Pin>>);
        
        Self {
            name: "Keyboard".to_string(),
            input_pins: HashMap::new(),
            output_pins,
            internal_pins: HashMap::new(),
            current_key: 0,
        }
    }
    
    /// Get the current key value
    pub fn get_key(&self) -> u16 {
        self.current_key
    }
    
    /// Set the current key value (simulates key press)
    pub fn set_key(&mut self, key: u16) {
        self.current_key = key & 0xFFFF;
        // Update output immediately
        self.output_pins["out"].borrow_mut().set_bus_voltage(self.current_key);
    }
    
    /// Clear the current key (simulates key release)
    pub fn clear_key(&mut self) {
        self.current_key = 0;
        self.output_pins["out"].borrow_mut().set_bus_voltage(0);
    }
    
    /// Simulate typing a character (sets key code based on ASCII)
    pub fn type_char(&mut self, c: char) {
        let key_code = match c {
            // Standard ASCII characters
            'a'..='z' => c as u16,
            'A'..='Z' => c as u16,
            '0'..='9' => c as u16,
            ' ' => 32,
            '\n' => 128, // Enter key in Hack
            '\t' => 129, // Tab key in Hack
            // Special keys with Hack-specific codes
            _ => c as u16, // Default to ASCII for other characters
        };
        self.set_key(key_code);
    }
    
    /// Check if any key is currently pressed
    pub fn is_key_pressed(&self) -> bool {
        self.current_key != 0
    }
}

impl ChipInterface for KeyboardChip {
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
        // Keyboard always outputs current key value
        self.output_pins["out"].borrow_mut().set_bus_voltage(self.current_key);
        Ok(())
    }
    
    fn reset(&mut self) -> Result<()> {
        // Reset clears any pressed key
        self.current_key = 0;
        self.output_pins["out"].borrow_mut().set_bus_voltage(0);
        Ok(())
    }
}

impl Default for KeyboardChip {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_keyboard_basic_structure() {
        let keyboard = KeyboardChip::new();
        
        // Test basic properties
        assert_eq!(keyboard.name(), "Keyboard");
        assert!(keyboard.get_pin("out").is_ok());
        
        // Keyboard should not have input pins
        assert_eq!(keyboard.input_pins().len(), 0);
        assert_eq!(keyboard.output_pins().len(), 1);
        
        // Test constants
        assert_eq!(KEYBOARD_OFFSET, 24576);
    }
    
    #[test]
    fn test_keyboard_key_operations() {
        let mut keyboard = KeyboardChip::new();
        
        // Initially no key is pressed
        assert_eq!(keyboard.get_key(), 0);
        assert!(!keyboard.is_key_pressed());
        
        // Test setting a key
        keyboard.set_key(65); // 'A'
        assert_eq!(keyboard.get_key(), 65);
        assert!(keyboard.is_key_pressed());
        
        // Verify output pin is updated
        keyboard.eval().unwrap();
        let output = keyboard.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 65);
        
        // Test clearing key
        keyboard.clear_key();
        assert_eq!(keyboard.get_key(), 0);
        assert!(!keyboard.is_key_pressed());
        
        // Verify output pin is cleared
        let output = keyboard.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0);
    }
    
    #[test]
    fn test_keyboard_character_typing() {
        let mut keyboard = KeyboardChip::new();
        
        // Test typing regular characters
        keyboard.type_char('a');
        assert_eq!(keyboard.get_key(), 'a' as u16);
        
        keyboard.type_char('Z');
        assert_eq!(keyboard.get_key(), 'Z' as u16);
        
        keyboard.type_char('5');
        assert_eq!(keyboard.get_key(), '5' as u16);
        
        // Test special characters
        keyboard.type_char(' ');
        assert_eq!(keyboard.get_key(), 32); // Space
        
        keyboard.type_char('\n');
        assert_eq!(keyboard.get_key(), 128); // Enter in Hack
        
        keyboard.type_char('\t');
        assert_eq!(keyboard.get_key(), 129); // Tab in Hack
    }
    
    #[test]
    fn test_keyboard_value_masking() {
        let mut keyboard = KeyboardChip::new();
        
        // Test that values are masked to 16 bits
        keyboard.set_key(0x10000u32 as u16); // 17-bit value cast to u16
        assert_eq!(keyboard.get_key(), 0); // Should be masked to 16 bits
        
        keyboard.set_key(0xFFFF);
        assert_eq!(keyboard.get_key(), 0xFFFF);
        
        keyboard.set_key(0x1FFFFu32 as u16); // 17-bit value cast to u16
        assert_eq!(keyboard.get_key(), 0xFFFF); // Should be masked to 16 bits
    }
    
    #[test]
    fn test_keyboard_reset() {
        let mut keyboard = KeyboardChip::new();
        
        // Set a key and verify it's set
        keyboard.set_key(100);
        assert_eq!(keyboard.get_key(), 100);
        assert!(keyboard.is_key_pressed());
        
        // Reset should clear the key
        keyboard.reset().unwrap();
        assert_eq!(keyboard.get_key(), 0);
        assert!(!keyboard.is_key_pressed());
        
        // Output should also be cleared
        let output = keyboard.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0);
    }
    
    #[test]
    fn test_keyboard_eval_updates_output() {
        let mut keyboard = KeyboardChip::new();
        
        // Set key internally
        keyboard.current_key = 42;
        
        // Eval should update output pin
        keyboard.eval().unwrap();
        let output = keyboard.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 42);
        
        // Change key and eval again
        keyboard.current_key = 123;
        keyboard.eval().unwrap();
        let output = keyboard.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 123);
    }
    
    #[test]
    fn test_keyboard_common_key_codes() {
        let mut keyboard = KeyboardChip::new();
        
        // Test some common ASCII key codes
        let test_cases = [
            ('A', 65),
            ('Z', 90),
            ('a', 97),
            ('z', 122),
            ('0', 48),
            ('9', 57),
            (' ', 32),
            ('\n', 128), // Hack Enter
            ('\t', 129), // Hack Tab
        ];
        
        for (character, expected_code) in test_cases {
            keyboard.type_char(character);
            assert_eq!(keyboard.get_key(), expected_code, "Character '{}' should have code {}", character, expected_code);
        }
    }
}