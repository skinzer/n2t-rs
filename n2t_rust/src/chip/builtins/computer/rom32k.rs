use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::chip::{ChipInterface, Bus, Pin};
use crate::error::Result;
use super::super::sequential::Memory;

/// ROM32K - 32768-register ROM using 15-bit address
/// ROM is read-only memory - load signal has no effect
#[derive(Debug)]
pub struct Rom32kChip {
    name: String,
    input_pins: HashMap<String, Rc<RefCell<dyn Pin>>>,
    output_pins: HashMap<String, Rc<RefCell<dyn Pin>>>,
    internal_pins: HashMap<String, Rc<RefCell<dyn Pin>>>,
    memory: Memory,
}

impl Rom32kChip {
    pub fn new() -> Self {
        let mut input_pins = HashMap::new();
        let mut output_pins = HashMap::new();
        
        // Create pins with trait object casting - ROM has address input and data output only
        input_pins.insert("address".to_string(), Rc::new(RefCell::new(Bus::new("address".to_string(), 15))) as Rc<RefCell<dyn Pin>>);
        output_pins.insert("out".to_string(), Rc::new(RefCell::new(Bus::new("out".to_string(), 16))) as Rc<RefCell<dyn Pin>>);
        
        Self {
            name: "ROM32K".to_string(),
            input_pins,
            output_pins,
            internal_pins: HashMap::new(),
            memory: Memory::new(32768), // 2^15 = 32768 registers
        }
    }
    
    /// Load data into ROM from a vector of instructions
    pub fn load_program(&mut self, program: &[u16]) {
        for (address, &instruction) in program.iter().enumerate() {
            if address < 32768 {
                self.memory.set(address, instruction);
            }
        }
    }
    
    /// Get current memory for inspection/testing
    pub fn memory(&self) -> &Memory {
        &self.memory
    }
    
    /// Set a single memory location (for testing)
    pub fn set_memory(&mut self, address: usize, value: u16) {
        if address < 32768 {
            self.memory.set(address, value);
        }
    }
}

impl ChipInterface for Rom32kChip {
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
        // ROM is pure combinatorial - output data at address immediately
        let address = self.input_pins["address"].borrow().bus_voltage() as usize;
        let address = address & 0b111111111111111; // Mask to 15 bits for ROM32K
        let value = self.memory.get(address);
        self.output_pins["out"].borrow_mut().set_bus_voltage(value);
        Ok(())
    }
    
    fn reset(&mut self) -> Result<()> {
        // ROM doesn't clear its contents on reset, just outputs current value at address 0
        self.output_pins["out"].borrow_mut().set_bus_voltage(self.memory.get(0));
        Ok(())
    }
}

impl Default for Rom32kChip {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rom32k_basic_structure() {
        let rom32k = Rom32kChip::new();
        
        // Test basic properties
        assert_eq!(rom32k.name(), "ROM32K");
        assert!(rom32k.get_pin("address").is_ok());
        assert!(rom32k.get_pin("out").is_ok());
        
        // ROM should not have load or in pins
        assert!(rom32k.get_pin("load").is_err());
        assert!(rom32k.get_pin("in").is_err());
        
        // Test memory size
        assert_eq!(rom32k.memory().size(), 32768);
    }
    
    #[test]
    fn test_rom32k_read_operations() {
        let mut rom32k = Rom32kChip::new();
        
        // Load some test data
        let test_program = vec![0x1234, 0x5678, 0x9ABC, 0xDEF0];
        rom32k.load_program(&test_program);
        
        // Test reading from different addresses
        for (expected_addr, &expected_value) in test_program.iter().enumerate() {
            rom32k.get_pin("address").unwrap().borrow_mut().set_bus_voltage(expected_addr as u16);
            rom32k.eval().unwrap();
            let output = rom32k.get_pin("out").unwrap().borrow().bus_voltage();
            assert_eq!(output, expected_value, "ROM32K[{}] should contain {:#x}", expected_addr, expected_value);
        }
    }
    
    #[test]
    fn test_rom32k_address_masking() {
        let mut rom32k = Rom32kChip::new();
        
        // Set value at address 0
        rom32k.set_memory(0, 0x1111);
        
        // Test that addresses are properly masked to 15 bits
        // Address 32768 (0b1000000000000000) should be masked to 0 (0b000000000000000)
        rom32k.get_pin("address").unwrap().borrow_mut().set_bus_voltage(32768);
        rom32k.eval().unwrap();
        let output = rom32k.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x1111, "Address 32768 should be masked to 0");
    }
    
    #[test]
    fn test_rom32k_boundary_addresses() {
        let mut rom32k = Rom32kChip::new();
        
        // Test reads from boundary addresses including powers of 2
        let test_addresses = [0, 1, 1023, 1024, 2047, 2048, 4095, 4096, 8191, 8192, 16383, 16384, 32767];
        let test_values = [0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6666, 0x7777, 0x8888, 0x9999, 0xAAAA, 0xBBBB, 0xCCCC, 0xDDDD];
        
        // Set memory at all test addresses
        for (i, &addr) in test_addresses.iter().enumerate() {
            rom32k.set_memory(addr, test_values[i]);
        }
        
        // Verify all values can be read correctly
        for (i, &addr) in test_addresses.iter().enumerate() {
            rom32k.get_pin("address").unwrap().borrow_mut().set_bus_voltage(addr as u16);
            rom32k.eval().unwrap();
            let output = rom32k.get_pin("out").unwrap().borrow().bus_voltage();
            assert_eq!(output, test_values[i], "ROM32K[{}] should contain correct value", addr);
        }
    }
    
    #[test]
    fn test_rom32k_empty_memory() {
        let mut rom32k = Rom32kChip::new();
        
        // Test that empty ROM returns 0 for all addresses
        for addr in [0, 100, 1000, 10000, 32767] {
            rom32k.get_pin("address").unwrap().borrow_mut().set_bus_voltage(addr as u16);
            rom32k.eval().unwrap();
            let output = rom32k.get_pin("out").unwrap().borrow().bus_voltage();
            assert_eq!(output, 0, "Empty ROM32K[{}] should be 0", addr);
        }
    }
    
    #[test]
    fn test_rom32k_load_program() {
        let mut rom32k = Rom32kChip::new();
        
        // Create a larger test program
        let program: Vec<u16> = (0..1000).map(|i| i as u16 * 2 + 1).collect();
        rom32k.load_program(&program);
        
        // Test that program was loaded correctly
        for (addr, &expected) in program.iter().enumerate().take(100) {
            rom32k.get_pin("address").unwrap().borrow_mut().set_bus_voltage(addr as u16);
            rom32k.eval().unwrap();
            let output = rom32k.get_pin("out").unwrap().borrow().bus_voltage();
            assert_eq!(output, expected, "Program at ROM32K[{}] should be {}", addr, expected);
        }
    }
}