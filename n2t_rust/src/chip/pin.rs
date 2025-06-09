use std::rc::Weak;
use std::cell::RefCell;
use crate::error::{Result, SimulatorError};

pub const HIGH: Voltage = 1;
pub const LOW: Voltage = 0;

pub type Voltage = u8;

pub trait Pin: std::fmt::Debug {
    fn name(&self) -> &str;
    fn width(&self) -> usize;
    fn bus_voltage(&self) -> u16;
    fn set_bus_voltage(&mut self, voltage: u16);
    fn pull(&mut self, voltage: Voltage, bit: Option<usize>) -> Result<()>;
    fn toggle(&mut self, bit: Option<usize>) -> Result<()>;
    fn voltage(&self, bit: Option<usize>) -> Result<Voltage>;
    fn connect(&mut self, pin: Weak<RefCell<dyn Pin>>);
}

pub fn is_constant_pin(pin_name: &str) -> bool {
    matches!(pin_name, "false" | "true" | "0" | "1")
}

#[derive(Debug)]
pub struct ConstantPin {
    name: String,
    voltage: Voltage,
}

impl ConstantPin {
    pub fn new(name: String) -> Result<Self> {
        let voltage = match name.as_str() {
            "false" | "0" => LOW,
            "true" | "1" => HIGH,
            _ => return Err(SimulatorError::Hardware(format!("Invalid constant pin name: {}", name))),
        };
        
        Ok(Self { name, voltage })
    }
}

impl Pin for ConstantPin {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn width(&self) -> usize {
        1
    }
    
    fn bus_voltage(&self) -> u16 {
        self.voltage as u16
    }
    
    fn set_bus_voltage(&mut self, _voltage: u16) {
        // Constants cannot be modified
    }
    
    fn pull(&mut self, _voltage: Voltage, _bit: Option<usize>) -> Result<()> {
        // Constants cannot be pulled
        Ok(())
    }
    
    fn toggle(&mut self, _bit: Option<usize>) -> Result<()> {
        // Constants cannot be toggled
        Ok(())
    }
    
    fn voltage(&self, bit: Option<usize>) -> Result<Voltage> {
        let bit = bit.unwrap_or(0);
        if bit >= self.width() {
            return Err(SimulatorError::Hardware(
                format!("Bit {} out of bounds for pin {} (width {})", bit, self.name, self.width())
            ));
        }
        Ok(self.voltage)
    }
    
    fn connect(&mut self, _pin: Weak<RefCell<dyn Pin>>) {
        // Constants don't need connections
    }
}