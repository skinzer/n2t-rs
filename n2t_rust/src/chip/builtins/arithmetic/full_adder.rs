use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::chip::{ChipInterface, Bus, Pin};
use crate::chip::pin::{HIGH, LOW};
use crate::error::Result;
use super::super::{basic_chip_struct, impl_chip_interface_boilerplate};

basic_chip_struct!(FullAdderChip);

impl FullAdderChip {
    pub fn new() -> Self {
        let mut chip = Self {
            name: "FullAdder".to_string(),
            input_pins: HashMap::new(),
            output_pins: HashMap::new(),
            internal_pins: HashMap::new(),
        };
        
        let a_pin = Rc::new(RefCell::new(Bus::new("a".to_string(), 1)));
        let b_pin = Rc::new(RefCell::new(Bus::new("b".to_string(), 1)));
        let c_pin = Rc::new(RefCell::new(Bus::new("c".to_string(), 1)));
        let sum_pin = Rc::new(RefCell::new(Bus::new("sum".to_string(), 1)));
        let carry_pin = Rc::new(RefCell::new(Bus::new("carry".to_string(), 1)));
        
        chip.input_pins.insert("a".to_string(), a_pin);
        chip.input_pins.insert("b".to_string(), b_pin);
        chip.input_pins.insert("c".to_string(), c_pin);
        chip.output_pins.insert("sum".to_string(), sum_pin);
        chip.output_pins.insert("carry".to_string(), carry_pin);
        
        chip
    }
    
    // Helper function implementing half adder logic
    fn half_adder(a: u8, b: u8) -> (u8, u8) {
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
        
        (sum, carry)
    }
}

impl ChipInterface for FullAdderChip {
    impl_chip_interface_boilerplate!("FullAdder");
    
    fn eval(&mut self) -> Result<()> {
        let a = self.input_pins["a"].borrow().voltage(None)?;
        let b = self.input_pins["b"].borrow().voltage(None)?;
        let c = self.input_pins["c"].borrow().voltage(None)?;
        
        // Full adder logic using two half adders and an OR gate:
        // 1. First half adder: add a and b
        let (s, ca) = Self::half_adder(a, b);
        
        // 2. Second half adder: add s (from first half adder) and c
        let (sum, cb) = Self::half_adder(s, c);
        
        // 3. OR the two carry outputs
        let carry = if ca == HIGH || cb == HIGH {
            HIGH
        } else {
            LOW
        };
        
        self.output_pins["sum"].borrow_mut().pull(sum, None)?;
        self.output_pins["carry"].borrow_mut().pull(carry, None)?;
        
        Ok(())
    }
}