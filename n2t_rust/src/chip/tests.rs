// Tests translated from TypeScript chip.test.ts
// These tests match the original TypeScript test suite structure

use super::*;
use crate::chip::pin::{HIGH, LOW};
use crate::chip::builder::ChipBuilder;

/// Test suite for combinatorial logic chips 
/// Translated from chip.test.ts describe("combinatorial")
mod combinatorial {
    use super::*;
    
    #[test]
    fn test_nand_chip() {
        // Translated from chip.test.ts describe("nand")
        let builder = ChipBuilder::new();
        let mut nand = builder.build_builtin_chip("Nand").unwrap();
        
        // Initial state
        nand.eval().unwrap();
        let output = nand.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, HIGH);
        
        // nand.in("a")?.pull(HIGH);
        nand.get_pin("a").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        nand.eval().unwrap();
        let output = nand.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, HIGH);
        
        // nand.in("b")?.pull(HIGH);
        nand.get_pin("b").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        nand.eval().unwrap();
        let output = nand.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, LOW);
        
        // nand.in("a")?.pull(LOW);
        nand.get_pin("a").unwrap().borrow_mut().pull(LOW, None).unwrap();
        nand.eval().unwrap();
        let output = nand.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, HIGH);
    }
    
    #[test]
    fn test_not_chip() {
        // Translated from chip.test.ts describe("not")
        let builder = ChipBuilder::new();
        let mut not_chip = builder.build_builtin_chip("Not").unwrap();
        
        // Initial state
        not_chip.eval().unwrap();
        let output = not_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, HIGH);
        
        // notChip.in().pull(HIGH);
        not_chip.get_pin("in").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        not_chip.eval().unwrap();
        let output = not_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, LOW);
    }
    
    #[test]
    fn test_and_chip() {
        // Translated from chip.test.ts describe("and")
        let builder = ChipBuilder::new();
        let mut and_chip = builder.build_builtin_chip("And").unwrap();
        
        let a = and_chip.get_pin("a").unwrap();
        let b = and_chip.get_pin("b").unwrap();
        
        and_chip.eval().unwrap();
        let output = and_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, LOW);
        
        a.borrow_mut().pull(HIGH, None).unwrap();
        and_chip.eval().unwrap();
        let output = and_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, LOW);
        
        b.borrow_mut().pull(HIGH, None).unwrap();
        and_chip.eval().unwrap();
        let output = and_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, HIGH);
        
        a.borrow_mut().pull(LOW, None).unwrap();
        and_chip.eval().unwrap();
        let output = and_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, LOW);
    }
    
    #[test]
    fn test_or_chip() {
        // Translated from chip.test.ts describe("or")
        let builder = ChipBuilder::new();
        let mut or_chip = builder.build_builtin_chip("Or").unwrap();
        
        let a = or_chip.get_pin("a").unwrap();
        let b = or_chip.get_pin("b").unwrap();
        
        or_chip.eval().unwrap();
        let output = or_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, LOW);
        
        a.borrow_mut().pull(HIGH, None).unwrap();
        or_chip.eval().unwrap();
        let output = or_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, HIGH);
        
        b.borrow_mut().pull(HIGH, None).unwrap();
        or_chip.eval().unwrap();
        let output = or_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, HIGH);
        
        a.borrow_mut().pull(LOW, None).unwrap();
        or_chip.eval().unwrap();
        let output = or_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, HIGH);
    }
    
    #[test]
    fn test_xor_chip() {
        // Translated from chip.test.ts describe("xor")
        let builder = ChipBuilder::new();
        let mut xor_chip = builder.build_builtin_chip("Xor").unwrap();
        
        let a = xor_chip.get_pin("a").unwrap();
        let b = xor_chip.get_pin("b").unwrap();
        
        xor_chip.eval().unwrap();
        let output = xor_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, LOW);
        
        a.borrow_mut().pull(HIGH, None).unwrap();
        xor_chip.eval().unwrap();
        let output = xor_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, HIGH);
        
        b.borrow_mut().pull(HIGH, None).unwrap();
        xor_chip.eval().unwrap();
        let output = xor_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, LOW);
        
        a.borrow_mut().pull(LOW, None).unwrap();
        xor_chip.eval().unwrap();
        let output = xor_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, HIGH);
    }
}

/// Test suite for wide (16-bit) logic chips
/// Translated from chip.test.ts describe("wide")
mod wide {
    use super::*;
    
    #[test]
    fn test_not16_chip() {
        // Translated from chip.test.ts describe("Not16")
        let builder = ChipBuilder::new();
        let mut not16 = builder.build_builtin_chip("Not16").unwrap();
        
        let inn = not16.get_pin("in").unwrap();
        
        // inn.busVoltage = 0x0;
        inn.borrow_mut().set_bus_voltage(0x0);
        not16.eval().unwrap();
        let output = not16.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0xffff);
        
        // inn.busVoltage = 0xf00f;
        inn.borrow_mut().set_bus_voltage(0xf00f);
        not16.eval().unwrap();
        let output = not16.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x0ff0);
    }
    
    #[test]
    fn test_bus_voltage_operations() {
        // Translated from chip.test.ts describe("bus voltage") "sets and returns wide busses"
        let mut pin = Bus::new("wide".to_string(), 16);
        pin.set_bus_voltage(0xf00f);
        
        assert_eq!(pin.voltage(Some(0)).unwrap(), HIGH);
        assert_eq!(pin.voltage(Some(8)).unwrap(), LOW);
        assert_eq!(pin.voltage(Some(9)).unwrap(), LOW);
        assert_eq!(pin.voltage(Some(15)).unwrap(), HIGH);
        assert_eq!(pin.bus_voltage(), 0xf00f);
    }
}

/// Test suite for HDL parsing and chip creation
/// Some tests translated from chip.test.ts bus_voltage "creates wide busses internally"
mod hdl_integration {
    use super::*;
    use crate::prelude::HdlParser;
    
    #[test]
    fn test_hdl_chip_creation_with_wide_buses() {
        // This test validates that our HDL parser correctly handles 16-bit pins
        // Based on the TypeScript test that wires Not16 with 16-bit buses
        let mut parser = HdlParser::new().unwrap();
        let _builder = ChipBuilder::new();
        
        let hdl = r#"
            CHIP TestWide {
                IN a[16];
                OUT out[16];
                BUILTIN;
            }
        "#;
        
        let hdl_chip = parser.parse(hdl).unwrap();
        
        // Validate parsing
        assert_eq!(hdl_chip.name, "TestWide");
        assert_eq!(hdl_chip.inputs.len(), 1);
        assert_eq!(hdl_chip.outputs.len(), 1);
        assert_eq!(hdl_chip.inputs[0].name, "a");
        assert_eq!(hdl_chip.inputs[0].width, Some(16));
        assert_eq!(hdl_chip.outputs[0].name, "out");
        assert_eq!(hdl_chip.outputs[0].width, Some(16));
    }
}

/// Tests related to pin parsing functionality
/// Translated from chip.test.ts "parses toPin"
mod pin_parsing {
    use crate::prelude::HdlParser;
    
    // Note: The TypeScript version has a parseToPin function that we haven't implemented yet
    // This would parse strings like "a", "a[2]", "a[2..4]" into pin references
    // For now, we test our HDL parser's ability to handle these patterns
    
    #[test]
    fn test_pin_range_parsing_in_hdl() {
        let mut parser = HdlParser::new().unwrap();
        
        // Test single pin
        let hdl_single = r#"
            CHIP TestSingle {
                IN a;
                OUT out;
                BUILTIN;
            }
        "#;
        
        let chip = parser.parse(hdl_single).unwrap();
        assert_eq!(chip.inputs[0].name, "a");
        assert_eq!(chip.inputs[0].width, None); // Single bit
        
        // Test pin with width
        let hdl_width = r#"
            CHIP TestWidth {
                IN a[8];
                OUT out[8];
                BUILTIN;
            }
        "#;
        
        let chip = parser.parse(hdl_width).unwrap();
        assert_eq!(chip.inputs[0].name, "a");
        assert_eq!(chip.inputs[0].width, Some(8));
    }
}

/// Test suite for wire connections functionality
/// Translated from chip.test.ts SubBus tests
mod wire_connections {
    use super::*;
    use std::rc::Rc;
    use std::cell::RefCell;
    use crate::chip::subbus::PinRange;
    
    #[test]
    fn test_simple_wire_connection() {
        // Create a composite chip that wires a Not gate
        let mut host_chip = Chip::new("TestChip".to_string());
        
        // Add input and output pins  
        host_chip.add_input_pin("in".to_string(), Rc::new(RefCell::new(Bus::new("in".to_string(), 1))));
        host_chip.add_output_pin("out".to_string(), Rc::new(RefCell::new(Bus::new("out".to_string(), 1))));
        
        // Create a Not chip part
        let builder = ChipBuilder::new();
        let not_part = builder.build_builtin_chip("Not").unwrap();
        
        // Wire the Not chip: in -> Not.in, Not.out -> out
        let connections = vec![
            Connection::new(
                PinSide::new("in".to_string()),   // from host chip's input
                PinSide::new("in".to_string()),   // to Not part's input
            ),
            Connection::new(
                PinSide::new("out".to_string()),  // from Not part's output
                PinSide::new("out".to_string()),  // to host chip's output
            ),
        ];
        
        // This should work without error
        host_chip.wire(not_part, connections).unwrap();
        
        // Test the wired chip
        host_chip.get_pin("in").unwrap().borrow_mut().pull(LOW, None).unwrap();
        host_chip.eval().unwrap();
        let output = host_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, HIGH);
        
        host_chip.get_pin("in").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        host_chip.eval().unwrap();
        let output = host_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, LOW);
    }
    
    #[test]
    fn test_pin_range_wire_connection() {
        // Test wiring with pin ranges - similar to the SubBus tests in TypeScript
        let mut host_chip = Chip::new("TestChip".to_string());
        
        // Add a 3-bit input and 3-bit output
        host_chip.add_input_pin("in".to_string(), Rc::new(RefCell::new(Bus::new("in".to_string(), 3))));
        host_chip.add_output_pin("out".to_string(), Rc::new(RefCell::new(Bus::new("out".to_string(), 3))));
        
        // Create Not chip parts 
        let builder = ChipBuilder::new();
        let not_part1 = builder.build_builtin_chip("Not").unwrap();
        let not_part2 = builder.build_builtin_chip("Not").unwrap();
        let not_part3 = builder.build_builtin_chip("Not").unwrap();
        
        // Wire bit 0: in[0] -> Not1.in, Not1.out -> out[0]
        let connections1 = vec![
            Connection::new(
                PinSide::with_range("in".to_string(), PinRange::new_single_bit("in".to_string(), 0)),
                PinSide::new("in".to_string()),
            ),
            Connection::new(
                PinSide::with_range("out".to_string(), PinRange::new_single_bit("out".to_string(), 0)),
                PinSide::new("out".to_string()),
            ),
        ];
        
        // Wire bit 1: in[1] -> Not2.in, Not2.out -> out[1]
        let connections2 = vec![
            Connection::new(
                PinSide::with_range("in".to_string(), PinRange::new_single_bit("in".to_string(), 1)),
                PinSide::new("in".to_string()),
            ),
            Connection::new(
                PinSide::with_range("out".to_string(), PinRange::new_single_bit("out".to_string(), 1)),
                PinSide::new("out".to_string()),
            ),
        ];
        
        // Wire bit 2: in[2] -> Not3.in, Not3.out -> out[2]
        let connections3 = vec![
            Connection::new(
                PinSide::with_range("in".to_string(), PinRange::new_single_bit("in".to_string(), 2)),
                PinSide::new("in".to_string()),
            ),
            Connection::new(
                PinSide::with_range("out".to_string(), PinRange::new_single_bit("out".to_string(), 2)),
                PinSide::new("out".to_string()),
            ),
        ];
        
        // Wire all the parts
        host_chip.wire(not_part1, connections1).unwrap();
        host_chip.wire(not_part2, connections2).unwrap();
        host_chip.wire(not_part3, connections3).unwrap();
        
        // Test the functionality: input 0b101 should become 0b010
        host_chip.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0b101);
        host_chip.eval().unwrap();
        let output = host_chip.get_pin("out").unwrap().borrow().bus_voltage();
        println!("Input: 0b{:03b}, Output: 0b{:03b}, Expected: 0b{:03b}", 0b101, output, 0b010);
        assert_eq!(output, 0b010);
        
        // Test input 0b010 should become 0b101  
        host_chip.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0b010);
        host_chip.eval().unwrap();
        let output = host_chip.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0b101);
    }
    
    #[test]
    fn test_constant_wire_connection() {
        // Test connecting constants (true/false) to pins
        let mut host_chip = Chip::new("TestChip".to_string());
        
        // Add output pin
        host_chip.add_output_pin("out".to_string(), Rc::new(RefCell::new(Bus::new("out".to_string(), 1))));
        
        // Create a Not chip and wire true to its input
        let builder = ChipBuilder::new();
        let not_part = builder.build_builtin_chip("Not").unwrap();
        
        let connections = vec![
            Connection::new(
                PinSide::new("true".to_string()),  // Constant true
                PinSide::new("in".to_string()),
            ),
            Connection::new(
                PinSide::new("out".to_string()),
                PinSide::new("out".to_string()),
            ),
        ];
        
        host_chip.wire(not_part, connections).unwrap();
        
        // Evaluate: Not(true) should be false
        host_chip.eval().unwrap();
        let output = host_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, LOW);
    }
    
    #[test]
    fn test_direct_pin_connection() {
        // Test direct pin-to-pin connection without SubBus to isolate the issue
        let mut host_chip = Chip::new("TestChip".to_string());
        
        // Add 1-bit input and output (no SubBus needed)
        host_chip.add_input_pin("in".to_string(), Rc::new(RefCell::new(Bus::new("in".to_string(), 1))));
        host_chip.add_output_pin("out".to_string(), Rc::new(RefCell::new(Bus::new("out".to_string(), 1))));
        
        // Create one Not chip
        let builder = ChipBuilder::new();
        let not_part = builder.build_builtin_chip("Not").unwrap();
        
        // Wire: in -> Not.in, Not.out -> out (no ranges, direct pin-to-pin)
        let connections = vec![
            Connection::new(
                PinSide::new("in".to_string()),
                PinSide::new("in".to_string()),
            ),
            Connection::new(
                PinSide::new("out".to_string()),
                PinSide::new("out".to_string()),
            ),
        ];
        
        host_chip.wire(not_part, connections).unwrap();
        
        // Test: set input to HIGH, expect output to be LOW
        host_chip.get_pin("in").unwrap().borrow_mut().set_bus_voltage(1);
        host_chip.eval().unwrap();
        let output = host_chip.get_pin("out").unwrap().borrow().bus_voltage();
        
        println!("Direct connection test - Input: {}, Output: {}", 1, output);
        assert_eq!(output, 0); // Not(HIGH) = LOW
        
        // Test reverse: set input to LOW, expect output to be HIGH  
        host_chip.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0);
        host_chip.eval().unwrap();
        let output = host_chip.get_pin("out").unwrap().borrow().bus_voltage();
        
        println!("Direct connection test reverse - Input: {}, Output: {}", 0, output);
        assert_eq!(output, 1); // Not(LOW) = HIGH
    }
    
    #[test]
    fn test_single_bit_subbus_connection() {
        // Simplified test - just one Not gate with SubBus
        let mut host_chip = Chip::new("TestChip".to_string());
        
        // Add 2-bit input and output
        host_chip.add_input_pin("in".to_string(), Rc::new(RefCell::new(Bus::new("in".to_string(), 2))));
        host_chip.add_output_pin("out".to_string(), Rc::new(RefCell::new(Bus::new("out".to_string(), 2))));
        
        // Create one Not chip
        let builder = ChipBuilder::new();
        let not_part = builder.build_builtin_chip("Not").unwrap();
        
        // Wire: in[0] -> Not.in, Not.out -> out[0]
        let connections = vec![
            Connection::new(
                PinSide::with_range("in".to_string(), PinRange::new_single_bit("in".to_string(), 0)),
                PinSide::new("in".to_string()),
            ),
            Connection::new(
                PinSide::with_range("out".to_string(), PinRange::new_single_bit("out".to_string(), 0)),
                PinSide::new("out".to_string()),
            ),
        ];
        
        host_chip.wire(not_part, connections).unwrap();
        
        // Test: set input bit 0 to HIGH, expect output bit 0 to be LOW
        host_chip.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0b01); // bit 0 = HIGH
        host_chip.eval().unwrap();
        let output = host_chip.get_pin("out").unwrap().borrow().bus_voltage();
        
        println!("Single bit test - Input: 0b{:02b}, Output: 0b{:02b}", 0b01, output);
        
        // We expect: input bit 0 (HIGH) -> Not -> output bit 0 (LOW)
        // So output should be 0b00 (bit 0 = LOW, bit 1 = unchanged/LOW)
        assert_eq!(output & 0b01, 0b00); // Check bit 0 is LOW
        
        // Test reverse: set input bit 0 to LOW, expect output bit 0 to be HIGH  
        host_chip.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0b00); // bit 0 = LOW
        host_chip.eval().unwrap();
        let output = host_chip.get_pin("out").unwrap().borrow().bus_voltage();
        
        println!("Single bit test reverse - Input: 0b{:02b}, Output: 0b{:02b}", 0b00, output);
        assert_eq!(output & 0b01, 0b01); // Check bit 0 is HIGH
    }
    
    #[test]  
    fn test_width_mismatch_error() {
        // Test that width mismatches are detected
        let mut host_chip = Chip::new("TestChip".to_string());
        
        // Add 8-bit input
        host_chip.add_input_pin("in".to_string(), Rc::new(RefCell::new(Bus::new("in".to_string(), 8))));
        
        // Create a Not chip (1-bit)
        let builder = ChipBuilder::new();
        let not_part = builder.build_builtin_chip("Not").unwrap();
        
        // Try to connect 8-bit input to 1-bit Not input - should fail
        let connections = vec![
            Connection::new(
                PinSide::new("in".to_string()), // 8 bits
                PinSide::new("in".to_string()), // 1 bit
            ),
        ];
        
        let result = host_chip.wire(not_part, connections);
        assert!(result.is_err());
        
        if let Err(WireError::WidthMismatch { from_width, to_width, .. }) = result {
            assert_eq!(from_width, 8);
            assert_eq!(to_width, 1);
        } else {
            panic!("Expected WidthMismatch error");
        }
    }
}