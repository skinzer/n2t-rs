use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::chip::{ChipInterface, Bus, Pin};
use crate::chip::pin::{HIGH, LOW};
use crate::error::Result;
use super::super::{basic_chip_struct, impl_chip_interface_boilerplate};

basic_chip_struct!(DMux4WayChip);

impl DMux4WayChip {
    pub fn new() -> Self {
        let mut chip = Self {
            name: "DMux4Way".to_string(),
            input_pins: HashMap::new(),
            output_pins: HashMap::new(),
            internal_pins: HashMap::new(),
        };
        
        let in_pin = Rc::new(RefCell::new(Bus::new("in".to_string(), 1)));
        let sel_pin = Rc::new(RefCell::new(Bus::new("sel".to_string(), 2)));
        let a_pin = Rc::new(RefCell::new(Bus::new("a".to_string(), 1)));
        let b_pin = Rc::new(RefCell::new(Bus::new("b".to_string(), 1)));
        let c_pin = Rc::new(RefCell::new(Bus::new("c".to_string(), 1)));
        let d_pin = Rc::new(RefCell::new(Bus::new("d".to_string(), 1)));
        
        chip.input_pins.insert("in".to_string(), in_pin);
        chip.input_pins.insert("sel".to_string(), sel_pin);
        chip.output_pins.insert("a".to_string(), a_pin);
        chip.output_pins.insert("b".to_string(), b_pin);
        chip.output_pins.insert("c".to_string(), c_pin);
        chip.output_pins.insert("d".to_string(), d_pin);
        
        chip
    }
}

impl ChipInterface for DMux4WayChip {
    impl_chip_interface_boilerplate!("DMUX4WAY");

    fn eval(&mut self) -> Result<()> {
        let inn = self.input_pins["in"].borrow().voltage(None)?;
        let sel = self.input_pins["sel"].borrow().bus_voltage();
        
        // DMux4Way logic: route input to one of 4 outputs based on 2-bit selector
        let (a, b, c, d) = match sel & 0b11 {
            0b00 => (if inn == HIGH { HIGH } else { LOW }, LOW, LOW, LOW),
            0b01 => (LOW, if inn == HIGH { HIGH } else { LOW }, LOW, LOW),
            0b10 => (LOW, LOW, if inn == HIGH { HIGH } else { LOW }, LOW),
            0b11 => (LOW, LOW, LOW, if inn == HIGH { HIGH } else { LOW }),
            _ => unreachable!(),
        };
        
        self.output_pins["a"].borrow_mut().pull(a, None)?;
        self.output_pins["b"].borrow_mut().pull(b, None)?;
        self.output_pins["c"].borrow_mut().pull(c, None)?;
        self.output_pins["d"].borrow_mut().pull(d, None)?;
        
        Ok(())
    }
}

basic_chip_struct!(DMux8WayChip);

impl DMux8WayChip {
    pub fn new() -> Self {
        let mut chip = Self {
            name: "DMux8Way".to_string(),
            input_pins: HashMap::new(),
            output_pins: HashMap::new(),
            internal_pins: HashMap::new(),
        };
        
        let in_pin = Rc::new(RefCell::new(Bus::new("in".to_string(), 1)));
        let sel_pin = Rc::new(RefCell::new(Bus::new("sel".to_string(), 3)));
        let a_pin = Rc::new(RefCell::new(Bus::new("a".to_string(), 1)));
        let b_pin = Rc::new(RefCell::new(Bus::new("b".to_string(), 1)));
        let c_pin = Rc::new(RefCell::new(Bus::new("c".to_string(), 1)));
        let d_pin = Rc::new(RefCell::new(Bus::new("d".to_string(), 1)));
        let e_pin = Rc::new(RefCell::new(Bus::new("e".to_string(), 1)));
        let f_pin = Rc::new(RefCell::new(Bus::new("f".to_string(), 1)));
        let g_pin = Rc::new(RefCell::new(Bus::new("g".to_string(), 1)));
        let h_pin = Rc::new(RefCell::new(Bus::new("h".to_string(), 1)));
        
        chip.input_pins.insert("in".to_string(), in_pin);
        chip.input_pins.insert("sel".to_string(), sel_pin);
        chip.output_pins.insert("a".to_string(), a_pin);
        chip.output_pins.insert("b".to_string(), b_pin);
        chip.output_pins.insert("c".to_string(), c_pin);
        chip.output_pins.insert("d".to_string(), d_pin);
        chip.output_pins.insert("e".to_string(), e_pin);
        chip.output_pins.insert("f".to_string(), f_pin);
        chip.output_pins.insert("g".to_string(), g_pin);
        chip.output_pins.insert("h".to_string(), h_pin);
        
        chip
    }
}

impl ChipInterface for DMux8WayChip {
    impl_chip_interface_boilerplate!("DMUX8WAY");

    fn eval(&mut self) -> Result<()> {
        let inn = self.input_pins["in"].borrow().voltage(None)?;
        let sel = self.input_pins["sel"].borrow().bus_voltage();
        
        // DMux8Way logic: route input to one of 8 outputs based on 3-bit selector
        let (a, b, c, d, e, f, g, h) = match sel & 0b111 {
            0b000 => (if inn == HIGH { HIGH } else { LOW }, LOW, LOW, LOW, LOW, LOW, LOW, LOW),
            0b001 => (LOW, if inn == HIGH { HIGH } else { LOW }, LOW, LOW, LOW, LOW, LOW, LOW),
            0b010 => (LOW, LOW, if inn == HIGH { HIGH } else { LOW }, LOW, LOW, LOW, LOW, LOW),
            0b011 => (LOW, LOW, LOW, if inn == HIGH { HIGH } else { LOW }, LOW, LOW, LOW, LOW),
            0b100 => (LOW, LOW, LOW, LOW, if inn == HIGH { HIGH } else { LOW }, LOW, LOW, LOW),
            0b101 => (LOW, LOW, LOW, LOW, LOW, if inn == HIGH { HIGH } else { LOW }, LOW, LOW),
            0b110 => (LOW, LOW, LOW, LOW, LOW, LOW, if inn == HIGH { HIGH } else { LOW }, LOW),
            0b111 => (LOW, LOW, LOW, LOW, LOW, LOW, LOW, if inn == HIGH { HIGH } else { LOW }),
            _ => unreachable!(),
        };
        
        self.output_pins["a"].borrow_mut().pull(a, None)?;
        self.output_pins["b"].borrow_mut().pull(b, None)?;
        self.output_pins["c"].borrow_mut().pull(c, None)?;
        self.output_pins["d"].borrow_mut().pull(d, None)?;
        self.output_pins["e"].borrow_mut().pull(e, None)?;
        self.output_pins["f"].borrow_mut().pull(f, None)?;
        self.output_pins["g"].borrow_mut().pull(g, None)?;
        self.output_pins["h"].borrow_mut().pull(h, None)?;
        
        Ok(())
    }
}