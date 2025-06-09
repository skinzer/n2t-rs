use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::chip::{ChipInterface, Bus, Pin};
use crate::chip::pin::{HIGH, LOW};
use crate::error::Result;
use super::super::{basic_chip_struct, impl_chip_interface_boilerplate};

basic_chip_struct!(HalfAdderChip);

impl HalfAdderChip {
    pub fn new() -> Self {
        let mut chip = Self {
            name: "HalfAdder".to_string(),
            input_pins: HashMap::new(),
            output_pins: HashMap::new(),
            internal_pins: HashMap::new(),
        };
        
        let a_pin = Rc::new(RefCell::new(Bus::new("a".to_string(), 1)));
        let b_pin = Rc::new(RefCell::new(Bus::new("b".to_string(), 1)));
        let sum_pin = Rc::new(RefCell::new(Bus::new("sum".to_string(), 1)));
        let carry_pin = Rc::new(RefCell::new(Bus::new("carry".to_string(), 1)));
        
        chip.input_pins.insert("a".to_string(), a_pin);
        chip.input_pins.insert("b".to_string(), b_pin);
        chip.output_pins.insert("sum".to_string(), sum_pin);
        chip.output_pins.insert("carry".to_string(), carry_pin);
        
        chip
    }
}

impl ChipInterface for HalfAdderChip {
    impl_chip_interface_boilerplate!("HalfAdder");
    
    fn eval(&mut self) -> Result<()> {
        let a = self.input_pins["a"].borrow().voltage(None)?;
        let b = self.input_pins["b"].borrow().voltage(None)?;
        
        // Half adder logic:
        // sum = a XOR b (true when exactly one input is HIGH)
        // carry = a AND b (true when both inputs are HIGH)
        let sum = if (a == HIGH && b == LOW) || (a == LOW && b == HIGH) {
            HIGH
        } else {
            LOW
        };
        
        let carry = if a == HIGH && b == HIGH {
            HIGH
        } else {
            LOW
        };
        
        self.output_pins["sum"].borrow_mut().pull(sum, None)?;
        self.output_pins["carry"].borrow_mut().pull(carry, None)?;
        
        Ok(())
    }
}