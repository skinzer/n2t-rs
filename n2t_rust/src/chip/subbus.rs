// SubBus implementation for pin range operations
// Supports HDL syntax like a[0..7], a[8..15], a[5], etc.

use std::rc::Rc;
use std::cell::RefCell;
use crate::chip::pin::{Pin, Voltage};
use crate::error::{Result, SimulatorError};

/// Creates a bitmask with the specified number of bits
/// e.g., mask(3) returns 0b111 (7)
fn mask(bits: usize) -> u16 {
    if bits >= 16 {
        0xFFFF
    } else {
        (1u16 << bits) - 1
    }
}

/// SubBus for input connections - allows writing to a sub-range of a wider bus
/// Used when connecting TO input pins of internal parts
#[derive(Debug)]
pub struct InSubBus {
    name: String,
    parent_bus: Rc<RefCell<dyn Pin>>,
    start: usize,
    width: usize,
}

impl InSubBus {
    pub fn new(parent_bus: Rc<RefCell<dyn Pin>>, start: usize, width: usize) -> Result<Self> {
        let parent_width = parent_bus.borrow().width();
        
        if start + width > parent_width {
            return Err(SimulatorError::Hardware(format!(
                "SubBus range [{}..{}] exceeds parent bus width {} on pin '{}'",
                start, start + width - 1, parent_width, parent_bus.borrow().name()
            )).into());
        }
        
        let name = format!("{}[{}..{}]", parent_bus.borrow().name(), start, start + width - 1);
        
        Ok(Self {
            name,
            parent_bus,
            start,
            width,
        })
    }
    
    pub fn new_single_bit(parent_bus: Rc<RefCell<dyn Pin>>, bit: usize) -> Result<Self> {
        Self::new(parent_bus, bit, 1)
    }
}

impl Pin for InSubBus {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn width(&self) -> usize {
        self.width
    }
    
    fn bus_voltage(&self) -> u16 {
        let parent_voltage = self.parent_bus.borrow().bus_voltage();
        (parent_voltage >> self.start) & mask(self.width)
    }
    
    fn set_bus_voltage(&mut self, voltage: u16) {
        let mut parent = self.parent_bus.borrow_mut();
        let current_voltage = parent.bus_voltage();
        
        // Clear the bits we're about to write
        let clear_mask = !(mask(self.width) << self.start);
        let cleared = current_voltage & clear_mask;
        
        // Set the new bits
        let new_bits = (voltage & mask(self.width)) << self.start;
        let final_voltage = cleared | new_bits;
        
        parent.set_bus_voltage(final_voltage);
    }
    
    fn pull(&mut self, voltage: Voltage, bit: Option<usize>) -> Result<()> {
        let bit = bit.unwrap_or(0);
        if bit >= self.width {
            return Err(SimulatorError::Hardware(format!(
                "Bit index {} out of range for SubBus width {}", bit, self.width
            )).into());
        }
        
        self.parent_bus.borrow_mut().pull(voltage, Some(self.start + bit))
    }
    
    fn voltage(&self, bit: Option<usize>) -> Result<Voltage> {
        let bit = bit.unwrap_or(0);
        if bit >= self.width {
            return Err(SimulatorError::Hardware(format!(
                "Bit index {} out of range for SubBus width {}", bit, self.width
            )).into());
        }
        
        self.parent_bus.borrow().voltage(Some(self.start + bit))
    }
    
    fn connect(&mut self, pin: std::rc::Weak<RefCell<dyn Pin>>) {
        // SubBus connections are handled differently - they modify the parent bus
        if let Some(pin_rc) = pin.upgrade() {
            // Connect to parent bus instead
            self.parent_bus.borrow_mut().connect(Rc::downgrade(&pin_rc));
        }
    }
    
    fn toggle(&mut self, bit: Option<usize>) -> Result<()> {
        let bit = bit.unwrap_or(0);
        if bit >= self.width {
            return Err(SimulatorError::Hardware(format!(
                "Bit index {} out of range for SubBus width {}", bit, self.width
            )).into());
        }
        
        self.parent_bus.borrow_mut().toggle(Some(self.start + bit))
    }
}

/// SubBus for output connections - allows reading from a sub-range of a wider bus
/// Used when connecting FROM output pins of internal parts  
#[derive(Debug)]
pub struct OutSubBus {
    name: String,
    parent_bus: Rc<RefCell<dyn Pin>>,
    start: usize,
    width: usize,
    connections: Vec<std::rc::Weak<RefCell<dyn Pin>>>,
}

impl OutSubBus {
    pub fn new(parent_bus: Rc<RefCell<dyn Pin>>, start: usize, width: usize) -> Result<Self> {
        let parent_width = parent_bus.borrow().width();
        
        if start + width > parent_width {
            return Err(SimulatorError::Hardware(format!(
                "SubBus range [{}..{}] exceeds parent bus width {} on pin '{}'",
                start, start + width - 1, parent_width, parent_bus.borrow().name()
            )).into());
        }
        
        let name = format!("{}[{}..{}]", parent_bus.borrow().name(), start, start + width - 1);
        
        Ok(Self {
            name,
            parent_bus,
            start,
            width,
            connections: Vec::new(),
        })
    }
    
    pub fn new_single_bit(parent_bus: Rc<RefCell<dyn Pin>>, bit: usize) -> Result<Self> {
        Self::new(parent_bus, bit, 1)
    }
    
    /// Propagate the current SubBus value to all connected pins
    fn propagate_to_connections(&mut self, value: u16) {
        // Clean up dead connections first
        self.connections.retain(|weak_pin| weak_pin.strong_count() > 0);
        
        // Propagate to all connected pins
        for weak_pin in &self.connections {
            if let Some(pin_ref) = weak_pin.upgrade() {
                if let Ok(mut connected_pin) = pin_ref.try_borrow_mut() {
                    connected_pin.set_bus_voltage(value);
                }
            }
        }
    }
}

impl Pin for OutSubBus {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn width(&self) -> usize {
        self.width
    }
    
    fn bus_voltage(&self) -> u16 {
        let parent_voltage = self.parent_bus.borrow().bus_voltage();
        (parent_voltage >> self.start) & mask(self.width)
    }
    
    fn set_bus_voltage(&mut self, voltage: u16) {
        // OutSubBus typically shouldn't be written to directly
        // but we implement it for completeness and for triggering propagation
        
        // Get the current value that should be propagated
        let current_subbus_value = self.bus_voltage();
        
        // Propagate this value to all connected pins
        self.propagate_to_connections(current_subbus_value);
        
        // Also update the parent if voltage parameter is different
        if voltage != current_subbus_value {
            let mut parent = self.parent_bus.borrow_mut();
            let current_voltage = parent.bus_voltage();
            
            // Clear the bits we're about to write
            let clear_mask = !(mask(self.width) << self.start);
            let cleared = current_voltage & clear_mask;
            
            // Set the new bits
            let new_bits = (voltage & mask(self.width)) << self.start;
            let final_voltage = cleared | new_bits;
            
            parent.set_bus_voltage(final_voltage);
        }
    }
    
    fn pull(&mut self, voltage: Voltage, bit: Option<usize>) -> Result<()> {
        let bit = bit.unwrap_or(0);
        if bit >= self.width {
            return Err(SimulatorError::Hardware(format!(
                "Bit index {} out of range for SubBus width {}", bit, self.width
            )).into());
        }
        
        // For OutSubBus, pulls usually come from the parent, not to it
        // But we support it for flexibility
        self.parent_bus.borrow_mut().pull(voltage, Some(self.start + bit))
    }
    
    fn voltage(&self, bit: Option<usize>) -> Result<Voltage> {
        let bit = bit.unwrap_or(0);
        if bit >= self.width {
            return Err(SimulatorError::Hardware(format!(
                "Bit index {} out of range for SubBus width {}", bit, self.width
            )).into());
        }
        
        self.parent_bus.borrow().voltage(Some(self.start + bit))
    }
    
    fn connect(&mut self, pin: std::rc::Weak<RefCell<dyn Pin>>) {
        // OutSubBus maintains its own connections and propagates SubBus values
        if let Some(pin_ref) = pin.upgrade() {
            // Set initial voltage on the connected pin 
            if let Ok(mut connected_pin) = pin_ref.try_borrow_mut() {
                let current_value = self.bus_voltage();
                connected_pin.set_bus_voltage(current_value);
            }
        }
        self.connections.push(pin);
    }
    
    fn toggle(&mut self, bit: Option<usize>) -> Result<()> {
        let bit = bit.unwrap_or(0);
        if bit >= self.width {
            return Err(SimulatorError::Hardware(format!(
                "Bit index {} out of range for SubBus width {}", bit, self.width
            )).into());
        }
        
        self.parent_bus.borrow_mut().toggle(Some(self.start + bit))
    }
}

/// Parse pin range specification from HDL syntax
/// Supports: "pin", "pin[5]", "pin[0..7]"
#[derive(Debug, Clone, PartialEq)]
pub struct PinRange {
    pub pin_name: String,
    pub start: Option<usize>,
    pub end: Option<usize>,
}

impl PinRange {
    pub fn new(pin_name: String) -> Self {
        Self {
            pin_name,
            start: None,
            end: None,
        }
    }
    
    pub fn new_single_bit(pin_name: String, bit: usize) -> Self {
        Self {
            pin_name,
            start: Some(bit),
            end: Some(bit),
        }
    }
    
    pub fn new_range(pin_name: String, start: usize, end: usize) -> Result<Self> {
        if start > end {
            return Err(SimulatorError::Hardware(format!(
                "Invalid pin range: start {} > end {}", start, end
            )).into());
        }
        
        Ok(Self {
            pin_name,
            start: Some(start),
            end: Some(end),
        })
    }
    
    /// Get the width of this pin range
    pub fn width(&self) -> usize {
        match (self.start, self.end) {
            (Some(start), Some(end)) => end - start + 1,
            (None, None) => 1, // Full pin width - will be determined later
            _ => unreachable!(), // start and end should always be both Some or both None
        }
    }
    
    /// Check if this represents a full pin (no range specified)
    pub fn is_full_pin(&self) -> bool {
        self.start.is_none() && self.end.is_none()
    }
    
    /// Check if this represents a single bit
    pub fn is_single_bit(&self) -> bool {
        self.start == self.end && self.start.is_some()
    }
    
    /// Get the start index (0 if full pin)
    pub fn start_index(&self) -> usize {
        self.start.unwrap_or(0)
    }
    
    /// Get the end index (returns start if not specified)
    pub fn end_index(&self) -> usize {
        self.end.unwrap_or(self.start.unwrap_or(0))
    }
}

/// Utility functions for creating SubBus instances
pub fn create_input_subbus(
    parent_bus: Rc<RefCell<dyn Pin>>,
    range: &PinRange,
) -> Result<Rc<RefCell<dyn Pin>>> {
    if range.is_full_pin() {
        // No sub-range, return the full pin
        Ok(parent_bus)
    } else if range.is_single_bit() {
        // Single bit access
        let bit = range.start_index();
        let subbus = InSubBus::new_single_bit(parent_bus, bit)?;
        Ok(Rc::new(RefCell::new(subbus)) as Rc<RefCell<dyn Pin>>)
    } else {
        // Range access
        let start = range.start_index();
        let width = range.width();
        let subbus = InSubBus::new(parent_bus, start, width)?;
        Ok(Rc::new(RefCell::new(subbus)) as Rc<RefCell<dyn Pin>>)
    }
}

pub fn create_output_subbus(
    parent_bus: Rc<RefCell<dyn Pin>>,
    range: &PinRange,
) -> Result<Rc<RefCell<dyn Pin>>> {
    if range.is_full_pin() {
        // No sub-range, return the full pin
        Ok(parent_bus)
    } else if range.is_single_bit() {
        // Single bit access
        let bit = range.start_index();
        let subbus = OutSubBus::new_single_bit(parent_bus, bit)?;
        Ok(Rc::new(RefCell::new(subbus)) as Rc<RefCell<dyn Pin>>)
    } else {
        // Range access
        let start = range.start_index();
        let width = range.width();
        let subbus = OutSubBus::new(parent_bus, start, width)?;
        Ok(Rc::new(RefCell::new(subbus)) as Rc<RefCell<dyn Pin>>)
    }
}

/// Parse pin range from string (for testing and utilities)
/// Supports formats: "pin", "pin[5]", "pin[0..7]"
pub fn parse_pin_range(spec: &str) -> Result<PinRange> {
    if spec.is_empty() {
        return Err(SimulatorError::Parse("Empty pin specification".to_string()).into());
    }
    
    if !spec.contains('[') {
        // Simple pin name
        return Ok(PinRange::new(spec.to_string()));
    }
    
    // Extract pin name and range specification
    let parts: Vec<&str> = spec.split('[').collect();
    if parts.len() != 2 {
        return Err(SimulatorError::Parse(format!("Invalid pin specification: {}", spec)).into());
    }
    
    let pin_name = parts[0].to_string();
    if pin_name.is_empty() {
        return Err(SimulatorError::Parse("Missing pin name".to_string()).into());
    }
    
    if !parts[1].ends_with(']') {
        return Err(SimulatorError::Parse(format!("Missing closing bracket in: {}", spec)).into());
    }
    
    let range_part = parts[1].trim_end_matches(']');
    
    if range_part.contains("..") {
        // Range specification: pin[start..end]
        let range_parts: Vec<&str> = range_part.split("..").collect();
        if range_parts.len() != 2 {
            return Err(SimulatorError::Parse(format!("Invalid range specification: {}", range_part)).into());
        }
        
        let start: usize = range_parts[0].parse()
            .map_err(|_| SimulatorError::Parse(format!("Invalid start index: {}", range_parts[0])))?;
        let end: usize = range_parts[1].parse()
            .map_err(|_| SimulatorError::Parse(format!("Invalid end index: {}", range_parts[1])))?;
            
        // Auto-normalize reversed ranges
        let (normalized_start, normalized_end) = if start > end {
            (end, start)
        } else {
            (start, end)
        };
        
        PinRange::new_range(pin_name, normalized_start, normalized_end)
    } else {
        // Single bit specification: pin[bit]
        let bit: usize = range_part.parse()
            .map_err(|_| SimulatorError::Parse(format!("Invalid bit index: {}", range_part)))?;
        Ok(PinRange::new_single_bit(pin_name, bit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chip::Bus;
    use crate::chip::pin::{HIGH, LOW};
    
    #[test]
    fn test_mask_function() {
        assert_eq!(mask(0), 0);
        assert_eq!(mask(1), 0b1);
        assert_eq!(mask(3), 0b111);
        assert_eq!(mask(8), 0b11111111);
        assert_eq!(mask(16), 0xFFFF);
    }
    
    #[test]
    fn test_pin_range_parsing() {
        // Simple pin
        let range = parse_pin_range("a").unwrap();
        assert_eq!(range.pin_name, "a");
        assert!(range.is_full_pin());
        
        // Single bit
        let range = parse_pin_range("a[5]").unwrap();
        assert_eq!(range.pin_name, "a");
        assert_eq!(range.start, Some(5));
        assert_eq!(range.end, Some(5));
        assert!(range.is_single_bit());
        
        // Range
        let range = parse_pin_range("a[0..7]").unwrap();
        assert_eq!(range.pin_name, "a");
        assert_eq!(range.start, Some(0));
        assert_eq!(range.end, Some(7));
        assert_eq!(range.width(), 8);
    }
    
    #[test]
    fn test_in_subbus_single_bit() {
        let parent = Rc::new(RefCell::new(Bus::new("test".to_string(), 8)));
        
        // Set parent bus to some value
        parent.borrow_mut().set_bus_voltage(0b10101010);
        
        // Create SubBus for bit 1
        let mut subbus = InSubBus::new_single_bit(parent.clone(), 1).unwrap();
        
        // Should read bit 1 from parent (which is 1)
        assert_eq!(subbus.voltage(None).unwrap(), HIGH);
        assert_eq!(subbus.bus_voltage(), 1);
        
        // Write to the SubBus
        subbus.pull(LOW, None).unwrap();
        
        // Parent should now have bit 1 cleared
        assert_eq!(parent.borrow().bus_voltage(), 0b10101000);
    }
    
    #[test]
    fn test_in_subbus_range() {
        let parent = Rc::new(RefCell::new(Bus::new("test".to_string(), 16)));
        
        // Set parent bus to some value
        parent.borrow_mut().set_bus_voltage(0xABCD);
        
        // Create SubBus for bits 4..7 (4 bits)
        let mut subbus = InSubBus::new(parent.clone(), 4, 4).unwrap();
        
        // Should read bits 4..7 from parent (0xC from 0xABCD)
        assert_eq!(subbus.bus_voltage(), 0xC);
        
        // Write new value to SubBus
        subbus.set_bus_voltage(0x5);
        
        // Parent should have bits 4..7 changed to 0x5
        // Original: 0xABCD = 1010 1011 1100 1101
        // New:      0xAB5D = 1010 1011 0101 1101
        assert_eq!(parent.borrow().bus_voltage(), 0xAB5D);
    }
    
    #[test]
    fn test_out_subbus_range() {
        let parent = Rc::new(RefCell::new(Bus::new("test".to_string(), 16)));
        
        // Set parent bus to some value
        parent.borrow_mut().set_bus_voltage(0x1234);
        
        // Create SubBus for bits 8..11 (4 bits)
        let subbus = OutSubBus::new(parent.clone(), 8, 4).unwrap();
        
        // Should read bits 8..11 from parent (0x2 from 0x1234)
        assert_eq!(subbus.bus_voltage(), 0x2);
        
        // Individual bit access
        assert_eq!(subbus.voltage(Some(0)).unwrap(), LOW);  // bit 8
        assert_eq!(subbus.voltage(Some(1)).unwrap(), HIGH); // bit 9
        assert_eq!(subbus.voltage(Some(2)).unwrap(), LOW);  // bit 10
        assert_eq!(subbus.voltage(Some(3)).unwrap(), LOW);  // bit 11
    }
    
    #[test]
    fn test_subbus_bounds_checking() {
        let parent = Rc::new(RefCell::new(Bus::new("test".to_string(), 8)));
        
        // Should fail - range exceeds parent width
        let result = InSubBus::new(parent.clone(), 6, 4);
        assert!(result.is_err());
        
        // Should fail - single bit out of range
        let result = InSubBus::new_single_bit(parent, 8);
        assert!(result.is_err());
    }
}