use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::chip::{ChipInterface, Clock, Bus, Pin};
use crate::chip::pin::{Voltage, LOW};
use crate::error::Result;
use tokio::sync::broadcast;
use super::ClockedChip;

/// D Flip-Flop - fundamental sequential building block
/// On tick: samples input, on tock: outputs previous input
#[derive(Debug)]
pub struct DffChip {
    name: String,
    input_pins: HashMap<String, Rc<RefCell<dyn Pin>>>,
    output_pins: HashMap<String, Rc<RefCell<dyn Pin>>>,
    internal_pins: HashMap<String, Rc<RefCell<dyn Pin>>>,
    clock_subscriber: Option<broadcast::Receiver<crate::chip::clock::ClockTick>>,
    // Internal state for two-phase clocking
    stored_value: Voltage,
}

impl DffChip {
    pub fn new() -> Self {
        let mut input_pins = HashMap::new();
        let mut output_pins = HashMap::new();
        let mut internal_pins = HashMap::new();
        
        // Create pins with trait object casting
        input_pins.insert("in".to_string(), Rc::new(RefCell::new(Bus::new("in".to_string(), 1))) as Rc<RefCell<dyn Pin>>);
        output_pins.insert("out".to_string(), Rc::new(RefCell::new(Bus::new("out".to_string(), 1))) as Rc<RefCell<dyn Pin>>);
        internal_pins.insert("t".to_string(), Rc::new(RefCell::new(Bus::new("t".to_string(), 1))) as Rc<RefCell<dyn Pin>>);
        
        Self {
            name: "DFF".to_string(),
            input_pins,
            output_pins,
            internal_pins,
            clock_subscriber: None,
            stored_value: LOW,
        }
    }
    
    pub fn subscribe_to_clock(&mut self, clock: &Clock) {
        self.clock_subscriber = Some(clock.subscribe());
    }
}

impl ChipInterface for DffChip {
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
        if let Some(pin) = self.internal_pins.get(name) {
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
        // DFF is sequential - evaluation happens in tick/tock, not here
        // This is called for combinatorial updates but DFF doesn't respond immediately
        Ok(())
    }
    
    fn reset(&mut self) -> Result<()> {
        self.stored_value = LOW;
        self.output_pins["out"].borrow_mut().pull(LOW, None)?;
        Ok(())
    }
}

impl ClockedChip for DffChip {
    fn tick(&mut self, _clock_level: Voltage) -> Result<()> {
        // Rising edge: sample input and store it
        let input_value = self.input_pins["in"].borrow().voltage(None)?;
        self.stored_value = input_value;
        
        // Store in internal pin for debugging/inspection
        self.internal_pins["t"].borrow_mut().pull(input_value, None)?;
        
        Ok(())
    }
    
    fn tock(&mut self, _clock_level: Voltage) -> Result<()> {
        // Falling edge: output the stored value
        self.output_pins["out"].borrow_mut().pull(self.stored_value, None)?;
        Ok(())
    }
}

impl Default for DffChip {
    fn default() -> Self {
        Self::new()
    }
}