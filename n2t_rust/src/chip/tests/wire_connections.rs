// Tests for wire connections and SubBus functionality
// Covers complex wiring patterns, bus connections, and error handling

use crate::chip::*;
use crate::chip::pin::{HIGH, LOW};
use crate::chip::builder::ChipBuilder;
use crate::chip::subbus::{PinRange, create_input_subbus, create_output_subbus};
use std::rc::Rc;
use std::cell::RefCell;

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
    // Test direct pin-to-pin connection without SubBus
    let mut host_chip = Chip::new("TestChip".to_string());
    
    // Add 1-bit input and output
    host_chip.add_input_pin("in".to_string(), Rc::new(RefCell::new(Bus::new("in".to_string(), 1))));
    host_chip.add_output_pin("out".to_string(), Rc::new(RefCell::new(Bus::new("out".to_string(), 1))));
    
    // Create one Not chip
    let builder = ChipBuilder::new();
    let not_part = builder.build_builtin_chip("Not").unwrap();
    
    // Wire: in -> Not.in, Not.out -> out (direct pin-to-pin)
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
    assert_eq!(output, 0); // Not(HIGH) = LOW
    
    // Test reverse: set input to LOW, expect output to be HIGH  
    host_chip.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0);
    host_chip.eval().unwrap();
    let output = host_chip.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 1); // Not(LOW) = HIGH
}

#[test]
fn test_subbus_creation() {
    // Test creating SubBus connections
    let source_bus = Rc::new(RefCell::new(Bus::new("source".to_string(), 16)));
    
    // Create input subbus for bits 0-7
    let range = PinRange::new_range("source".to_string(), 0, 7).unwrap();
    let input_subbus = create_input_subbus(source_bus.clone(), &range).unwrap();
    
    // Set source bus value
    source_bus.borrow_mut().set_bus_voltage(0xFF00); // Upper 8 bits set
    
    // Input subbus should reflect only lower 8 bits (0x00)
    assert_eq!(input_subbus.borrow().bus_voltage(), 0x00);
    
    // Set source to have lower bits set
    source_bus.borrow_mut().set_bus_voltage(0x00FF); // Lower 8 bits set
    
    // Input subbus should now reflect lower 8 bits (0xFF)
    assert_eq!(input_subbus.borrow().bus_voltage(), 0xFF);
}

#[test]
fn test_output_subbus() {
    // Test creating output SubBus connections
    let target_bus = Rc::new(RefCell::new(Bus::new("target".to_string(), 16)));
    
    // Create output subbus for bits 8-15
    let range = PinRange::new_range("target".to_string(), 8, 15).unwrap();
    let output_subbus = create_output_subbus(target_bus.clone(), &range).unwrap();
    
    // Set subbus value
    output_subbus.borrow_mut().set_bus_voltage(0xAB);
    
    // Target bus should have upper 8 bits set to 0xAB, lower 8 bits unchanged
    let target_value = target_bus.borrow().bus_voltage();
    assert_eq!(target_value & 0xFF00, 0xAB00); // Upper 8 bits should be 0xAB
    assert_eq!(target_value & 0x00FF, 0x0000); // Lower 8 bits should be 0
}

#[test]
fn test_single_bit_subbus_connection() {
    // Test connecting individual bits via SubBus
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
    
    // We expect: input bit 0 (HIGH) -> Not -> output bit 0 (LOW)
    assert_eq!(output & 0b01, 0b00); // Check bit 0 is LOW
    
    // Test reverse: set input bit 0 to LOW, expect output bit 0 to be HIGH  
    host_chip.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0b00); // bit 0 = LOW
    host_chip.eval().unwrap();
    let output = host_chip.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output & 0b01, 0b01); // Check bit 0 is HIGH
}

#[test]
fn test_pin_range_wire_connection() {
    // Test wiring with pin ranges - create a 3-bit inverter
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
    assert_eq!(output, 0b010);
    
    // Test input 0b010 should become 0b101  
    host_chip.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0b010);
    host_chip.eval().unwrap();
    let output = host_chip.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 0b101);
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

#[test]
fn test_bus_voltage_masking() {
    // Test that SubBus properly masks values to fit target width
    let source_bus = Rc::new(RefCell::new(Bus::new("source".to_string(), 16)));
    
    // Set a large value
    source_bus.borrow_mut().set_bus_voltage(0xFFFF);
    
    // Create subbus for lower 4 bits
    let range = PinRange::new_range("source".to_string(), 0, 3).unwrap();
    let subbus = create_input_subbus(source_bus.clone(), &range).unwrap();
    
    // Should only see lower 4 bits
    assert_eq!(subbus.borrow().bus_voltage(), 0x0F); // Only 4 bits
    
    // Create subbus for upper 4 bits  
    let range = PinRange::new_range("source".to_string(), 12, 15).unwrap();
    let subbus = create_input_subbus(source_bus.clone(), &range).unwrap();
    
    // Should see the upper 4 bits shifted down
    assert_eq!(subbus.borrow().bus_voltage(), 0x0F); // Upper 4 bits, shifted to position 0-3
}

#[test]
fn test_multi_part_composite_chip() {
    // Test a more complex composite chip with multiple parts
    let mut host_chip = Chip::new("AndOrChip".to_string());
    
    // Add inputs and outputs
    host_chip.add_input_pin("a".to_string(), Rc::new(RefCell::new(Bus::new("a".to_string(), 1))));
    host_chip.add_input_pin("b".to_string(), Rc::new(RefCell::new(Bus::new("b".to_string(), 1))));
    host_chip.add_input_pin("c".to_string(), Rc::new(RefCell::new(Bus::new("c".to_string(), 1))));
    host_chip.add_output_pin("out".to_string(), Rc::new(RefCell::new(Bus::new("out".to_string(), 1))));
    
    // Add internal pin for connecting AND output to OR input
    host_chip.add_internal_pin("and_out".to_string(), Rc::new(RefCell::new(Bus::new("and_out".to_string(), 1))));
    
    // Create parts: (a AND b) OR c
    let builder = ChipBuilder::new();
    let and_part = builder.build_builtin_chip("And").unwrap();
    let or_part = builder.build_builtin_chip("Or").unwrap();
    
    // Wire AND gate: a -> And.a, b -> And.b, And.out -> and_out
    let and_connections = vec![
        Connection::new(PinSide::new("a".to_string()), PinSide::new("a".to_string())),
        Connection::new(PinSide::new("b".to_string()), PinSide::new("b".to_string())),
        Connection::new(PinSide::new("and_out".to_string()), PinSide::new("out".to_string())), // AND output to internal pin
    ];
    
    // Wire OR gate: and_out -> Or.a, c -> Or.b, Or.out -> out
    let or_connections = vec![
        Connection::new(PinSide::new("and_out".to_string()), PinSide::new("a".to_string())), // Internal pin to OR input
        Connection::new(PinSide::new("c".to_string()), PinSide::new("b".to_string())),        // c to OR input
        Connection::new(PinSide::new("out".to_string()), PinSide::new("out".to_string())),    // OR output to chip output
    ];
    
    host_chip.wire(and_part, and_connections).unwrap();
    host_chip.wire(or_part, or_connections).unwrap();
    
    // Test the logic: (a AND b) OR c
    
    // Test: a=0, b=0, c=0 -> out=0
    host_chip.get_pin("a").unwrap().borrow_mut().pull(LOW, None).unwrap();
    host_chip.get_pin("b").unwrap().borrow_mut().pull(LOW, None).unwrap();
    host_chip.get_pin("c").unwrap().borrow_mut().pull(LOW, None).unwrap();
    host_chip.eval().unwrap();
    let output = host_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, LOW);
    
    // Test: a=1, b=1, c=0 -> out=1 (AND part outputs 1)
    host_chip.get_pin("a").unwrap().borrow_mut().pull(HIGH, None).unwrap();
    host_chip.get_pin("b").unwrap().borrow_mut().pull(HIGH, None).unwrap();
    host_chip.get_pin("c").unwrap().borrow_mut().pull(LOW, None).unwrap();
    host_chip.eval().unwrap();
    let output = host_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, HIGH);
    
    // Test: a=0, b=0, c=1 -> out=1 (OR part outputs 1 due to c)
    host_chip.get_pin("a").unwrap().borrow_mut().pull(LOW, None).unwrap();
    host_chip.get_pin("b").unwrap().borrow_mut().pull(LOW, None).unwrap();
    host_chip.get_pin("c").unwrap().borrow_mut().pull(HIGH, None).unwrap();
    host_chip.eval().unwrap();
    let output = host_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, HIGH);
}