use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::chip::{ChipInterface, Bus, Pin};
use crate::chip::pin::{LOW};
use crate::error::Result;
use super::super::{basic_chip_struct, impl_chip_interface_boilerplate};

basic_chip_struct!(Mux16Chip);

impl Mux16Chip {
    pub fn new() -> Self {
        let mut chip = Self {
            name: "Mux16".to_string(),
            input_pins: HashMap::new(),
            output_pins: HashMap::new(),
            internal_pins: HashMap::new(),
        };
        
        let a_pin = Rc::new(RefCell::new(Bus::new("a".to_string(), 16)));
        let b_pin = Rc::new(RefCell::new(Bus::new("b".to_string(), 16)));
        let sel_pin = Rc::new(RefCell::new(Bus::new("sel".to_string(), 1)));
        let out_pin = Rc::new(RefCell::new(Bus::new("out".to_string(), 16)));
        
        chip.input_pins.insert("a".to_string(), a_pin);
        chip.input_pins.insert("b".to_string(), b_pin);
        chip.input_pins.insert("sel".to_string(), sel_pin);
        chip.output_pins.insert("out".to_string(), out_pin);
        
        chip
    }
}

impl ChipInterface for Mux16Chip {
    impl_chip_interface_boilerplate!("MUX16");

    fn eval(&mut self) -> Result<()> {
        let a = self.input_pins["a"].borrow().bus_voltage();
        let b = self.input_pins["b"].borrow().bus_voltage();
        let sel = self.input_pins["sel"].borrow().voltage(None)?;
        
        // Mux16 logic: output = sel ? b : a
        let output = if sel == LOW { a } else { b };
        
        self.output_pins["out"].borrow_mut().set_bus_voltage(output);
        
        Ok(())
    }
}

basic_chip_struct!(Mux4Way16Chip);

impl Mux4Way16Chip {
    pub fn new() -> Self {
        let mut chip = Self {
            name: "Mux4Way16".to_string(),
            input_pins: HashMap::new(),
            output_pins: HashMap::new(),
            internal_pins: HashMap::new(),
        };
        
        let a_pin = Rc::new(RefCell::new(Bus::new("a".to_string(), 16)));
        let b_pin = Rc::new(RefCell::new(Bus::new("b".to_string(), 16)));
        let c_pin = Rc::new(RefCell::new(Bus::new("c".to_string(), 16)));
        let d_pin = Rc::new(RefCell::new(Bus::new("d".to_string(), 16)));
        let sel_pin = Rc::new(RefCell::new(Bus::new("sel".to_string(), 2)));
        let out_pin = Rc::new(RefCell::new(Bus::new("out".to_string(), 16)));
        
        chip.input_pins.insert("a".to_string(), a_pin);
        chip.input_pins.insert("b".to_string(), b_pin);
        chip.input_pins.insert("c".to_string(), c_pin);
        chip.input_pins.insert("d".to_string(), d_pin);
        chip.input_pins.insert("sel".to_string(), sel_pin);
        chip.output_pins.insert("out".to_string(), out_pin);
        
        chip
    }
}

impl ChipInterface for Mux4Way16Chip {
    impl_chip_interface_boilerplate!("MUX4WAY16");

    fn eval(&mut self) -> Result<()> {
        let a = self.input_pins["a"].borrow().bus_voltage();
        let b = self.input_pins["b"].borrow().bus_voltage();
        let c = self.input_pins["c"].borrow().bus_voltage();
        let d = self.input_pins["d"].borrow().bus_voltage();
        let sel = self.input_pins["sel"].borrow().bus_voltage();
        
        // Mux4Way16 logic: select one of 4 inputs based on 2-bit selector
        let output = match sel & 0b11 {
            0b00 => a,
            0b01 => b,
            0b10 => c,
            0b11 => d,
            _ => unreachable!(),
        };
        
        self.output_pins["out"].borrow_mut().set_bus_voltage(output);
        
        Ok(())
    }
}

basic_chip_struct!(Mux8Way16Chip);

impl Mux8Way16Chip {
    pub fn new() -> Self {
        let mut chip = Self {
            name: "Mux8Way16".to_string(),
            input_pins: HashMap::new(),
            output_pins: HashMap::new(),
            internal_pins: HashMap::new(),
        };
        
        let a_pin = Rc::new(RefCell::new(Bus::new("a".to_string(), 16)));
        let b_pin = Rc::new(RefCell::new(Bus::new("b".to_string(), 16)));
        let c_pin = Rc::new(RefCell::new(Bus::new("c".to_string(), 16)));
        let d_pin = Rc::new(RefCell::new(Bus::new("d".to_string(), 16)));
        let e_pin = Rc::new(RefCell::new(Bus::new("e".to_string(), 16)));
        let f_pin = Rc::new(RefCell::new(Bus::new("f".to_string(), 16)));
        let g_pin = Rc::new(RefCell::new(Bus::new("g".to_string(), 16)));
        let h_pin = Rc::new(RefCell::new(Bus::new("h".to_string(), 16)));
        let sel_pin = Rc::new(RefCell::new(Bus::new("sel".to_string(), 3)));
        let out_pin = Rc::new(RefCell::new(Bus::new("out".to_string(), 16)));
        
        chip.input_pins.insert("a".to_string(), a_pin);
        chip.input_pins.insert("b".to_string(), b_pin);
        chip.input_pins.insert("c".to_string(), c_pin);
        chip.input_pins.insert("d".to_string(), d_pin);
        chip.input_pins.insert("e".to_string(), e_pin);
        chip.input_pins.insert("f".to_string(), f_pin);
        chip.input_pins.insert("g".to_string(), g_pin);
        chip.input_pins.insert("h".to_string(), h_pin);
        chip.input_pins.insert("sel".to_string(), sel_pin);
        chip.output_pins.insert("out".to_string(), out_pin);
        
        chip
    }
}

impl ChipInterface for Mux8Way16Chip {
    impl_chip_interface_boilerplate!("MUX8WAY16");

    fn eval(&mut self) -> Result<()> {
        let a = self.input_pins["a"].borrow().bus_voltage();
        let b = self.input_pins["b"].borrow().bus_voltage();
        let c = self.input_pins["c"].borrow().bus_voltage();
        let d = self.input_pins["d"].borrow().bus_voltage();
        let e = self.input_pins["e"].borrow().bus_voltage();
        let f = self.input_pins["f"].borrow().bus_voltage();
        let g = self.input_pins["g"].borrow().bus_voltage();
        let h = self.input_pins["h"].borrow().bus_voltage();
        let sel = self.input_pins["sel"].borrow().bus_voltage();
        
        // Mux8Way16 logic: select one of 8 inputs based on 3-bit selector
        let output = match sel & 0b111 {
            0b000 => a,
            0b001 => b,
            0b010 => c,
            0b011 => d,
            0b100 => e,
            0b101 => f,
            0b110 => g,
            0b111 => h,
            _ => unreachable!(),
        };
        
        self.output_pins["out"].borrow_mut().set_bus_voltage(output);
        
        Ok(())
    }
}