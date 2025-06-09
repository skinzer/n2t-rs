use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::chip::{ChipInterface, Bus, Pin};
use crate::chip::pin::LOW;
use crate::error::Result;
use super::super::{basic_chip_struct, impl_chip_interface_boilerplate};

basic_chip_struct!(MuxChip);

impl MuxChip {
    pub fn new() -> Self {
        let mut chip = Self {
            name: "Mux".to_string(),
            input_pins: HashMap::new(),
            output_pins: HashMap::new(),
            internal_pins: HashMap::new(),
        };
        
        let a_pin = Rc::new(RefCell::new(Bus::new("a".to_string(), 1)));
        let b_pin = Rc::new(RefCell::new(Bus::new("b".to_string(), 1)));
        let sel_pin = Rc::new(RefCell::new(Bus::new("sel".to_string(), 1)));
        let out_pin = Rc::new(RefCell::new(Bus::new("out".to_string(), 1)));
        
        chip.input_pins.insert("a".to_string(), a_pin);
        chip.input_pins.insert("b".to_string(), b_pin);
        chip.input_pins.insert("sel".to_string(), sel_pin);
        chip.output_pins.insert("out".to_string(), out_pin);
        
        chip
    }
}

impl ChipInterface for MuxChip {
    impl_chip_interface_boilerplate!("MUX");

    fn eval(&mut self) -> Result<()> {
        let a = self.input_pins["a"].borrow().voltage(None)?;
        let b = self.input_pins["b"].borrow().voltage(None)?;
        let sel = self.input_pins["sel"].borrow().voltage(None)?;
        
        // Mux logic: output = sel ? b : a
        let output = if sel == LOW { a } else { b };
        
        self.output_pins["out"].borrow_mut().pull(output, None)?;
        
        Ok(())
    }
}