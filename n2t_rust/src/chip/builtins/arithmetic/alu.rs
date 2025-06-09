use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::chip::{ChipInterface, Bus, Pin};
use crate::chip::pin::{HIGH, LOW};
use crate::error::Result;
use super::super::{basic_chip_struct, impl_chip_interface_boilerplate};

// Flags enum for ALU output status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AluFlags {
    Positive = 0x01,
    Zero = 0x00,
    Negative = 0x0f,
}

basic_chip_struct!(AluChip);

impl AluChip {
    pub fn new() -> Self {
        let mut chip = Self {
            name: "ALU".to_string(),
            input_pins: HashMap::new(),
            output_pins: HashMap::new(),
            internal_pins: HashMap::new(),
        };
        
        // Create 16-bit input buses
        let x_pin = Rc::new(RefCell::new(Bus::new("x".to_string(), 16)));
        let y_pin = Rc::new(RefCell::new(Bus::new("y".to_string(), 16)));
        
        // Create control signal inputs (1-bit each)
        let zx_pin = Rc::new(RefCell::new(Bus::new("zx".to_string(), 1)));
        let nx_pin = Rc::new(RefCell::new(Bus::new("nx".to_string(), 1)));
        let zy_pin = Rc::new(RefCell::new(Bus::new("zy".to_string(), 1)));
        let ny_pin = Rc::new(RefCell::new(Bus::new("ny".to_string(), 1)));
        let f_pin = Rc::new(RefCell::new(Bus::new("f".to_string(), 1)));
        let no_pin = Rc::new(RefCell::new(Bus::new("no".to_string(), 1)));
        
        // Create output pins
        let out_pin = Rc::new(RefCell::new(Bus::new("out".to_string(), 16)));
        let zr_pin = Rc::new(RefCell::new(Bus::new("zr".to_string(), 1)));
        let ng_pin = Rc::new(RefCell::new(Bus::new("ng".to_string(), 1)));
        
        // Add input pins
        chip.input_pins.insert("x".to_string(), x_pin);
        chip.input_pins.insert("y".to_string(), y_pin);
        chip.input_pins.insert("zx".to_string(), zx_pin);
        chip.input_pins.insert("nx".to_string(), nx_pin);
        chip.input_pins.insert("zy".to_string(), zy_pin);
        chip.input_pins.insert("ny".to_string(), ny_pin);
        chip.input_pins.insert("f".to_string(), f_pin);
        chip.input_pins.insert("no".to_string(), no_pin);
        
        // Add output pins
        chip.output_pins.insert("out".to_string(), out_pin);
        chip.output_pins.insert("zr".to_string(), zr_pin);
        chip.output_pins.insert("ng".to_string(), ng_pin);
        
        chip
    }
    
    // ALU implementation following the alua function from TypeScript
    fn alu_operation(op: u16, mut x: u16, mut y: u16) -> (u16, AluFlags) {
        // Apply control signals to inputs
        if op & 0b100000 != 0 { x = 0; }           // zx: zero x
        if op & 0b010000 != 0 { x = !x & 0xffff; } // nx: negate x
        if op & 0b001000 != 0 { y = 0; }           // zy: zero y
        if op & 0b000100 != 0 { y = !y & 0xffff; } // ny: negate y
        
        // Compute operation: f=1 means addition, f=0 means AND
        let mut result = if op & 0b000010 != 0 {
            x.wrapping_add(y) & 0xffff  // Addition with overflow handling
        } else {
            x & y  // Bitwise AND
        };
        
        // Apply output negation if no=1
        if op & 0b000001 != 0 {
            result = !result & 0xffff;
        }
        
        // Determine flags
        let flags = if result == 0 {
            AluFlags::Zero
        } else if result & 0x8000 != 0 {  // Check sign bit (bit 15)
            AluFlags::Negative
        } else {
            AluFlags::Positive
        };
        
        (result, flags)
    }
}

impl ChipInterface for AluChip {
    impl_chip_interface_boilerplate!("ALU");
    
    fn eval(&mut self) -> Result<()> {
        // Get input values
        let x = self.input_pins["x"].borrow().bus_voltage();
        let y = self.input_pins["y"].borrow().bus_voltage();
        
        // Get control signals and build operation code
        let zx = if self.input_pins["zx"].borrow().voltage(None)? == HIGH { 1u16 } else { 0u16 };
        let nx = if self.input_pins["nx"].borrow().voltage(None)? == HIGH { 1u16 } else { 0u16 };
        let zy = if self.input_pins["zy"].borrow().voltage(None)? == HIGH { 1u16 } else { 0u16 };
        let ny = if self.input_pins["ny"].borrow().voltage(None)? == HIGH { 1u16 } else { 0u16 };
        let f = if self.input_pins["f"].borrow().voltage(None)? == HIGH { 1u16 } else { 0u16 };
        let no = if self.input_pins["no"].borrow().voltage(None)? == HIGH { 1u16 } else { 0u16 };
        
        // Build operation code (6-bit control word)
        let op = (zx << 5) + (nx << 4) + (zy << 3) + (ny << 2) + (f << 1) + no;
        
        // Perform ALU operation
        let (result, flags) = Self::alu_operation(op, x, y);
        
        // Set outputs
        self.output_pins["out"].borrow_mut().set_bus_voltage(result);
        
        // Set flag outputs
        let zr_out = if flags == AluFlags::Zero { HIGH } else { LOW };
        let ng_out = if flags == AluFlags::Negative { HIGH } else { LOW };
        
        self.output_pins["zr"].borrow_mut().pull(zr_out, None)?;
        self.output_pins["ng"].borrow_mut().pull(ng_out, None)?;
        
        Ok(())
    }
}