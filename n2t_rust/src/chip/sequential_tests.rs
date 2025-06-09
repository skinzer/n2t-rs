// Tests for sequential chips - DFF, Bit, Register, PC
// These tests verify the clocked behavior of memory elements

use super::*;
use crate::chip::pin::{HIGH, LOW};
use crate::chip::builder::ChipBuilder;
use crate::chip::builtins::{DffChip, BitChip, RegisterChip, PcChip, ClockedChip};
use crate::chip::Clock;

#[cfg(test)]
mod dff_tests {
    use super::*;
    
    #[test]
    fn test_dff_basic_operation() {
        let mut dff = DffChip::new();
        let clock = Clock::new();
        dff.subscribe_to_clock(&clock);
        
        // Initially output should be LOW
        dff.eval().unwrap();
        let output = dff.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, LOW);
        
        // Set input to HIGH
        dff.get_pin("in").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        
        // Tick (rising edge) - should sample input
        dff.tick(HIGH).unwrap();
        
        // Output should still be LOW until tock
        let output = dff.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, LOW);
        
        // Tock (falling edge) - should output stored value
        dff.tock(LOW).unwrap();
        let output = dff.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, HIGH);
        
        // Change input to LOW
        dff.get_pin("in").unwrap().borrow_mut().pull(LOW, None).unwrap();
        
        // Output should remain HIGH until next cycle
        let output = dff.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, HIGH);
        
        // Next cycle: tick and tock
        dff.tick(HIGH).unwrap();
        dff.tock(LOW).unwrap();
        
        // Now output should be LOW
        let output = dff.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, LOW);
    }
    
    #[test]
    fn test_dff_reset() {
        let mut dff = DffChip::new();
        
        // Set up a HIGH state
        dff.get_pin("in").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        dff.tick(HIGH).unwrap();
        dff.tock(LOW).unwrap();
        
        let output = dff.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, HIGH);
        
        // Reset should clear output
        dff.reset().unwrap();
        let output = dff.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, LOW);
    }
}

#[cfg(test)]
mod bit_tests {
    use super::*;
    
    #[test]
    fn test_bit_load_operation() {
        let mut bit = BitChip::new();
        
        // Initially output should be LOW
        bit.eval().unwrap();
        let output = bit.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, LOW);
        
        // Set input and load
        bit.get_pin("in").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        bit.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        
        // Tick should load the value
        bit.tick(HIGH).unwrap();
        
        // Tock should output the loaded value
        bit.tock(LOW).unwrap();
        let output = bit.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, HIGH);
        
        // Change input but don't set load
        bit.get_pin("in").unwrap().borrow_mut().pull(LOW, None).unwrap();
        bit.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
        
        // Value should remain unchanged
        bit.tick(HIGH).unwrap();
        bit.tock(LOW).unwrap();
        let output = bit.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, HIGH);
        
        // Now load the new value
        bit.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        bit.tick(HIGH).unwrap();
        bit.tock(LOW).unwrap();
        let output = bit.get_pin("out").unwrap().borrow().voltage(None).unwrap();
        assert_eq!(output, LOW);
    }
}

#[cfg(test)]
mod register_tests {
    use super::*;
    
    #[test]
    fn test_register_16bit_operation() {
        let mut register = RegisterChip::new();
        
        // Initially output should be 0
        register.eval().unwrap();
        let output = register.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0);
        
        // Set input to a 16-bit value
        let test_value = 0xABCD;
        register.get_pin("in").unwrap().borrow_mut().set_bus_voltage(test_value);
        register.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        
        // Tick and tock to load the value
        register.tick(HIGH).unwrap();
        register.tock(LOW).unwrap();
        
        let output = register.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, test_value);
        
        // Change input but don't load
        register.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0x1234);
        register.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
        
        register.tick(HIGH).unwrap();
        register.tock(LOW).unwrap();
        
        // Value should remain unchanged
        let output = register.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, test_value);
    }
    
    #[test]
    fn test_register_reset() {
        let mut register = RegisterChip::new();
        
        // Load a value
        register.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0xFFFF);
        register.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        register.tick(HIGH).unwrap();
        register.tock(LOW).unwrap();
        
        let output = register.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0xFFFF);
        
        // Reset should clear the register
        register.reset().unwrap();
        let output = register.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0);
    }
}

#[cfg(test)]
mod pc_tests {
    use super::*;
    
    #[test]
    fn test_pc_increment() {
        let mut pc = PcChip::new();
        
        // Initially output should be 0
        pc.eval().unwrap();
        let output = pc.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0);
        
        // Enable increment
        pc.get_pin("inc").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        pc.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
        pc.get_pin("reset").unwrap().borrow_mut().pull(LOW, None).unwrap();
        
        // Should increment on each clock cycle
        for i in 1..=5 {
            pc.tick(HIGH).unwrap();
            pc.tock(LOW).unwrap();
            let output = pc.get_pin("out").unwrap().borrow().bus_voltage();
            assert_eq!(output, i);
        }
    }
    
    #[test]
    fn test_pc_load() {
        let mut pc = PcChip::new();
        
        // Increment to some value first
        pc.get_pin("inc").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        pc.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
        pc.get_pin("reset").unwrap().borrow_mut().pull(LOW, None).unwrap();
        
        pc.tick(HIGH).unwrap();
        pc.tock(LOW).unwrap();
        pc.tick(HIGH).unwrap();
        pc.tock(LOW).unwrap();
        
        let output = pc.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 2);
        
        // Now load a specific value
        let load_value = 0x100;
        pc.get_pin("in").unwrap().borrow_mut().set_bus_voltage(load_value);
        pc.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        pc.get_pin("inc").unwrap().borrow_mut().pull(LOW, None).unwrap();
        
        pc.tick(HIGH).unwrap();
        pc.tock(LOW).unwrap();
        
        let output = pc.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, load_value);
    }
    
    #[test]
    fn test_pc_reset() {
        let mut pc = PcChip::new();
        
        // Load a value and increment
        pc.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0x500);
        pc.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        pc.get_pin("inc").unwrap().borrow_mut().pull(LOW, None).unwrap();
        pc.get_pin("reset").unwrap().borrow_mut().pull(LOW, None).unwrap();
        
        pc.tick(HIGH).unwrap();
        pc.tock(LOW).unwrap();
        
        let output = pc.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x500);
        
        // Reset should override everything
        pc.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        pc.get_pin("inc").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        pc.get_pin("reset").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        
        pc.tick(HIGH).unwrap();
        pc.tock(LOW).unwrap();
        
        let output = pc.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0);
    }
    
    #[test]
    fn test_pc_priority_order() {
        let mut pc = PcChip::new();
        
        // Test that reset has highest priority
        pc.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0x123);
        pc.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        pc.get_pin("inc").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        pc.get_pin("reset").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        
        pc.tick(HIGH).unwrap();
        pc.tock(LOW).unwrap();
        
        let output = pc.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0); // Reset wins
        
        // Test that load has priority over increment
        pc.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        pc.get_pin("inc").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        pc.get_pin("reset").unwrap().borrow_mut().pull(LOW, None).unwrap();
        
        pc.tick(HIGH).unwrap();
        pc.tock(LOW).unwrap();
        
        let output = pc.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0x123); // Load wins over increment
    }
}

#[cfg(test)]
mod builder_integration_tests {
    use super::*;
    
    #[test]
    fn test_sequential_chips_from_builder() {
        let builder = ChipBuilder::new();
        
        // Test DFF
        let dff = builder.build_builtin_chip("DFF").unwrap();
        assert_eq!(dff.name(), "DFF");
        assert!(dff.get_pin("in").is_ok());
        assert!(dff.get_pin("out").is_ok());
        
        // Test Bit
        let bit = builder.build_builtin_chip("Bit").unwrap();
        assert_eq!(bit.name(), "Bit");
        assert!(bit.get_pin("in").is_ok());
        assert!(bit.get_pin("load").is_ok());
        assert!(bit.get_pin("out").is_ok());
        
        // Test Register
        let register = builder.build_builtin_chip("Register").unwrap();
        assert_eq!(register.name(), "Register");
        assert!(register.get_pin("in").is_ok());
        assert!(register.get_pin("load").is_ok());
        assert!(register.get_pin("out").is_ok());
        
        // Test PC
        let pc = builder.build_builtin_chip("PC").unwrap();
        assert_eq!(pc.name(), "PC");
        assert!(pc.get_pin("in").is_ok());
        assert!(pc.get_pin("load").is_ok());
        assert!(pc.get_pin("inc").is_ok());
        assert!(pc.get_pin("reset").is_ok());
        assert!(pc.get_pin("out").is_ok());
    }
}