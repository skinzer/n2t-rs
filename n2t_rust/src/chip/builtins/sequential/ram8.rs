use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::chip::{ChipInterface, Clock, Bus, Pin};
use crate::chip::pin::{Voltage, HIGH};
use crate::error::Result;
use tokio::sync::broadcast;
use super::{ClockedChip};
use super::memory::Memory;

/// RAM8 - 8-register RAM using 3-bit address
#[derive(Debug)]
pub struct Ram8Chip {
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

impl Ram8Chip {
    pub fn new() -> Self {
        let mut input_pins = HashMap::new();
        let mut output_pins = HashMap::new();
        
        // Create pins with trait object casting
        input_pins.insert("in".to_string(), Rc::new(RefCell::new(Bus::new("in".to_string(), 16))) as Rc<RefCell<dyn Pin>>);
        input_pins.insert("load".to_string(), Rc::new(RefCell::new(Bus::new("load".to_string(), 1))) as Rc<RefCell<dyn Pin>>);
        input_pins.insert("address".to_string(), Rc::new(RefCell::new(Bus::new("address".to_string(), 3))) as Rc<RefCell<dyn Pin>>);
        output_pins.insert("out".to_string(), Rc::new(RefCell::new(Bus::new("out".to_string(), 16))) as Rc<RefCell<dyn Pin>>);
        
        Self {
            name: "RAM8".to_string(),
            input_pins,
            output_pins,
            internal_pins: HashMap::new(),
            clock_subscriber: None,
            memory: Memory::new(8), // 2^3 = 8 registers
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
}

impl ChipInterface for Ram8Chip {
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
        // Get current inputs
        let address = self.input_pins["address"].borrow().bus_voltage() as usize;
        let address = address & 0b111; // Mask to 3 bits for RAM8
        let load = self.input_pins["load"].borrow().voltage(None)?;
        
        // If load is high, write to memory (for testing purposes)
        if load == HIGH {
            let data = self.input_pins["in"].borrow().bus_voltage();
            self.memory.set(address, data);
        }
        
        // Always output current value at address
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

impl ClockedChip for Ram8Chip {
    fn tick(&mut self, _clock_level: Voltage) -> Result<()> {
        // Rising edge: sample inputs and conditionally write to memory
        let load = self.input_pins["load"].borrow().voltage(None)?;
        let address = self.input_pins["address"].borrow().bus_voltage() as usize;
        self.current_address = address & 0b111; // Mask to 3 bits for RAM8
        
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

impl Default for Ram8Chip {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chip::pin::{HIGH, LOW};
    
    #[test]
    fn test_ram8_basic_structure() {
        let ram8 = Ram8Chip::new();
        
        // Test basic properties
        assert_eq!(ram8.name(), "RAM8");
        assert!(ram8.get_pin("in").is_ok());
        assert!(ram8.get_pin("address").is_ok());
        assert!(ram8.get_pin("load").is_ok());
        assert!(ram8.get_pin("out").is_ok());
        
        // Test memory size
        assert_eq!(ram8.memory().size(), 8);
    }
    
    #[test]
    fn test_ram8_combinatorial_read() {
        let mut ram8 = Ram8Chip::new();
        
        // Test initial state - should read 0 from all addresses
        for addr in 0..8 {
            ram8.get_pin("address").unwrap().borrow_mut().set_bus_voltage(addr);
            ram8.eval().unwrap();
            let output = ram8.get_pin("out").unwrap().borrow().bus_voltage();
            assert_eq!(output, 0, "Initial RAM8[{}] should be 0", addr);
        }
    }
    
    #[test]
    fn test_ram8_sequential_write_read() {
        let mut ram8 = Ram8Chip::new();
        
        // Test write operation at address 0
        ram8.get_pin("address").unwrap().borrow_mut().set_bus_voltage(0);
        ram8.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0x1234);
        ram8.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        
        // Simulate clock cycle (tick for write, tock for output update)
        ram8.tick(HIGH).unwrap();
        ram8.tock(LOW).unwrap();
        
        // Verify write worked
        ram8.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
        ram8.eval().unwrap();
        let output = ram8.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x1234, "RAM8[0] should contain written value");
        
        // Test write to different address
        ram8.get_pin("address").unwrap().borrow_mut().set_bus_voltage(7);
        ram8.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0x5678);
        ram8.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        
        ram8.tick(HIGH).unwrap();
        ram8.tock(LOW).unwrap();
        
        // Verify second write worked
        ram8.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
        ram8.eval().unwrap();
        let output = ram8.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x5678, "RAM8[7] should contain second written value");
        
        // Check first address is still intact
        ram8.get_pin("address").unwrap().borrow_mut().set_bus_voltage(0);
        ram8.eval().unwrap();
        let output = ram8.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x1234, "RAM8[0] should still contain first written value");
    }
    
    #[test]
    fn test_ram8_no_write_without_load() {
        let mut ram8 = Ram8Chip::new();
        
        // Set input and address but don't set load signal
        ram8.get_pin("address").unwrap().borrow_mut().set_bus_voltage(0);
        ram8.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0x1234);
        ram8.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
        
        // Simulate clock cycle
        ram8.tick(HIGH).unwrap();
        ram8.tock(LOW).unwrap();
        
        // Verify no write occurred
        ram8.eval().unwrap();
        let output = ram8.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0, "RAM8[0] should remain 0 without load signal");
    }
    
    #[test]
    fn test_ram8_address_masking() {
        let mut ram8 = Ram8Chip::new();
        
        // Test that addresses are properly masked to 3 bits
        // Address 8 (0b1000) should be masked to 0 (0b000)
        ram8.get_pin("address").unwrap().borrow_mut().set_bus_voltage(8);
        ram8.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0x9999);
        ram8.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        
        ram8.tick(HIGH).unwrap();
        ram8.tock(LOW).unwrap();
        
        // Check that value was written to address 0 (8 & 0b111 = 0)
        ram8.get_pin("address").unwrap().borrow_mut().set_bus_voltage(0);
        ram8.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
        ram8.eval().unwrap();
        let output = ram8.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x9999, "Address 8 should be masked to 0");
    }
    
    #[test]
    fn test_ram8_reset() {
        let mut ram8 = Ram8Chip::new();
        
        // Write some values
        for addr in 0..8 {
            ram8.get_pin("address").unwrap().borrow_mut().set_bus_voltage(addr);
            ram8.get_pin("in").unwrap().borrow_mut().set_bus_voltage((addr + 1) * 0x1111);
            ram8.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
            ram8.tick(HIGH).unwrap();
            ram8.tock(LOW).unwrap();
        }
        
        // Reset
        ram8.reset().unwrap();
        
        // Verify all values are reset to 0
        for addr in 0..8 {
            ram8.get_pin("address").unwrap().borrow_mut().set_bus_voltage(addr);
            ram8.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap(); // Ensure load is low for read-only
            ram8.eval().unwrap();
            let output = ram8.get_pin("out").unwrap().borrow().bus_voltage();
            assert_eq!(output, 0, "RAM8[{}] should be 0 after reset", addr);
        }
    }
}