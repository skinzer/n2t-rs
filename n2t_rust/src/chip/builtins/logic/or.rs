use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::chip::{ChipInterface, Bus, Pin};
use crate::chip::pin::{HIGH, LOW};
use crate::error::Result;
use super::super::{basic_chip_struct, impl_chip_interface_boilerplate};

basic_chip_struct!(OrChip);

impl OrChip {
    pub fn new() -> Self {
        let mut chip = Self {
            name: "Or".to_string(),
            input_pins: HashMap::new(),
            output_pins: HashMap::new(),
            internal_pins: HashMap::new(),
        };
        
        let a_pin = Rc::new(RefCell::new(Bus::new("a".to_string(), 1)));
        let b_pin = Rc::new(RefCell::new(Bus::new("b".to_string(), 1)));
        let out_pin = Rc::new(RefCell::new(Bus::new("out".to_string(), 1)));
        
        chip.input_pins.insert("a".to_string(), a_pin);
        chip.input_pins.insert("b".to_string(), b_pin);
        chip.output_pins.insert("out".to_string(), out_pin);
        
        chip
    }
}

impl ChipInterface for OrChip {
    impl_chip_interface_boilerplate!("OR");

    fn eval(&mut self) -> Result<()> {
        let a = self.input_pins["a"].borrow().voltage(None)?;
        let b = self.input_pins["b"].borrow().voltage(None)?;
        
        let output = if a == HIGH || b == HIGH { HIGH } else { LOW };
        
        self.output_pins["out"].borrow_mut().pull(output, None)?;
        
        Ok(())
    }
}