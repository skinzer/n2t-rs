use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::chip::{ChipInterface, Bus, Pin};
use crate::chip::pin::{HIGH, LOW};
use crate::error::Result;
use super::super::{basic_chip_struct, impl_chip_interface_boilerplate};

basic_chip_struct!(DMuxChip);

impl DMuxChip {
    pub fn new() -> Self {
        let mut chip = Self {
            name: "DMux".to_string(),
            input_pins: HashMap::new(),
            output_pins: HashMap::new(),
            internal_pins: HashMap::new(),
        };
        
        let in_pin = Rc::new(RefCell::new(Bus::new("in".to_string(), 1)));
        let sel_pin = Rc::new(RefCell::new(Bus::new("sel".to_string(), 1)));
        let a_pin = Rc::new(RefCell::new(Bus::new("a".to_string(), 1)));
        let b_pin = Rc::new(RefCell::new(Bus::new("b".to_string(), 1)));
        
        chip.input_pins.insert("in".to_string(), in_pin);
        chip.input_pins.insert("sel".to_string(), sel_pin);
        chip.output_pins.insert("a".to_string(), a_pin);
        chip.output_pins.insert("b".to_string(), b_pin);
        
        chip
    }
}

impl ChipInterface for DMuxChip {
    impl_chip_interface_boilerplate!("DMUX");

    fn eval(&mut self) -> Result<()> {
        let inn = self.input_pins["in"].borrow().voltage(None)?;
        let sel = self.input_pins["sel"].borrow().voltage(None)?;
        
        // DMux logic: route input to selected output
        let (a, b) = if sel == LOW {
            // Route to output 'a' when sel is LOW
            (if inn == HIGH { HIGH } else { LOW }, LOW)
        } else {
            // Route to output 'b' when sel is HIGH
            (LOW, if inn == HIGH { HIGH } else { LOW })
        };
        
        self.output_pins["a"].borrow_mut().pull(a, None)?;
        self.output_pins["b"].borrow_mut().pull(b, None)?;
        
        Ok(())
    }
}