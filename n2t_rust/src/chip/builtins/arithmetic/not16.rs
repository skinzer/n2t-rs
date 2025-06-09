use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::chip::{ChipInterface, Bus, Pin};
use crate::error::Result;
use super::super::{basic_chip_struct, impl_chip_interface_boilerplate};

basic_chip_struct!(Not16Chip);

impl Not16Chip {
    pub fn new() -> Self {
        let mut chip = Self {
            name: "Not16".to_string(),
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

impl ChipInterface for Not16Chip {
    impl_chip_interface_boilerplate!("NOT16");

    fn eval(&mut self) -> Result<()> {
        let input = self.input_pins["in"].borrow().bus_voltage();
        let output = !input; // Bitwise NOT on 16-bit value
        
        self.output_pins["out"].borrow_mut().set_bus_voltage(output);
        
        Ok(())
    }
}