use std::rc::{Rc, Weak};
use std::cell::RefCell;
use crate::chip::pin::{Pin, Voltage, HIGH, LOW};
use crate::error::{Result, SimulatorError};

#[derive(Debug)]
pub struct Bus {
    name: String,
    width: usize,
    state: Vec<Voltage>,
    connections: Vec<Weak<RefCell<dyn Pin>>>,
}

impl Bus {
    pub fn new(name: String, width: usize) -> Self {
        assert!(width > 0 && width <= 16, "Bus width must be between 1 and 16 bits");
        
        Self {
            name,
            width,
            state: vec![LOW; width],
            connections: Vec::new(),
        }
    }
    
    pub fn ensure_width(&mut self, new_width: usize) -> Result<()> {
        if new_width > 16 {
            return Err(SimulatorError::Hardware(
                format!("Cannot widen bus past 16 bits to {} bits", new_width)
            ));
        }
        
        if self.width < new_width {
            self.state.resize(new_width, LOW);
            self.width = new_width;
        }
        
        Ok(())
    }
    
    fn propagate_voltage(&mut self, voltage: Voltage, bit: usize) {
        // Remove dead weak references
        self.connections.retain(|weak_pin| weak_pin.strong_count() > 0);
        
        // Propagate to connected pins
        for weak_pin in &self.connections {
            if let Some(pin_ref) = weak_pin.upgrade() {
                if let Ok(mut pin) = pin_ref.try_borrow_mut() {
                    let _ = pin.pull(voltage, Some(bit));
                }
            }
        }
    }
    
    fn propagate_bus_voltage(&mut self, voltage: u16) {
        // Remove dead weak references
        self.connections.retain(|weak_pin| weak_pin.strong_count() > 0);
        
        // Propagate to connected pins
        for weak_pin in &self.connections {
            if let Some(pin_ref) = weak_pin.upgrade() {
                if let Ok(mut pin) = pin_ref.try_borrow_mut() {
                    pin.set_bus_voltage(voltage);
                }
            }
        }
    }
}

impl Pin for Bus {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn width(&self) -> usize {
        self.width
    }
    
    fn bus_voltage(&self) -> u16 {
        let mut result = 0u16;
        for (i, &voltage) in self.state.iter().enumerate() {
            if voltage == HIGH {
                result |= 1 << i;
            }
        }
        result
    }
    
    fn set_bus_voltage(&mut self, voltage: u16) {
        for i in 0..self.width {
            self.state[i] = if (voltage & (1 << i)) != 0 { HIGH } else { LOW };
        }
        self.propagate_bus_voltage(voltage);
    }
    
    fn pull(&mut self, voltage: Voltage, bit: Option<usize>) -> Result<()> {
        let bit = bit.unwrap_or(0);
        
        if bit >= self.width {
            return Err(SimulatorError::Hardware(
                format!("Bit {} out of bounds for bus {} (width {})", bit, self.name, self.width)
            ));
        }
        
        self.state[bit] = voltage;
        self.propagate_voltage(voltage, bit);
        
        Ok(())
    }
    
    fn toggle(&mut self, bit: Option<usize>) -> Result<()> {
        let bit = bit.unwrap_or(0);
        let current = self.voltage(Some(bit))?;
        let new_voltage = if current == LOW { HIGH } else { LOW };
        self.pull(new_voltage, Some(bit))
    }
    
    fn voltage(&self, bit: Option<usize>) -> Result<Voltage> {
        let bit = bit.unwrap_or(0);
        
        if bit >= self.width {
            return Err(SimulatorError::Hardware(
                format!("Bit {} out of bounds for bus {} (width {})", bit, self.name, self.width)
            ));
        }
        
        Ok(self.state[bit])
    }
    
    fn connect(&mut self, pin: Weak<RefCell<dyn Pin>>) {
        // Set initial voltage on connected pin
        if let Some(pin_ref) = pin.upgrade() {
            if let Ok(mut pin_mut) = pin_ref.try_borrow_mut() {
                pin_mut.set_bus_voltage(self.bus_voltage());
            }
        }
        
        self.connections.push(pin);
    }
}

pub struct SubBus {
    parent: Rc<RefCell<dyn Pin>>,
    start: usize,
    width: usize,
    name: String,
}

impl SubBus {
    pub fn new(parent: Rc<RefCell<dyn Pin>>, start: usize, width: usize) -> Result<Self> {
        let parent_width = parent.borrow().width();
        
        if start + width > parent_width {
            return Err(SimulatorError::Hardware(
                format!("SubBus range [{}, {}) exceeds parent width {}", 
                       start, start + width, parent_width)
            ));
        }
        
        let name = format!("{}[{}..{}]", parent.borrow().name(), start, start + width - 1);
        
        Ok(Self {
            parent,
            start,
            width,
            name,
        })
    }
}

impl Pin for SubBus {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn width(&self) -> usize {
        self.width
    }
    
    fn bus_voltage(&self) -> u16 {
        let parent_voltage = self.parent.borrow().bus_voltage();
        let mask = (1 << self.width) - 1;
        ((parent_voltage >> self.start) & mask) as u16
    }
    
    fn set_bus_voltage(&mut self, voltage: u16) {
        let mask = (1 << self.width) - 1;
        let shifted_voltage = (voltage & mask) << self.start;
        let parent_mask = !((mask) << self.start);
        
        let current_parent = self.parent.borrow().bus_voltage();
        let new_parent = (current_parent & parent_mask) | shifted_voltage;
        
        self.parent.borrow_mut().set_bus_voltage(new_parent);
    }
    
    fn pull(&mut self, voltage: Voltage, bit: Option<usize>) -> Result<()> {
        let bit = bit.unwrap_or(0);
        if bit >= self.width {
            return Err(SimulatorError::Hardware(
                format!("Bit {} out of bounds for SubBus {} (width {})", bit, self.name, self.width)
            ));
        }
        
        self.parent.borrow_mut().pull(voltage, Some(self.start + bit))
    }
    
    fn toggle(&mut self, bit: Option<usize>) -> Result<()> {
        let bit = bit.unwrap_or(0);
        if bit >= self.width {
            return Err(SimulatorError::Hardware(
                format!("Bit {} out of bounds for SubBus {} (width {})", bit, self.name, self.width)
            ));
        }
        
        self.parent.borrow_mut().toggle(Some(self.start + bit))
    }
    
    fn voltage(&self, bit: Option<usize>) -> Result<Voltage> {
        let bit = bit.unwrap_or(0);
        if bit >= self.width {
            return Err(SimulatorError::Hardware(
                format!("Bit {} out of bounds for SubBus {} (width {})", bit, self.name, self.width)
            ));
        }
        
        self.parent.borrow().voltage(Some(self.start + bit))
    }
    
    fn connect(&mut self, pin: Weak<RefCell<dyn Pin>>) {
        self.parent.borrow_mut().connect(pin);
    }
}

use std::fmt;

impl fmt::Debug for SubBus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubBus")
            .field("name", &self.name)
            .field("start", &self.start)
            .field("width", &self.width)
            .finish()
    }
}