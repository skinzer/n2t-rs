use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use crate::chip::{Chip, ChipInterface, Bus, Pin};
use crate::chip::pin::{ConstantPin, is_constant_pin};
use crate::chip::builtins::*;
use crate::languages::hdl::{HdlChip, PinDecl, Part, Wire, WireSide};
use crate::error::{Result, SimulatorError};

// Pin type methods are now implemented by the builtins using their own macros

pub struct ChipBuilder {
    builtin_registry: HashMap<String, Box<dyn Fn() -> Box<dyn ChipInterface>>>,
}

impl ChipBuilder {
    pub fn new() -> Self {
        let mut builder = Self {
            builtin_registry: HashMap::new(),
        };
        
        // Register builtin chips
        builder.register_builtins();
        builder
    }
    
    pub fn build_chip(&self, hdl_chip: &HdlChip) -> Result<Box<dyn ChipInterface>> {
        if hdl_chip.is_builtin {
            return self.build_builtin_chip(&hdl_chip.name);
        }
        
        let mut chip = Chip::new(hdl_chip.name.clone());
        
        // Create input pins
        for input in &hdl_chip.inputs {
            let pin = self.create_pin_from_decl(input)?;
            chip.add_input_pin(input.name.clone(), pin);
        }
        
        // Create output pins
        for output in &hdl_chip.outputs {
            let pin = self.create_pin_from_decl(output)?;
            chip.add_output_pin(output.name.clone(), pin);
        }
        
        // Create internal pins and sub-chips
        self.build_parts(&mut chip, &hdl_chip.parts)?;
        
        Ok(Box::new(chip))
    }
    
    fn create_pin_from_decl(&self, pin_decl: &PinDecl) -> Result<Rc<RefCell<dyn Pin>>> {
        let width = pin_decl.width.unwrap_or(1) as usize;
        let bus = Bus::new(pin_decl.name.clone(), width);
        Ok(Rc::new(RefCell::new(bus)))
    }
    
    fn build_parts(&self, chip: &mut Chip, parts: &[Part]) -> Result<()> {
        // Track all internal pins needed
        let mut internal_pins: HashMap<String, Rc<RefCell<dyn Pin>>> = HashMap::new();
        
        // First pass: identify all internal pins
        for part in parts {
            for wire in &part.connections {
                self.collect_internal_pins(&mut internal_pins, &wire.from, chip)?;
                self.collect_internal_pins(&mut internal_pins, &wire.to, chip)?;
            }
        }
        
        // Add internal pins to chip
        for (name, pin) in internal_pins {
            chip.add_internal_pin(name, pin);
        }
        
        // Second pass: build sub-chips and connect them
        for part in parts {
            let sub_chip = self.build_builtin_chip(&part.name)?;
            self.connect_part(chip, sub_chip.as_ref(), &part.connections)?;
            chip.add_sub_chip(sub_chip);
        }
        
        Ok(())
    }
    
    fn collect_internal_pins(
        &self,
        internal_pins: &mut HashMap<String, Rc<RefCell<dyn Pin>>>,
        wire_side: &WireSide,
        chip: &Chip,
    ) -> Result<()> {
        if let WireSide::Pin { name, .. } = wire_side {
            // Check if this pin is already an input or output
            if chip.input_pins().contains_key(name) || chip.output_pins().contains_key(name) {
                return Ok(());
            }
            
            // Check if it's a constant
            if is_constant_pin(name) {
                return Ok(());
            }
            
            // Add as internal pin if not already present
            if !internal_pins.contains_key(name) {
                let bus = Bus::new(name.clone(), 1); // Default width, will be adjusted if needed
                internal_pins.insert(name.clone(), Rc::new(RefCell::new(bus)));
            }
        }
        
        Ok(())
    }
    
    fn connect_part(
        &self,
        chip: &Chip,
        _sub_chip: &dyn ChipInterface,
        connections: &[Wire],
    ) -> Result<()> {
        for wire in connections {
            let from_pin = self.resolve_wire_side(chip, &wire.from)?;
            let to_pin = self.resolve_wire_side(chip, &wire.to)?;
            
            // Connect the pins
            let weak_to = Rc::downgrade(&to_pin);
            from_pin.borrow_mut().connect(weak_to);
        }
        
        Ok(())
    }
    
    fn resolve_wire_side(
        &self,
        chip: &Chip,
        wire_side: &WireSide,
    ) -> Result<Rc<RefCell<dyn Pin>>> {
        match wire_side {
            WireSide::Pin { name, range: _ } => {
                // Check constants first
                if is_constant_pin(name) {
                    let constant = ConstantPin::new(name.clone())?;
                    return Ok(Rc::new(RefCell::new(constant)));
                }
                
                // Try to find in chip pins
                chip.get_pin(name)
            }
            WireSide::Constant(value) => {
                let constant_name = if *value { "true" } else { "false" };
                let constant = ConstantPin::new(constant_name.to_string())?;
                Ok(Rc::new(RefCell::new(constant)))
            }
        }
    }
    
    pub fn build_builtin_chip(&self, name: &str) -> Result<Box<dyn ChipInterface>> {
        if let Some(factory) = self.builtin_registry.get(name) {
            Ok(factory())
        } else {
            Err(SimulatorError::Hardware(format!("Unknown builtin chip: {}", name)))
        }
    }
    
    fn register_builtins(&mut self) {
        // Register basic logic gates
        self.builtin_registry.insert("Nand".to_string(), Box::new(|| {
            Box::new(NandChip::new())
        }));
        
        self.builtin_registry.insert("Not".to_string(), Box::new(|| {
            Box::new(NotChip::new())
        }));
        
        self.builtin_registry.insert("And".to_string(), Box::new(|| {
            Box::new(AndChip::new())
        }));
        
        self.builtin_registry.insert("Or".to_string(), Box::new(|| {
            Box::new(OrChip::new())
        }));
        
        self.builtin_registry.insert("Xor".to_string(), Box::new(|| {
            Box::new(XorChip::new())
        }));
        
        self.builtin_registry.insert("Mux".to_string(), Box::new(|| {
            Box::new(MuxChip::new())
        }));
        
        self.builtin_registry.insert("DMux".to_string(), Box::new(|| {
            Box::new(DMuxChip::new())
        }));
        
        self.builtin_registry.insert("DMux4Way".to_string(), Box::new(|| {
            Box::new(DMux4WayChip::new())
        }));
        
        self.builtin_registry.insert("DMux8Way".to_string(), Box::new(|| {
            Box::new(DMux8WayChip::new())
        }));
        
        // Register 16-bit chips
        self.builtin_registry.insert("Not16".to_string(), Box::new(|| {
            Box::new(Not16Chip::new())
        }));
        
        self.builtin_registry.insert("And16".to_string(), Box::new(|| {
            Box::new(And16Chip::new())
        }));
        
        self.builtin_registry.insert("Or16".to_string(), Box::new(|| {
            Box::new(Or16Chip::new())
        }));
        
        self.builtin_registry.insert("Mux16".to_string(), Box::new(|| {
            Box::new(Mux16Chip::new())
        }));
        
        self.builtin_registry.insert("Mux4Way16".to_string(), Box::new(|| {
            Box::new(Mux4Way16Chip::new())
        }));
        
        self.builtin_registry.insert("Mux8Way16".to_string(), Box::new(|| {
            Box::new(Mux8Way16Chip::new())
        }));
        
        self.builtin_registry.insert("Add16".to_string(), Box::new(|| {
            Box::new(Add16Chip::new())
        }));
        
        self.builtin_registry.insert("Inc16".to_string(), Box::new(|| {
            Box::new(Inc16Chip::new())
        }));
        
        self.builtin_registry.insert("HalfAdder".to_string(), Box::new(|| {
            Box::new(HalfAdderChip::new())
        }));
        
        self.builtin_registry.insert("FullAdder".to_string(), Box::new(|| {
            Box::new(FullAdderChip::new())
        }));
        
        self.builtin_registry.insert("ALU".to_string(), Box::new(|| {
            Box::new(AluChip::new())
        }));
        
        // Register sequential chips
        self.builtin_registry.insert("DFF".to_string(), Box::new(|| {
            Box::new(DffChip::new())
        }));
        
        self.builtin_registry.insert("Bit".to_string(), Box::new(|| {
            Box::new(BitChip::new())
        }));
        
        self.builtin_registry.insert("Register".to_string(), Box::new(|| {
            Box::new(RegisterChip::new())
        }));
        
        self.builtin_registry.insert("PC".to_string(), Box::new(|| {
            Box::new(PcChip::new())
        }));
        
        self.builtin_registry.insert("RAM8".to_string(), Box::new(|| {
            Box::new(Ram8Chip::new())
        }));
        
        self.builtin_registry.insert("RAM64".to_string(), Box::new(|| {
            Box::new(Ram64Chip::new())
        }));
        
        self.builtin_registry.insert("RAM512".to_string(), Box::new(|| {
            Box::new(Ram512Chip::new())
        }));
        
        self.builtin_registry.insert("RAM4K".to_string(), Box::new(|| {
            Box::new(Ram4kChip::new())
        }));
        
        self.builtin_registry.insert("RAM16K".to_string(), Box::new(|| {
            Box::new(Ram16kChip::new())
        }));
        
        self.builtin_registry.insert("ROM32K".to_string(), Box::new(|| {
            Box::new(Rom32kChip::new())
        }));
        
        self.builtin_registry.insert("Screen".to_string(), Box::new(|| {
            Box::new(ScreenChip::new())
        }));
        
        self.builtin_registry.insert("Keyboard".to_string(), Box::new(|| {
            Box::new(KeyboardChip::new())
        }));
    }
}

impl Default for ChipBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Builtin chip implementations are now in the builtins/ module


#[cfg(test)]
mod tests {
    use super::*;
    use crate::chip::pin::{HIGH, LOW};
    use crate::languages::hdl::{HdlParser};
    
    #[test]
    fn test_builtin_nand_chip() {
        let builder = ChipBuilder::new();
        let mut nand_chip = builder.build_builtin_chip("Nand").unwrap();
        
        // Test NAND truth table
        let test_cases = [
            (LOW, LOW, HIGH),   // 0 NAND 0 = 1
            (LOW, HIGH, HIGH),  // 0 NAND 1 = 1
            (HIGH, LOW, HIGH),  // 1 NAND 0 = 1
            (HIGH, HIGH, LOW),  // 1 NAND 1 = 0
        ];
        
        for (a_val, b_val, expected) in test_cases {
            // Set inputs
            nand_chip.get_pin("a").unwrap().borrow_mut().pull(a_val, None).unwrap();
            nand_chip.get_pin("b").unwrap().borrow_mut().pull(b_val, None).unwrap();
            
            // Evaluate
            nand_chip.eval().unwrap();
            
            // Check output
            let output = nand_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
            assert_eq!(output, expected, "NAND({}, {}) should be {}", a_val, b_val, expected);
        }
    }
    
    #[test]
    fn test_builtin_not_chip() {
        let builder = ChipBuilder::new();
        let mut not_chip = builder.build_builtin_chip("Not").unwrap();
        
        // Test NOT truth table
        let test_cases = [
            (LOW, HIGH),   // NOT 0 = 1
            (HIGH, LOW),   // NOT 1 = 0
        ];
        
        for (input_val, expected) in test_cases {
            not_chip.get_pin("in").unwrap().borrow_mut().pull(input_val, None).unwrap();
            not_chip.eval().unwrap();
            
            let output = not_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
            assert_eq!(output, expected, "NOT({}) should be {}", input_val, expected);
        }
    }
    
    #[test]
    fn test_builtin_and_chip() {
        let builder = ChipBuilder::new();
        let mut and_chip = builder.build_builtin_chip("And").unwrap();
        
        // Test AND truth table
        let test_cases = [
            (LOW, LOW, LOW),    // 0 AND 0 = 0
            (LOW, HIGH, LOW),   // 0 AND 1 = 0
            (HIGH, LOW, LOW),   // 1 AND 0 = 0
            (HIGH, HIGH, HIGH), // 1 AND 1 = 1
        ];
        
        for (a_val, b_val, expected) in test_cases {
            and_chip.get_pin("a").unwrap().borrow_mut().pull(a_val, None).unwrap();
            and_chip.get_pin("b").unwrap().borrow_mut().pull(b_val, None).unwrap();
            and_chip.eval().unwrap();
            
            let output = and_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
            assert_eq!(output, expected, "AND({}, {}) should be {}", a_val, b_val, expected);
        }
    }
    
    #[test]
    fn test_build_chip_from_hdl() {
        let builder = ChipBuilder::new();
        let mut parser = HdlParser::new().unwrap();
        
        let hdl = r#"
            CHIP Not {
                IN in;
                OUT out;
                BUILTIN;
            }
        "#;
        
        let hdl_chip = parser.parse(hdl).unwrap();
        let chip = builder.build_chip(&hdl_chip).unwrap();
        
        assert_eq!(chip.name(), "Not");
        assert_eq!(chip.input_pins().len(), 1);
        assert_eq!(chip.output_pins().len(), 1);
        assert!(chip.input_pins().contains_key("in"));
        assert!(chip.output_pins().contains_key("out"));
    }
    
    #[test]
    fn test_builtin_or_chip() {
        let builder = ChipBuilder::new();
        let mut or_chip = builder.build_builtin_chip("Or").unwrap();
        
        // Test OR truth table
        let test_cases = [
            (LOW, LOW, LOW),    // 0 OR 0 = 0
            (LOW, HIGH, HIGH),  // 0 OR 1 = 1
            (HIGH, LOW, HIGH),  // 1 OR 0 = 1
            (HIGH, HIGH, HIGH), // 1 OR 1 = 1
        ];
        
        for (a_val, b_val, expected) in test_cases {
            or_chip.get_pin("a").unwrap().borrow_mut().pull(a_val, None).unwrap();
            or_chip.get_pin("b").unwrap().borrow_mut().pull(b_val, None).unwrap();
            or_chip.eval().unwrap();
            
            let output = or_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
            assert_eq!(output, expected, "OR({}, {}) should be {}", a_val, b_val, expected);
        }
    }
    
    #[test]
    fn test_builtin_xor_chip() {
        let builder = ChipBuilder::new();
        let mut xor_chip = builder.build_builtin_chip("Xor").unwrap();
        
        // Test XOR truth table
        let test_cases = [
            (LOW, LOW, LOW),    // 0 XOR 0 = 0
            (LOW, HIGH, HIGH),  // 0 XOR 1 = 1
            (HIGH, LOW, HIGH),  // 1 XOR 0 = 1
            (HIGH, HIGH, LOW),  // 1 XOR 1 = 0
        ];
        
        for (a_val, b_val, expected) in test_cases {
            xor_chip.get_pin("a").unwrap().borrow_mut().pull(a_val, None).unwrap();
            xor_chip.get_pin("b").unwrap().borrow_mut().pull(b_val, None).unwrap();
            xor_chip.eval().unwrap();
            
            let output = xor_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
            assert_eq!(output, expected, "XOR({}, {}) should be {}", a_val, b_val, expected);
        }
    }
    
    #[test]
    fn test_builtin_not16_chip() {
        let builder = ChipBuilder::new();
        let mut not16_chip = builder.build_builtin_chip("Not16").unwrap();
        
        // Test from TypeScript: inn.busVoltage = 0x0; expect(out.busVoltage).toBe(0xffff);
        not16_chip.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0x0);
        not16_chip.eval().unwrap();
        let output = not16_chip.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0xffff, "NOT16(0x0000) should be 0xFFFF");
        
        // Test from TypeScript: inn.busVoltage = 0xf00f; expect(out.busVoltage).toBe(0x0ff0);
        not16_chip.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0xf00f);
        not16_chip.eval().unwrap();
        let output = not16_chip.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x0ff0, "NOT16(0xF00F) should be 0x0FF0");
    }
    
    #[test]
    fn test_builtin_and16_chip() {
        let builder = ChipBuilder::new();
        let mut and16_chip = builder.build_builtin_chip("And16").unwrap();
        
        // Test 16-bit AND operations
        and16_chip.get_pin("a").unwrap().borrow_mut().set_bus_voltage(0xffff);
        and16_chip.get_pin("b").unwrap().borrow_mut().set_bus_voltage(0x0000);
        and16_chip.eval().unwrap();
        let output = and16_chip.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x0000, "AND16(0xFFFF, 0x0000) should be 0x0000");
        
        and16_chip.get_pin("a").unwrap().borrow_mut().set_bus_voltage(0xf0f0);
        and16_chip.get_pin("b").unwrap().borrow_mut().set_bus_voltage(0x0f0f);
        and16_chip.eval().unwrap();
        let output = and16_chip.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x0000, "AND16(0xF0F0, 0x0F0F) should be 0x0000");
        
        and16_chip.get_pin("a").unwrap().borrow_mut().set_bus_voltage(0xabcd);
        and16_chip.get_pin("b").unwrap().borrow_mut().set_bus_voltage(0xefab);
        and16_chip.eval().unwrap();
        let output = and16_chip.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0xabcd & 0xefab, "AND16(0xABCD, 0xEFAB) should be correct");
    }
    
    #[test]
    fn test_builtin_or16_chip() {
        let builder = ChipBuilder::new();
        let mut or16_chip = builder.build_builtin_chip("Or16").unwrap();
        
        // Test 16-bit OR operations
        or16_chip.get_pin("a").unwrap().borrow_mut().set_bus_voltage(0x0000);
        or16_chip.get_pin("b").unwrap().borrow_mut().set_bus_voltage(0x0000);
        or16_chip.eval().unwrap();
        let output = or16_chip.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x0000, "OR16(0x0000, 0x0000) should be 0x0000");
        
        or16_chip.get_pin("a").unwrap().borrow_mut().set_bus_voltage(0xf0f0);
        or16_chip.get_pin("b").unwrap().borrow_mut().set_bus_voltage(0x0f0f);
        or16_chip.eval().unwrap();
        let output = or16_chip.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0xffff, "OR16(0xF0F0, 0x0F0F) should be 0xFFFF");
        
        or16_chip.get_pin("a").unwrap().borrow_mut().set_bus_voltage(0x1234);
        or16_chip.get_pin("b").unwrap().borrow_mut().set_bus_voltage(0x5678);
        or16_chip.eval().unwrap();
        let output = or16_chip.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x1234 | 0x5678, "OR16(0x1234, 0x5678) should be correct");
    }
    
    #[test]
    fn test_bus_voltage_operations() {
        // Test from TypeScript: "sets and returns wide busses"
        let mut bus = Bus::new("wide".to_string(), 16);
        bus.set_bus_voltage(0xf00f);
        
        // Test individual bit access
        assert_eq!(bus.voltage(Some(0)).unwrap(), HIGH); // Bit 0 of 0xf00f
        assert_eq!(bus.voltage(Some(8)).unwrap(), LOW);  // Bit 8 of 0xf00f  
        assert_eq!(bus.voltage(Some(9)).unwrap(), LOW);  // Bit 9 of 0xf00f
        assert_eq!(bus.voltage(Some(15)).unwrap(), HIGH); // Bit 15 of 0xf00f
        
        // Test bus voltage retrieval
        assert_eq!(bus.bus_voltage(), 0xf00f);
    }
    
    #[test]
    fn test_builtin_mux_chip() {
        let builder = ChipBuilder::new();
        let mut mux_chip = builder.build_builtin_chip("Mux").unwrap();
        
        // Test MUX truth table: out = sel ? b : a
        let test_cases = [
            (LOW, LOW, LOW, LOW),     // sel=0, a=0, b=0 -> out=0 (a)
            (LOW, HIGH, LOW, LOW),    // sel=0, a=0, b=1 -> out=0 (a) 
            (HIGH, LOW, LOW, HIGH),   // sel=0, a=1, b=0 -> out=1 (a)
            (HIGH, HIGH, LOW, HIGH),  // sel=0, a=1, b=1 -> out=1 (a)
            (LOW, LOW, HIGH, LOW),    // sel=1, a=0, b=0 -> out=0 (b)
            (LOW, HIGH, HIGH, HIGH),  // sel=1, a=0, b=1 -> out=1 (b)
            (HIGH, LOW, HIGH, LOW),   // sel=1, a=1, b=0 -> out=0 (b)
            (HIGH, HIGH, HIGH, HIGH), // sel=1, a=1, b=1 -> out=1 (b)
        ];
        
        for (a_val, b_val, sel_val, expected) in test_cases {
            mux_chip.get_pin("a").unwrap().borrow_mut().pull(a_val, None).unwrap();
            mux_chip.get_pin("b").unwrap().borrow_mut().pull(b_val, None).unwrap();
            mux_chip.get_pin("sel").unwrap().borrow_mut().pull(sel_val, None).unwrap();
            mux_chip.eval().unwrap();
            
            let output = mux_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
            assert_eq!(output, expected, "MUX(a={}, b={}, sel={}) should be {}", a_val, b_val, sel_val, expected);
        }
    }
    
    #[test]
    fn test_builtin_dmux_chip() {
        let builder = ChipBuilder::new();
        let mut dmux_chip = builder.build_builtin_chip("DMux").unwrap();
        
        // Test DMUX truth table: routes input to selected output
        let test_cases = [
            (LOW, LOW, LOW, LOW),     // in=0, sel=0 -> a=0, b=0
            (HIGH, LOW, HIGH, LOW),   // in=1, sel=0 -> a=1, b=0 (route to a)
            (LOW, HIGH, LOW, LOW),    // in=0, sel=1 -> a=0, b=0
            (HIGH, HIGH, LOW, HIGH),  // in=1, sel=1 -> a=0, b=1 (route to b)
        ];
        
        for (in_val, sel_val, expected_a, expected_b) in test_cases {
            dmux_chip.get_pin("in").unwrap().borrow_mut().pull(in_val, None).unwrap();
            dmux_chip.get_pin("sel").unwrap().borrow_mut().pull(sel_val, None).unwrap();
            dmux_chip.eval().unwrap();
            
            let output_a = dmux_chip.get_pin("a").unwrap().borrow().voltage(None).unwrap();
            let output_b = dmux_chip.get_pin("b").unwrap().borrow().voltage(None).unwrap();
            assert_eq!(output_a, expected_a, "DMUX(in={}, sel={}) output a should be {}", in_val, sel_val, expected_a);
            assert_eq!(output_b, expected_b, "DMUX(in={}, sel={}) output b should be {}", in_val, sel_val, expected_b);
        }
    }
    
    #[test]
    fn test_builtin_mux16_chip() {
        let builder = ChipBuilder::new();
        let mut mux16_chip = builder.build_builtin_chip("Mux16").unwrap();
        
        // Test 16-bit MUX operations
        mux16_chip.get_pin("a").unwrap().borrow_mut().set_bus_voltage(0x1234);
        mux16_chip.get_pin("b").unwrap().borrow_mut().set_bus_voltage(0x5678);
        mux16_chip.get_pin("sel").unwrap().borrow_mut().pull(LOW, None).unwrap();
        mux16_chip.eval().unwrap();
        let output = mux16_chip.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x1234, "MUX16 with sel=0 should select input a");
        
        mux16_chip.get_pin("sel").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        mux16_chip.eval().unwrap();
        let output = mux16_chip.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x5678, "MUX16 with sel=1 should select input b");
    }
    
    #[test]
    fn test_builtin_mux4way16_chip() {
        let builder = ChipBuilder::new();
        let mut mux4way_chip = builder.build_builtin_chip("Mux4Way16").unwrap();
        
        // Set up test values
        mux4way_chip.get_pin("a").unwrap().borrow_mut().set_bus_voltage(0x1111);
        mux4way_chip.get_pin("b").unwrap().borrow_mut().set_bus_voltage(0x2222);
        mux4way_chip.get_pin("c").unwrap().borrow_mut().set_bus_voltage(0x3333);
        mux4way_chip.get_pin("d").unwrap().borrow_mut().set_bus_voltage(0x4444);
        
        // Test each selector value
        let test_cases = [
            (0b00, 0x1111), // sel=00 -> a
            (0b01, 0x2222), // sel=01 -> b
            (0b10, 0x3333), // sel=10 -> c
            (0b11, 0x4444), // sel=11 -> d
        ];
        
        for (sel_val, expected) in test_cases {
            mux4way_chip.get_pin("sel").unwrap().borrow_mut().set_bus_voltage(sel_val);
            mux4way_chip.eval().unwrap();
            let output = mux4way_chip.get_pin("out").unwrap().borrow().bus_voltage();
            assert_eq!(output, expected, "MUX4WAY16 with sel={:02b} should output {:04x}", sel_val, expected);
        }
    }
    
    #[test]
    fn test_builtin_add16_chip() {
        let builder = ChipBuilder::new();
        let mut add16_chip = builder.build_builtin_chip("Add16").unwrap();
        
        // Test basic addition
        add16_chip.get_pin("a").unwrap().borrow_mut().set_bus_voltage(0x1234);
        add16_chip.get_pin("b").unwrap().borrow_mut().set_bus_voltage(0x5678);
        add16_chip.eval().unwrap();
        let output = add16_chip.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, (0x1234 + 0x5678) & 0xffff, "ADD16(0x1234, 0x5678) should be correct");
        
        // Test overflow wrapping
        add16_chip.get_pin("a").unwrap().borrow_mut().set_bus_voltage(0xffff);
        add16_chip.get_pin("b").unwrap().borrow_mut().set_bus_voltage(0x0001);
        add16_chip.eval().unwrap();
        let output = add16_chip.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x0000, "ADD16(0xFFFF, 0x0001) should wrap to 0x0000");
        
        // Test zero addition
        add16_chip.get_pin("a").unwrap().borrow_mut().set_bus_voltage(0x0000);
        add16_chip.get_pin("b").unwrap().borrow_mut().set_bus_voltage(0x0000);
        add16_chip.eval().unwrap();
        let output = add16_chip.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x0000, "ADD16(0x0000, 0x0000) should be 0x0000");
    }
    
    #[test]
    fn test_builtin_inc16_chip() {
        let builder = ChipBuilder::new();
        let mut inc16_chip = builder.build_builtin_chip("Inc16").unwrap();
        
        // Test basic increment
        inc16_chip.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0x1234);
        inc16_chip.eval().unwrap();
        let output = inc16_chip.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x1235, "INC16(0x1234) should be 0x1235");
        
        // Test overflow wrapping
        inc16_chip.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0xffff);
        inc16_chip.eval().unwrap();
        let output = inc16_chip.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x0000, "INC16(0xFFFF) should wrap to 0x0000");
        
        // Test zero increment
        inc16_chip.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0x0000);
        inc16_chip.eval().unwrap();
        let output = inc16_chip.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x0001, "INC16(0x0000) should be 0x0001");
    }
    
    #[test]
    fn test_builtin_half_adder_chip() {
        let builder = ChipBuilder::new();
        let mut half_adder_chip = builder.build_builtin_chip("HalfAdder").unwrap();
        
        // Test half adder truth table
        let test_cases = [
            (LOW, LOW, LOW, LOW),     // 0 + 0 = sum:0, carry:0
            (LOW, HIGH, HIGH, LOW),   // 0 + 1 = sum:1, carry:0
            (HIGH, LOW, HIGH, LOW),   // 1 + 0 = sum:1, carry:0
            (HIGH, HIGH, LOW, HIGH),  // 1 + 1 = sum:0, carry:1
        ];
        
        for (a_val, b_val, expected_sum, expected_carry) in test_cases {
            half_adder_chip.get_pin("a").unwrap().borrow_mut().pull(a_val, None).unwrap();
            half_adder_chip.get_pin("b").unwrap().borrow_mut().pull(b_val, None).unwrap();
            half_adder_chip.eval().unwrap();
            
            let sum = half_adder_chip.get_pin("sum").unwrap().borrow().voltage(None).unwrap();
            let carry = half_adder_chip.get_pin("carry").unwrap().borrow().voltage(None).unwrap();
            assert_eq!(sum, expected_sum, "HalfAdder({}, {}) sum should be {}", a_val, b_val, expected_sum);
            assert_eq!(carry, expected_carry, "HalfAdder({}, {}) carry should be {}", a_val, b_val, expected_carry);
        }
    }
    
    #[test]
    fn test_builtin_full_adder_chip() {
        let builder = ChipBuilder::new();
        let mut full_adder_chip = builder.build_builtin_chip("FullAdder").unwrap();
        
        // Test full adder truth table (adding 3 single bits)
        let test_cases = [
            (LOW, LOW, LOW, LOW, LOW),     // 0 + 0 + 0 = sum:0, carry:0
            (LOW, LOW, HIGH, HIGH, LOW),   // 0 + 0 + 1 = sum:1, carry:0
            (LOW, HIGH, LOW, HIGH, LOW),   // 0 + 1 + 0 = sum:1, carry:0
            (LOW, HIGH, HIGH, LOW, HIGH),  // 0 + 1 + 1 = sum:0, carry:1
            (HIGH, LOW, LOW, HIGH, LOW),   // 1 + 0 + 0 = sum:1, carry:0
            (HIGH, LOW, HIGH, LOW, HIGH),  // 1 + 0 + 1 = sum:0, carry:1
            (HIGH, HIGH, LOW, LOW, HIGH),  // 1 + 1 + 0 = sum:0, carry:1
            (HIGH, HIGH, HIGH, HIGH, HIGH), // 1 + 1 + 1 = sum:1, carry:1
        ];
        
        for (a_val, b_val, c_val, expected_sum, expected_carry) in test_cases {
            full_adder_chip.get_pin("a").unwrap().borrow_mut().pull(a_val, None).unwrap();
            full_adder_chip.get_pin("b").unwrap().borrow_mut().pull(b_val, None).unwrap();
            full_adder_chip.get_pin("c").unwrap().borrow_mut().pull(c_val, None).unwrap();
            full_adder_chip.eval().unwrap();
            
            let sum = full_adder_chip.get_pin("sum").unwrap().borrow().voltage(None).unwrap();
            let carry = full_adder_chip.get_pin("carry").unwrap().borrow().voltage(None).unwrap();
            assert_eq!(sum, expected_sum, "FullAdder({}, {}, {}) sum should be {}", a_val, b_val, c_val, expected_sum);
            assert_eq!(carry, expected_carry, "FullAdder({}, {}, {}) carry should be {}", a_val, b_val, c_val, expected_carry);
        }
    }
    
    #[test]
    fn test_builtin_alu_chip() {
        let builder = ChipBuilder::new();
        let mut alu_chip = builder.build_builtin_chip("ALU").unwrap();
        
        // Helper function to set ALU control signals
        let set_control_signals = |alu: &mut Box<dyn ChipInterface>, zx: u8, nx: u8, zy: u8, ny: u8, f: u8, no: u8| {
            alu.get_pin("zx").unwrap().borrow_mut().pull(zx, None).unwrap();
            alu.get_pin("nx").unwrap().borrow_mut().pull(nx, None).unwrap();
            alu.get_pin("zy").unwrap().borrow_mut().pull(zy, None).unwrap();
            alu.get_pin("ny").unwrap().borrow_mut().pull(ny, None).unwrap();
            alu.get_pin("f").unwrap().borrow_mut().pull(f, None).unwrap();
            alu.get_pin("no").unwrap().borrow_mut().pull(no, None).unwrap();
        };
        
        // Test ALU operation: compute 0 (zx=1, nx=0, zy=1, ny=0, f=1, no=0)
        alu_chip.get_pin("x").unwrap().borrow_mut().set_bus_voltage(0x1234);
        alu_chip.get_pin("y").unwrap().borrow_mut().set_bus_voltage(0x5678);
        set_control_signals(&mut alu_chip, HIGH, LOW, HIGH, LOW, HIGH, LOW);
        alu_chip.eval().unwrap();
        
        let output = alu_chip.get_pin("out").unwrap().borrow().bus_voltage();
        let zr = alu_chip.get_pin("zr").unwrap().borrow().voltage(None).unwrap();
        let ng = alu_chip.get_pin("ng").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, 0x0000, "ALU should compute 0");
        assert_eq!(zr, HIGH, "Zero flag should be set");
        assert_eq!(ng, LOW, "Negative flag should be clear");
        
        // Test ALU operation: compute 1 (zx=1, nx=1, zy=1, ny=1, f=1, no=1)
        set_control_signals(&mut alu_chip, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH);
        alu_chip.eval().unwrap();
        
        let output = alu_chip.get_pin("out").unwrap().borrow().bus_voltage();
        let zr = alu_chip.get_pin("zr").unwrap().borrow().voltage(None).unwrap();
        let ng = alu_chip.get_pin("ng").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, 0x0001, "ALU should compute 1");
        assert_eq!(zr, LOW, "Zero flag should be clear");
        assert_eq!(ng, LOW, "Negative flag should be clear");
        
        // Test ALU operation: compute -1 (zx=1, nx=1, zy=1, ny=0, f=1, no=0)
        set_control_signals(&mut alu_chip, HIGH, HIGH, HIGH, LOW, HIGH, LOW);
        alu_chip.eval().unwrap();
        
        let output = alu_chip.get_pin("out").unwrap().borrow().bus_voltage();
        let zr = alu_chip.get_pin("zr").unwrap().borrow().voltage(None).unwrap();
        let ng = alu_chip.get_pin("ng").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, 0xffff, "ALU should compute -1 (0xFFFF)");
        assert_eq!(zr, LOW, "Zero flag should be clear");
        assert_eq!(ng, HIGH, "Negative flag should be set");
        
        // Test ALU operation: compute x (zx=0, nx=0, zy=1, ny=1, f=0, no=0)
        alu_chip.get_pin("x").unwrap().borrow_mut().set_bus_voltage(0x1234);
        set_control_signals(&mut alu_chip, LOW, LOW, HIGH, HIGH, LOW, LOW);
        alu_chip.eval().unwrap();
        
        let output = alu_chip.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x1234, "ALU should compute x");
        
        // Test ALU operation: compute y (zx=1, nx=1, zy=0, ny=0, f=0, no=0)
        alu_chip.get_pin("y").unwrap().borrow_mut().set_bus_voltage(0x5678);
        set_control_signals(&mut alu_chip, HIGH, HIGH, LOW, LOW, LOW, LOW);
        alu_chip.eval().unwrap();
        
        let output = alu_chip.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x5678, "ALU should compute y");
        
        // Test ALU operation: compute x+y (zx=0, nx=0, zy=0, ny=0, f=1, no=0)
        alu_chip.get_pin("x").unwrap().borrow_mut().set_bus_voltage(0x1234);
        alu_chip.get_pin("y").unwrap().borrow_mut().set_bus_voltage(0x5678);
        set_control_signals(&mut alu_chip, LOW, LOW, LOW, LOW, HIGH, LOW);
        alu_chip.eval().unwrap();
        
        let output = alu_chip.get_pin("out").unwrap().borrow().bus_voltage();
        let expected = (0x1234u16.wrapping_add(0x5678)) & 0xffff;
        assert_eq!(output, expected, "ALU should compute x+y");
        
        // Test ALU operation: compute x&y (zx=0, nx=0, zy=0, ny=0, f=0, no=0)
        set_control_signals(&mut alu_chip, LOW, LOW, LOW, LOW, LOW, LOW);
        alu_chip.eval().unwrap();
        
        let output = alu_chip.get_pin("out").unwrap().borrow().bus_voltage();
        let expected = 0x1234 & 0x5678;
        assert_eq!(output, expected, "ALU should compute x&y");
    }
    
    #[test]
    fn test_builtin_ram8_chip() {
        let builder = ChipBuilder::new();
        let ram8_chip = builder.build_builtin_chip("RAM8").unwrap();
        
        // Test basic pin existence and naming
        assert_eq!(ram8_chip.name(), "RAM8");
        assert!(ram8_chip.get_pin("in").is_ok());
        assert!(ram8_chip.get_pin("address").is_ok());
        assert!(ram8_chip.get_pin("load").is_ok());
        assert!(ram8_chip.get_pin("out").is_ok());
        
        // Test that it's registered correctly in the builder
        assert!(ram8_chip.is_input_pin("in"));
        assert!(ram8_chip.is_input_pin("address"));
        assert!(ram8_chip.is_input_pin("load"));
        assert!(ram8_chip.is_output_pin("out"));
    }
    
    #[test]
    fn test_builtin_ram64_chip() {
        let builder = ChipBuilder::new();
        let ram64_chip = builder.build_builtin_chip("RAM64").unwrap();
        
        // Test basic pin existence and naming
        assert_eq!(ram64_chip.name(), "RAM64");
        assert!(ram64_chip.get_pin("in").is_ok());
        assert!(ram64_chip.get_pin("address").is_ok());
        assert!(ram64_chip.get_pin("load").is_ok());
        assert!(ram64_chip.get_pin("out").is_ok());
        
        // Test that it's registered correctly in the builder
        assert!(ram64_chip.is_input_pin("in"));
        assert!(ram64_chip.is_input_pin("address"));
        assert!(ram64_chip.is_input_pin("load"));
        assert!(ram64_chip.is_output_pin("out"));
    }
}