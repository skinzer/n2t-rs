use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::chip::{ChipInterface, Bus, Pin};
use crate::chip::pin::{HIGH, LOW};
use crate::error::Result;
use super::super::{basic_chip_struct, impl_chip_interface_boilerplate};

basic_chip_struct!(NotChip);

impl NotChip {
    pub fn new() -> Self {
        let mut chip = Self {
            name: "Not".to_string(),
            input_pins: HashMap::new(),
            output_pins: HashMap::new(),
            internal_pins: HashMap::new(),
        };
        
        let in_pin = Rc::new(RefCell::new(Bus::new("in".to_string(), 1)));
        let out_pin = Rc::new(RefCell::new(Bus::new("out".to_string(), 1)));
        
        chip.input_pins.insert("in".to_string(), in_pin);
        chip.output_pins.insert("out".to_string(), out_pin);
        
        chip
    }
}

impl ChipInterface for NotChip {
    impl_chip_interface_boilerplate!("NOT");

    fn eval(&mut self) -> Result<()> {
        let input = self.input_pins["in"].borrow().voltage(None)?;
        let output = if input == HIGH { LOW } else { HIGH };
        
        self.output_pins["out"].borrow_mut().pull(output, None)?;
        
        Ok(())
    }
}