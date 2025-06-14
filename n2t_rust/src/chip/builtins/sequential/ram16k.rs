use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::chip::{ChipInterface, Clock, Bus, Pin};
use crate::chip::pin::{Voltage, HIGH};
use crate::error::Result;
use tokio::sync::broadcast;
use super::{ClockedChip};
use super::memory::Memory;

/// RAM16K - 16384-register RAM using 14-bit address
#[derive(Debug)]
pub struct Ram16kChip {
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

impl Ram16kChip {
    pub fn new() -> Self {
        let mut input_pins = HashMap::new();
        let mut output_pins = HashMap::new();
        
        // Create pins with trait object casting
        input_pins.insert("in".to_string(), Rc::new(RefCell::new(Bus::new("in".to_string(), 16))) as Rc<RefCell<dyn Pin>>);
        input_pins.insert("load".to_string(), Rc::new(RefCell::new(Bus::new("load".to_string(), 1))) as Rc<RefCell<dyn Pin>>);
        input_pins.insert("address".to_string(), Rc::new(RefCell::new(Bus::new("address".to_string(), 14))) as Rc<RefCell<dyn Pin>>);
        output_pins.insert("out".to_string(), Rc::new(RefCell::new(Bus::new("out".to_string(), 16))) as Rc<RefCell<dyn Pin>>);
        
        Self {
            name: "RAM16K".to_string(),
            input_pins,
            output_pins,
            internal_pins: HashMap::new(),
            clock_subscriber: None,
            memory: Memory::new(16384), // 2^14 = 16384 registers
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

impl ChipInterface for Ram16kChip {
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
        let address = address & 0b11111111111111; // Mask to 14 bits for RAM16K
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

impl ClockedChip for Ram16kChip {
    fn tick(&mut self, _clock_level: Voltage) -> Result<()> {
        // Rising edge: sample inputs and conditionally write to memory
        let load = self.input_pins["load"].borrow().voltage(None)?;
        let address = self.input_pins["address"].borrow().bus_voltage() as usize;
        self.current_address = address & 0b11111111111111; // Mask to 14 bits for RAM16K
        
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

impl Default for Ram16kChip {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chip::pin::{HIGH, LOW};
    
    #[test]
    fn test_ram16k_basic_structure() {
        let ram16k = Ram16kChip::new();
        
        // Test basic properties
        assert_eq!(ram16k.name(), "RAM16K");
        assert!(ram16k.get_pin("in").is_ok());
        assert!(ram16k.get_pin("address").is_ok());
        assert!(ram16k.get_pin("load").is_ok());
        assert!(ram16k.get_pin("out").is_ok());
        
        // Test memory size
        assert_eq!(ram16k.memory().size(), 16384);
    }
    
    #[test]
    fn test_ram16k_sequential_write_read() {
        let mut ram16k = Ram16kChip::new();
        
        // Test write operation at address 0
        ram16k.get_pin("address").unwrap().borrow_mut().set_bus_voltage(0);
        ram16k.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0x1234);
        ram16k.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        
        // Simulate clock cycle (tick for write, tock for output update)
        ram16k.tick(HIGH).unwrap();
        ram16k.tock(LOW).unwrap();
        
        // Verify write worked
        ram16k.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
        ram16k.eval().unwrap();
        let output = ram16k.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x1234, "RAM16K[0] should contain written value");
        
        // Test write to different address (edge of range)
        ram16k.get_pin("address").unwrap().borrow_mut().set_bus_voltage(16383);
        ram16k.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0x5678);
        ram16k.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        
        ram16k.tick(HIGH).unwrap();
        ram16k.tock(LOW).unwrap();
        
        // Verify second write worked
        ram16k.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
        ram16k.eval().unwrap();
        let output = ram16k.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x5678, "RAM16K[16383] should contain second written value");
        
        // Check first address is still intact
        ram16k.get_pin("address").unwrap().borrow_mut().set_bus_voltage(0);
        ram16k.eval().unwrap();
        let output = ram16k.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x1234, "RAM16K[0] should still contain first written value");
    }
    
    #[test]
    fn test_ram16k_address_masking() {
        let mut ram16k = Ram16kChip::new();
        
        // Test that addresses are properly masked to 14 bits
        // Address 16384 (0b100000000000000) should be masked to 0 (0b00000000000000)
        ram16k.get_pin("address").unwrap().borrow_mut().set_bus_voltage(16384);
        ram16k.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0x9999);
        ram16k.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        
        ram16k.tick(HIGH).unwrap();
        ram16k.tock(LOW).unwrap();
        
        // Check that value was written to address 0 (16384 & 0b11111111111111 = 0)
        ram16k.get_pin("address").unwrap().borrow_mut().set_bus_voltage(0);
        ram16k.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
        ram16k.eval().unwrap();
        let output = ram16k.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x9999, "Address 16384 should be masked to 0");
    }
    
    #[test]
    fn test_ram16k_boundary_addresses() {
        let mut ram16k = Ram16kChip::new();
        
        // Test writes to boundary addresses including powers of 2
        let test_addresses = [0, 1, 1023, 1024, 2047, 2048, 4095, 4096, 8191, 8192, 16383];
        let test_values = [0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6666, 0x7777, 0x8888, 0x9999, 0xAAAA, 0xBBBB];
        
        // Write to all test addresses
        for (i, &addr) in test_addresses.iter().enumerate() {
            ram16k.get_pin("address").unwrap().borrow_mut().set_bus_voltage(addr);
            ram16k.get_pin("in").unwrap().borrow_mut().set_bus_voltage(test_values[i]);
            ram16k.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
            ram16k.tick(HIGH).unwrap();
            ram16k.tock(LOW).unwrap();
        }
        
        // Verify all values were stored correctly
        for (i, &addr) in test_addresses.iter().enumerate() {
            ram16k.get_pin("address").unwrap().borrow_mut().set_bus_voltage(addr);
            ram16k.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
            ram16k.eval().unwrap();
            let output = ram16k.get_pin("out").unwrap().borrow().bus_voltage();
            assert_eq!(output, test_values[i], "RAM16K[{}] should contain correct value", addr);
        }
    }
}