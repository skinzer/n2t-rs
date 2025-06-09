use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::chip::{ChipInterface, Bus, Pin};
use crate::error::Result;
use super::super::{basic_chip_struct, impl_chip_interface_boilerplate};

basic_chip_struct!(Inc16Chip);

impl Inc16Chip {
    pub fn new() -> Self {
        let mut chip = Self {
            name: "Inc16".to_string(),
            input_pins: HashMap::new(),
            output_pins: HashMap::new(),
            internal_pins: HashMap::new(),
        };
        
        let in_pin = Rc::new(RefCell::new(Bus::new("in".to_string(), 16)));
        let out_pin = Rc::new(RefCell::new(Bus::new("out".to_string(), 16)));
        
        chip.input_pins.insert("in".to_string(), in_pin);
        chip.output_pins.insert("out".to_string(), out_pin);
        
        chip
    }
}

impl ChipInterface for Inc16Chip {
    impl_chip_interface_boilerplate!("Inc16");
    
    fn eval(&mut self) -> Result<()> {
        let n = self.input_pins["in"].borrow().bus_voltage();
        
        // Increment the 16-bit value with wrapping to handle overflow
        let output = n.wrapping_add(1) & 0xffff;
        
        self.output_pins["out"].borrow_mut().set_bus_voltage(output);
        Ok(())
    }
}