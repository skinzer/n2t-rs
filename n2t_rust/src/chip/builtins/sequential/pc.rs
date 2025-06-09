use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::chip::{ChipInterface, Clock, Bus, Pin};
use crate::chip::pin::{Voltage, HIGH};
use crate::error::Result;
use tokio::sync::broadcast;
use super::ClockedChip;

/// Program Counter - 16-bit register with increment, load, and reset
#[derive(Debug)]
pub struct PcChip {
    name: String,
    input_pins: HashMap<String, Rc<RefCell<dyn Pin>>>,
    output_pins: HashMap<String, Rc<RefCell<dyn Pin>>>,
    internal_pins: HashMap<String, Rc<RefCell<dyn Pin>>>,
    clock_subscriber: Option<broadcast::Receiver<crate::chip::clock::ClockTick>>,
    // State - 16-bit counter
    bits: u16,
}

impl PcChip {
    pub fn new() -> Self {
        let mut input_pins = HashMap::new();
        let mut output_pins = HashMap::new();
        
        // Create pins with trait object casting
        input_pins.insert("in".to_string(), Rc::new(RefCell::new(Bus::new("in".to_string(), 16))) as Rc<RefCell<dyn Pin>>);
        input_pins.insert("reset".to_string(), Rc::new(RefCell::new(Bus::new("reset".to_string(), 1))) as Rc<RefCell<dyn Pin>>);
        input_pins.insert("load".to_string(), Rc::new(RefCell::new(Bus::new("load".to_string(), 1))) as Rc<RefCell<dyn Pin>>);
        input_pins.insert("inc".to_string(), Rc::new(RefCell::new(Bus::new("inc".to_string(), 1))) as Rc<RefCell<dyn Pin>>);
        output_pins.insert("out".to_string(), Rc::new(RefCell::new(Bus::new("out".to_string(), 16))) as Rc<RefCell<dyn Pin>>);
        
        Self {
            name: "PC".to_string(),
            input_pins,
            output_pins,
            internal_pins: HashMap::new(),
            clock_subscriber: None,
            bits: 0,
        }
    }
    
    pub fn subscribe_to_clock(&mut self, clock: &Clock) {
        self.clock_subscriber = Some(clock.subscribe());
    }
}

impl ChipInterface for PcChip {
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
        // Output current state (combinatorial read)
        self.output_pins["out"].borrow_mut().set_bus_voltage(self.bits);
        Ok(())
    }
    
    fn reset(&mut self) -> Result<()> {
        self.bits = 0;
        self.output_pins["out"].borrow_mut().set_bus_voltage(0);
        Ok(())
    }
}

impl ClockedChip for PcChip {
    fn tick(&mut self, _clock_level: Voltage) -> Result<()> {
        // Rising edge: update based on control signals
        // Priority: reset > load > increment
        
        let reset = self.input_pins["reset"].borrow().voltage(None)?;
        let load = self.input_pins["load"].borrow().voltage(None)?;
        let inc = self.input_pins["inc"].borrow().voltage(None)?;
        
        if reset == HIGH {
            // Reset has highest priority
            self.bits = 0;
        } else if load == HIGH {
            // Load has second priority
            let input_value = self.input_pins["in"].borrow().bus_voltage();
            self.bits = input_value & 0xffff;
        } else if inc == HIGH {
            // Increment has lowest priority
            self.bits = (self.bits.wrapping_add(1)) & 0xffff;
        }
        // If none of the control signals are high, maintain current value
        
        Ok(())
    }
    
    fn tock(&mut self, _clock_level: Voltage) -> Result<()> {
        // Falling edge: update output
        self.output_pins["out"].borrow_mut().set_bus_voltage(self.bits);
        Ok(())
    }
}

impl Default for PcChip {
    fn default() -> Self {
        Self::new()
    }
}