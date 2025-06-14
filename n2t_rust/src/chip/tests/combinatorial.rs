// Tests for combinatorial logic chips
// Translated from TypeScript chip.test.ts describe("combinatorial")

use crate::chip::builder::ChipBuilder;
use crate::chip::pin::{HIGH, LOW};

#[test]
fn test_nand_chip() {
    let builder = ChipBuilder::new();
    let mut nand = builder.build_builtin_chip("Nand").unwrap();
    
    // Initial state
    nand.eval().unwrap();
    let output = nand.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, HIGH);
    
    // Test: a=HIGH, b=LOW -> out=HIGH
    nand.get_pin("a").unwrap().borrow_mut().pull(HIGH, None).unwrap();
    nand.eval().unwrap();
    let output = nand.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, HIGH);
    
    // Test: a=HIGH, b=HIGH -> out=LOW (NAND behavior)
    nand.get_pin("b").unwrap().borrow_mut().pull(HIGH, None).unwrap();
    nand.eval().unwrap();
    let output = nand.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, LOW);
    
    // Test: a=LOW, b=HIGH -> out=HIGH
    nand.get_pin("a").unwrap().borrow_mut().pull(LOW, None).unwrap();
    nand.eval().unwrap();
    let output = nand.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, HIGH);
}

#[test]
fn test_not_chip() {
    let builder = ChipBuilder::new();
    let mut not_chip = builder.build_builtin_chip("Not").unwrap();
    
    // Initial state (input defaults to LOW)
    not_chip.eval().unwrap();
    let output = not_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, HIGH);
    
    // Test: in=HIGH -> out=LOW
    not_chip.get_pin("in").unwrap().borrow_mut().pull(HIGH, None).unwrap();
    not_chip.eval().unwrap();
    let output = not_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, LOW);
}

#[test]
fn test_and_chip() {
    let builder = ChipBuilder::new();
    let mut and_chip = builder.build_builtin_chip("And").unwrap();
    
    let a = and_chip.get_pin("a").unwrap();
    let b = and_chip.get_pin("b").unwrap();
    
    // Test: a=LOW, b=LOW -> out=LOW
    and_chip.eval().unwrap();
    let output = and_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, LOW);
    
    // Test: a=HIGH, b=LOW -> out=LOW
    a.borrow_mut().pull(HIGH, None).unwrap();
    and_chip.eval().unwrap();
    let output = and_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, LOW);
    
    // Test: a=HIGH, b=HIGH -> out=HIGH
    b.borrow_mut().pull(HIGH, None).unwrap();
    and_chip.eval().unwrap();
    let output = and_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, HIGH);
    
    // Test: a=LOW, b=HIGH -> out=LOW
    a.borrow_mut().pull(LOW, None).unwrap();
    and_chip.eval().unwrap();
    let output = and_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, LOW);
}

#[test]
fn test_or_chip() {
    let builder = ChipBuilder::new();
    let mut or_chip = builder.build_builtin_chip("Or").unwrap();
    
    let a = or_chip.get_pin("a").unwrap();
    let b = or_chip.get_pin("b").unwrap();
    
    // Test: a=LOW, b=LOW -> out=LOW
    or_chip.eval().unwrap();
    let output = or_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, LOW);
    
    // Test: a=HIGH, b=LOW -> out=HIGH
    a.borrow_mut().pull(HIGH, None).unwrap();
    or_chip.eval().unwrap();
    let output = or_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, HIGH);
    
    // Test: a=HIGH, b=HIGH -> out=HIGH
    b.borrow_mut().pull(HIGH, None).unwrap();
    or_chip.eval().unwrap();
    let output = or_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, HIGH);
    
    // Test: a=LOW, b=HIGH -> out=HIGH
    a.borrow_mut().pull(LOW, None).unwrap();
    or_chip.eval().unwrap();
    let output = or_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, HIGH);
}

#[test]
fn test_xor_chip() {
    let builder = ChipBuilder::new();
    let mut xor_chip = builder.build_builtin_chip("Xor").unwrap();
    
    let a = xor_chip.get_pin("a").unwrap();
    let b = xor_chip.get_pin("b").unwrap();
    
    // Test: a=LOW, b=LOW -> out=LOW
    xor_chip.eval().unwrap();
    let output = xor_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, LOW);
    
    // Test: a=HIGH, b=LOW -> out=HIGH
    a.borrow_mut().pull(HIGH, None).unwrap();
    xor_chip.eval().unwrap();
    let output = xor_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, HIGH);
    
    // Test: a=HIGH, b=HIGH -> out=LOW
    b.borrow_mut().pull(HIGH, None).unwrap();
    xor_chip.eval().unwrap();
    let output = xor_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, LOW);
    
    // Test: a=LOW, b=HIGH -> out=HIGH
    a.borrow_mut().pull(LOW, None).unwrap();
    xor_chip.eval().unwrap();
    let output = xor_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, HIGH);
}

#[test]
fn test_mux_chip() {
    let builder = ChipBuilder::new();
    let mut mux_chip = builder.build_builtin_chip("Mux").unwrap();
    
    let a = mux_chip.get_pin("a").unwrap();
    let b = mux_chip.get_pin("b").unwrap();
    let sel = mux_chip.get_pin("sel").unwrap();
    
    // Test: sel=LOW -> out=a
    a.borrow_mut().pull(HIGH, None).unwrap();
    b.borrow_mut().pull(LOW, None).unwrap();
    sel.borrow_mut().pull(LOW, None).unwrap();
    mux_chip.eval().unwrap();
    let output = mux_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, HIGH); // Should select a
    
    // Test: sel=HIGH -> out=b
    sel.borrow_mut().pull(HIGH, None).unwrap();
    mux_chip.eval().unwrap();
    let output = mux_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, LOW); // Should select b
}

#[test]
fn test_dmux_chip() {
    let builder = ChipBuilder::new();
    let mut dmux_chip = builder.build_builtin_chip("DMux").unwrap();
    
    let input = dmux_chip.get_pin("in").unwrap();
    let sel = dmux_chip.get_pin("sel").unwrap();
    
    // Test: sel=LOW -> a=in, b=LOW
    input.borrow_mut().pull(HIGH, None).unwrap();
    sel.borrow_mut().pull(LOW, None).unwrap();
    dmux_chip.eval().unwrap();
    
    let a_output = dmux_chip.get_pin("a").unwrap().borrow().voltage(None).unwrap();
    let b_output = dmux_chip.get_pin("b").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(a_output, HIGH);
    assert_eq!(b_output, LOW);
    
    // Test: sel=HIGH -> a=LOW, b=in
    sel.borrow_mut().pull(HIGH, None).unwrap();
    dmux_chip.eval().unwrap();
    
    let a_output = dmux_chip.get_pin("a").unwrap().borrow().voltage(None).unwrap();
    let b_output = dmux_chip.get_pin("b").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(a_output, LOW);
    assert_eq!(b_output, HIGH);
}