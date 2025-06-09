use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::chip::{ChipInterface, Bus, Pin};
use crate::error::Result;
use super::super::{basic_chip_struct, impl_chip_interface_boilerplate};

basic_chip_struct!(Add16Chip);

impl Add16Chip {
    pub fn new() -> Self {
        let mut chip = Self {
            name: "Add16".to_string(),
            input_pins: HashMap::new(),
            output_pins: HashMap::new(),
            internal_pins: HashMap::new(),
        };
        
        let a_pin = Rc::new(RefCell::new(Bus::new("a".to_string(), 16)));
        let b_pin = Rc::new(RefCell::new(Bus::new("b".to_string(), 16)));
        let out_pin = Rc::new(RefCell::new(Bus::new("out".to_string(), 16)));
        
        chip.input_pins.insert("a".to_string(), a_pin);
        chip.input_pins.insert("b".to_string(), b_pin);
        chip.output_pins.insert("out".to_string(), out_pin);
        
        chip
    }
}

impl ChipInterface for Add16Chip {
    impl_chip_interface_boilerplate!("Add16");
    
    fn eval(&mut self) -> Result<()> {
        let a = self.input_pins["a"].borrow().bus_voltage();
        let b = self.input_pins["b"].borrow().bus_voltage();
        
        // Add the two 16-bit values with wrapping to handle overflow
        let output = a.wrapping_add(b) & 0xffff;
        
        self.output_pins["out"].borrow_mut().set_bus_voltage(output);
        Ok(())
    }
}