// Tests for wide (16-bit) operations
// Translated from TypeScript chip.test.ts describe("wide")

use crate::chip::builder::ChipBuilder;
use crate::chip::Bus;

#[test]
fn test_not16_chip() {
    let builder = ChipBuilder::new();
    let mut not16 = builder.build_builtin_chip("Not16").unwrap();
    
    let input = not16.get_pin("in").unwrap();
    
    // Test: in=0x0000 -> out=0xFFFF
    input.borrow_mut().set_bus_voltage(0x0000);
    not16.eval().unwrap();
    let output = not16.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 0xFFFF);
    
    // Test: in=0xF00F -> out=0x0FF0
    input.borrow_mut().set_bus_voltage(0xF00F);
    not16.eval().unwrap();
    let output = not16.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 0x0FF0);
    
    // Test: in=0xFFFF -> out=0x0000
    input.borrow_mut().set_bus_voltage(0xFFFF);
    not16.eval().unwrap();
    let output = not16.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 0x0000);
}

#[test]
fn test_and16_chip() {
    let builder = ChipBuilder::new();
    let mut and16 = builder.build_builtin_chip("And16").unwrap();
    
    let a = and16.get_pin("a").unwrap();
    let b = and16.get_pin("b").unwrap();
    
    // Test: a=0xFFFF, b=0x0000 -> out=0x0000
    a.borrow_mut().set_bus_voltage(0xFFFF);
    b.borrow_mut().set_bus_voltage(0x0000);
    and16.eval().unwrap();
    let output = and16.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 0x0000);
    
    // Test: a=0xFFFF, b=0xFFFF -> out=0xFFFF
    a.borrow_mut().set_bus_voltage(0xFFFF);
    b.borrow_mut().set_bus_voltage(0xFFFF);
    and16.eval().unwrap();
    let output = and16.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 0xFFFF);
    
    // Test: a=0xF0F0, b=0x0F0F -> out=0x0000
    a.borrow_mut().set_bus_voltage(0xF0F0);
    b.borrow_mut().set_bus_voltage(0x0F0F);
    and16.eval().unwrap();
    let output = and16.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 0x0000);
    
    // Test: a=0xF0F0, b=0xF0F0 -> out=0xF0F0
    a.borrow_mut().set_bus_voltage(0xF0F0);
    b.borrow_mut().set_bus_voltage(0xF0F0);
    and16.eval().unwrap();
    let output = and16.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 0xF0F0);
}

#[test]
fn test_or16_chip() {
    let builder = ChipBuilder::new();
    let mut or16 = builder.build_builtin_chip("Or16").unwrap();
    
    let a = or16.get_pin("a").unwrap();
    let b = or16.get_pin("b").unwrap();
    
    // Test: a=0x0000, b=0x0000 -> out=0x0000
    a.borrow_mut().set_bus_voltage(0x0000);
    b.borrow_mut().set_bus_voltage(0x0000);
    or16.eval().unwrap();
    let output = or16.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 0x0000);
    
    // Test: a=0xFFFF, b=0x0000 -> out=0xFFFF
    a.borrow_mut().set_bus_voltage(0xFFFF);
    b.borrow_mut().set_bus_voltage(0x0000);
    or16.eval().unwrap();
    let output = or16.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 0xFFFF);
    
    // Test: a=0xF0F0, b=0x0F0F -> out=0xFFFF
    a.borrow_mut().set_bus_voltage(0xF0F0);
    b.borrow_mut().set_bus_voltage(0x0F0F);
    or16.eval().unwrap();
    let output = or16.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 0xFFFF);
}

#[test]
fn test_mux16_chip() {
    let builder = ChipBuilder::new();
    let mut mux16 = builder.build_builtin_chip("Mux16").unwrap();
    
    let a = mux16.get_pin("a").unwrap();
    let b = mux16.get_pin("b").unwrap();
    let sel = mux16.get_pin("sel").unwrap();
    
    // Test: sel=LOW -> out=a
    a.borrow_mut().set_bus_voltage(0x1234);
    b.borrow_mut().set_bus_voltage(0x5678);
    sel.borrow_mut().pull(crate::chip::pin::LOW, None).unwrap();
    mux16.eval().unwrap();
    let output = mux16.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 0x1234);
    
    // Test: sel=HIGH -> out=b
    sel.borrow_mut().pull(crate::chip::pin::HIGH, None).unwrap();
    mux16.eval().unwrap();
    let output = mux16.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 0x5678);
}

#[test]
fn test_bus_voltage_operations() {
    // Test bus voltage manipulation functions
    use crate::chip::pin::Pin;
    let mut pin = Bus::new("wide".to_string(), 16);
    pin.set_bus_voltage(0xF00F);
    
    // Test individual bit access
    assert_eq!(pin.voltage(Some(0)).unwrap(), crate::chip::pin::HIGH);  // bit 0 = 1
    assert_eq!(pin.voltage(Some(1)).unwrap(), crate::chip::pin::HIGH);  // bit 1 = 1
    assert_eq!(pin.voltage(Some(2)).unwrap(), crate::chip::pin::HIGH);  // bit 2 = 1
    assert_eq!(pin.voltage(Some(3)).unwrap(), crate::chip::pin::HIGH);  // bit 3 = 1
    assert_eq!(pin.voltage(Some(4)).unwrap(), crate::chip::pin::LOW);   // bit 4 = 0
    assert_eq!(pin.voltage(Some(5)).unwrap(), crate::chip::pin::LOW);   // bit 5 = 0
    assert_eq!(pin.voltage(Some(6)).unwrap(), crate::chip::pin::LOW);   // bit 6 = 0
    assert_eq!(pin.voltage(Some(7)).unwrap(), crate::chip::pin::LOW);   // bit 7 = 0
    assert_eq!(pin.voltage(Some(8)).unwrap(), crate::chip::pin::LOW);   // bit 8 = 0
    assert_eq!(pin.voltage(Some(9)).unwrap(), crate::chip::pin::LOW);   // bit 9 = 0
    assert_eq!(pin.voltage(Some(10)).unwrap(), crate::chip::pin::LOW);  // bit 10 = 0
    assert_eq!(pin.voltage(Some(11)).unwrap(), crate::chip::pin::LOW);  // bit 11 = 0
    assert_eq!(pin.voltage(Some(12)).unwrap(), crate::chip::pin::HIGH); // bit 12 = 1
    assert_eq!(pin.voltage(Some(13)).unwrap(), crate::chip::pin::HIGH); // bit 13 = 1
    assert_eq!(pin.voltage(Some(14)).unwrap(), crate::chip::pin::HIGH); // bit 14 = 1
    assert_eq!(pin.voltage(Some(15)).unwrap(), crate::chip::pin::HIGH); // bit 15 = 1
    
    // Test bus voltage
    assert_eq!(pin.bus_voltage(), 0xF00F);
}

#[test]
fn test_add16_chip() {
    let builder = ChipBuilder::new();
    let mut add16 = builder.build_builtin_chip("Add16").unwrap();
    
    let a = add16.get_pin("a").unwrap();
    let b = add16.get_pin("b").unwrap();
    
    // Test: 0 + 0 = 0
    a.borrow_mut().set_bus_voltage(0);
    b.borrow_mut().set_bus_voltage(0);
    add16.eval().unwrap();
    let output = add16.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 0);
    
    // Test: 1 + 1 = 2
    a.borrow_mut().set_bus_voltage(1);
    b.borrow_mut().set_bus_voltage(1);
    add16.eval().unwrap();
    let output = add16.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 2);
    
    // Test: 100 + 200 = 300
    a.borrow_mut().set_bus_voltage(100);
    b.borrow_mut().set_bus_voltage(200);
    add16.eval().unwrap();
    let output = add16.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 300);
    
    // Test: overflow (should wrap around)
    a.borrow_mut().set_bus_voltage(0xFFFF);
    b.borrow_mut().set_bus_voltage(1);
    add16.eval().unwrap();
    let output = add16.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 0); // Wraps around to 0
}

#[test]
fn test_inc16_chip() {
    let builder = ChipBuilder::new();
    let mut inc16 = builder.build_builtin_chip("Inc16").unwrap();
    
    let input = inc16.get_pin("in").unwrap();
    
    // Test: 0 + 1 = 1
    input.borrow_mut().set_bus_voltage(0);
    inc16.eval().unwrap();
    let output = inc16.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 1);
    
    // Test: 99 + 1 = 100
    input.borrow_mut().set_bus_voltage(99);
    inc16.eval().unwrap();
    let output = inc16.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 100);
    
    // Test: overflow
    input.borrow_mut().set_bus_voltage(0xFFFF);
    inc16.eval().unwrap();
    let output = inc16.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 0); // Wraps around to 0
}