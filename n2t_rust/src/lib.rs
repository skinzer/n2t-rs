pub mod chip;
pub mod cpu;
pub mod error;
pub mod languages;
pub mod test;
pub mod vm;

pub use error::{Result, SimulatorError};

pub mod prelude {
    pub use crate::chip::{Bus, Chip, Pin, Voltage, ChipBuilder};
    pub use crate::error::{Result, SimulatorError};
    pub use crate::languages::hdl::HdlParser;
}

#[cfg(test)]
mod integration_tests {
    use super::prelude::*;
    use crate::chip::pin::{HIGH, LOW};
    
    #[test]
    fn test_complete_hdl_to_simulation_pipeline() {
        // Test the complete pipeline: HDL -> AST -> Chip -> Simulation
        
        let mut parser = HdlParser::new().unwrap();
        let builder = ChipBuilder::new();
        
        // Test 1: Parse and simulate a builtin NOT gate
        let not_hdl = r#"
            CHIP Not {
                IN in;
                OUT out;
                BUILTIN;
            }
        "#;
        
        let hdl_chip = parser.parse(not_hdl).unwrap();
        let mut not_chip = builder.build_chip(&hdl_chip).unwrap();
        
        // Simulate NOT gate
        not_chip.get_pin("in").unwrap().borrow_mut().pull(LOW, None).unwrap();
        not_chip.eval().unwrap();
        let output = not_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, HIGH, "NOT(0) should be 1");
        
        not_chip.get_pin("in").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        not_chip.eval().unwrap();
        let output = not_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, LOW, "NOT(1) should be 0");
        
        // Test 2: Parse and simulate a 16-bit chip definition
        let wide_bus_hdl = r#"
            CHIP WideBus {
                IN a[16], b[16];
                OUT out[16];
                BUILTIN;
            }
        "#;
        
        let hdl_chip = parser.parse(wide_bus_hdl).unwrap();
        
        // Even though WideBus isn't a real builtin, we can test the parser
        assert_eq!(hdl_chip.name, "WideBus");
        assert_eq!(hdl_chip.inputs.len(), 2);
        assert_eq!(hdl_chip.outputs.len(), 1);
        assert_eq!(hdl_chip.inputs[0].name, "a");
        assert_eq!(hdl_chip.inputs[0].width, Some(16));
        assert_eq!(hdl_chip.inputs[1].name, "b");
        assert_eq!(hdl_chip.inputs[1].width, Some(16));
        assert_eq!(hdl_chip.outputs[0].name, "out");
        assert_eq!(hdl_chip.outputs[0].width, Some(16));
        
        // Test 3: Test AND gate with proper evaluation
        let and_hdl = r#"
            CHIP And {
                IN a, b;
                OUT out;
                BUILTIN;
            }
        "#;
        
        let hdl_chip = parser.parse(and_hdl).unwrap();
        let mut and_chip = builder.build_chip(&hdl_chip).unwrap();
        
        // Test AND truth table through HDL pipeline
        and_chip.get_pin("a").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        and_chip.get_pin("b").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        and_chip.eval().unwrap();
        let output = and_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, HIGH, "AND(1, 1) should be 1");
    }
    
    #[test]
    fn test_pin_bus_system_with_16bit_values() {
        // Test the pin/bus system with multi-bit values
        let mut bus = Bus::new("test_bus".to_string(), 16);
        
        // Test setting bus voltage
        bus.set_bus_voltage(0xABCD); // 43981 in decimal
        
        // Test individual bit access
        assert_eq!(bus.voltage(Some(0)).unwrap(), HIGH); // Bit 0 of 0xABCD
        assert_eq!(bus.voltage(Some(1)).unwrap(), LOW);  // Bit 1 of 0xABCD
        assert_eq!(bus.voltage(Some(15)).unwrap(), HIGH); // Bit 15 of 0xABCD
        
        // Test bus voltage retrieval
        assert_eq!(bus.bus_voltage(), 0xABCD);
        
        // Test individual bit manipulation
        bus.pull(LOW, Some(0)).unwrap(); // Clear bit 0
        assert_eq!(bus.bus_voltage(), 0xABCC); // Should be 0xABCD with bit 0 cleared
        
        bus.toggle(Some(1)).unwrap(); // Toggle bit 1 (was 0, should become 1)
        assert_eq!(bus.bus_voltage(), 0xABCE); // Should have bit 1 set
    }
    
    #[test]
    fn test_chip_interconnection() {
        // Test connecting multiple chips together
        let builder = ChipBuilder::new();
        
        // Create a NOT chip
        let mut not_chip = builder.build_builtin_chip("Not").unwrap();
        
        // Create an AND chip
        let mut and_chip = builder.build_builtin_chip("And").unwrap();
        
        // Test independent operation first
        not_chip.get_pin("in").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        not_chip.eval().unwrap();
        let not_output = not_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(not_output, LOW);
        
        // Test AND chip
        and_chip.get_pin("a").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        and_chip.get_pin("b").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        and_chip.eval().unwrap();
        let and_output = and_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(and_output, HIGH);
        
        // Reset for different test
        and_chip.reset().unwrap();
        let and_output = and_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(and_output, LOW); // Should be reset to LOW
    }
}