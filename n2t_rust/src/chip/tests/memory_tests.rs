// Tests for memory chips (RAM hierarchy)
// Translated from TypeScript memory tests and sequential logic

use crate::chip::*;
use crate::chip::pin::{HIGH, LOW};
use crate::chip::builder::ChipBuilder;
use crate::chip::builtins::{ClockedChip, Ram8Chip, Ram64Chip, Ram512Chip, Ram4kChip, Ram16kChip};

#[test]
fn test_ram8_basic_operations() {
    let builder = ChipBuilder::new();
    let mut ram8 = builder.build_builtin_chip("RAM8").unwrap();
    
    // Test writing to different addresses
    for addr in 0..8 {
        let test_value = (addr + 1) * 10;
        
        // Write value to address
        ram8.get_pin("in").unwrap().borrow_mut().set_bus_voltage(test_value);
        ram8.get_pin("address").unwrap().borrow_mut().set_bus_voltage(addr);
        ram8.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        
        // Evaluate the RAM
        ram8.eval().unwrap();
        
        // Read back the value
        ram8.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
        ram8.eval().unwrap();
        let output = ram8.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, test_value, "RAM8 address {} should contain {}", addr, test_value);
    }
}

#[test]
fn test_ram8_address_isolation() {
    let builder = ChipBuilder::new();
    let mut ram8 = builder.build_builtin_chip("RAM8").unwrap();
    
    // Write different values to different addresses
    let test_data = [100, 200, 300, 400, 500, 600, 700, 800];
    
    for (addr, &value) in test_data.iter().enumerate() {
        ram8.get_pin("in").unwrap().borrow_mut().set_bus_voltage(value);
        ram8.get_pin("address").unwrap().borrow_mut().set_bus_voltage(addr as u16);
        ram8.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        
        if let Ok(clocked_ram) = ram8.as_any_mut().downcast_mut::<Ram8Chip>() {
            clocked_ram.tick(HIGH).unwrap();
            clocked_ram.tock(LOW).unwrap();
        }
    }
    
    // Verify each address contains the correct value
    for (addr, &expected_value) in test_data.iter().enumerate() {
        ram8.get_pin("address").unwrap().borrow_mut().set_bus_voltage(addr as u16);
        ram8.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
        ram8.eval().unwrap();
        
        let output = ram8.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, expected_value, "Address {} should contain {}", addr, expected_value);
    }
}

#[test]
fn test_ram64_capacity() {
    let builder = ChipBuilder::new();
    let mut ram64 = builder.build_builtin_chip("RAM64").unwrap();
    
    // Test that we can address all 64 locations
    let test_addresses = [0, 1, 7, 8, 15, 31, 32, 63];
    
    for &addr in &test_addresses {
        let test_value = addr * 2 + 1000;
        
        // Write value
        ram64.get_pin("in").unwrap().borrow_mut().set_bus_voltage(test_value);
        ram64.get_pin("address").unwrap().borrow_mut().set_bus_voltage(addr);
        ram64.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        
        ram64.eval().unwrap();
        
        // Read back
        ram64.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
        ram64.eval().unwrap();
        let output = ram64.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, test_value, "RAM64 address {} should contain {}", addr, test_value);
    }
}

#[test]
fn test_ram512_large_capacity() {
    let builder = ChipBuilder::new();
    let mut ram512 = builder.build_builtin_chip("RAM512").unwrap();
    
    // Test sparse addressing across the 512-word space
    let test_addresses = [0, 1, 8, 64, 128, 256, 511];
    
    for &addr in &test_addresses {
        let test_value = addr + 2000;
        
        // Write value
        ram512.get_pin("in").unwrap().borrow_mut().set_bus_voltage(test_value);
        ram512.get_pin("address").unwrap().borrow_mut().set_bus_voltage(addr);
        ram512.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        
        ram512.eval().unwrap();
        
        // Read back
        ram512.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
        ram512.eval().unwrap();
        let output = ram512.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, test_value, "RAM512 address {} should contain {}", addr, test_value);
    }
}

#[test]
fn test_ram4k_addressing() {
    let builder = ChipBuilder::new();
    let mut ram4k = builder.build_builtin_chip("RAM4K").unwrap();
    
    // Test key addresses in the 4K space
    let test_addresses = [0, 1, 512, 1024, 2048, 4095];
    
    for &addr in &test_addresses {
        let test_value = (addr % 65536) + 3000; // Keep within 16-bit range
        
        // Write value
        ram4k.get_pin("in").unwrap().borrow_mut().set_bus_voltage(test_value);
        ram4k.get_pin("address").unwrap().borrow_mut().set_bus_voltage(addr);
        ram4k.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        
        ram4k.eval().unwrap();
        
        // Read back
        ram4k.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
        ram4k.eval().unwrap();
        let output = ram4k.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, test_value, "RAM4K address {} should contain {}", addr, test_value);
    }
}

#[test]
fn test_ram16k_max_capacity() {
    let builder = ChipBuilder::new();
    let mut ram16k = builder.build_builtin_chip("RAM16K").unwrap();
    
    // Test addresses across the full 16K range
    let test_addresses = [0, 1, 1024, 8192, 16383];
    
    for &addr in &test_addresses {
        let test_value = (addr % 65536) + 4000;
        
        // Write value
        ram16k.get_pin("in").unwrap().borrow_mut().set_bus_voltage(test_value);
        ram16k.get_pin("address").unwrap().borrow_mut().set_bus_voltage(addr);
        ram16k.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        
        ram16k.eval().unwrap();
        
        // Read back
        ram16k.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
        ram16k.eval().unwrap();
        let output = ram16k.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, test_value, "RAM16K address {} should contain {}", addr, test_value);
    }
}

#[test]
fn test_memory_load_control() {
    let builder = ChipBuilder::new();
    let mut ram8 = builder.build_builtin_chip("RAM8").unwrap();
    
    // Write initial value
    ram8.get_pin("in").unwrap().borrow_mut().set_bus_voltage(1000);
    ram8.get_pin("address").unwrap().borrow_mut().set_bus_voltage(0);
    ram8.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
    
    if let Ok(clocked_ram) = ram8.as_any_mut().downcast_mut::<Ram8Chip>() {
        clocked_ram.tick(HIGH).unwrap();
        clocked_ram.tock(LOW).unwrap();
    }
    
    // Change input but disable load - value should not change
    ram8.get_pin("in").unwrap().borrow_mut().set_bus_voltage(2000);
    ram8.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
    
    if let Ok(clocked_ram) = ram8.as_any_mut().downcast_mut::<Ram8Chip>() {
        clocked_ram.tick(HIGH).unwrap();
        clocked_ram.tock(LOW).unwrap();
    }
    
    // Read back - should still be original value
    ram8.eval().unwrap();
    let output = ram8.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 1000, "Value should not change when load is disabled");
}

#[test]
fn test_memory_reset() {
    let builder = ChipBuilder::new();
    let mut ram8 = builder.build_builtin_chip("RAM8").unwrap();
    
    // Write values to multiple addresses
    for addr in 0..8 {
        ram8.get_pin("in").unwrap().borrow_mut().set_bus_voltage(addr * 100 + 500);
        ram8.get_pin("address").unwrap().borrow_mut().set_bus_voltage(addr);
        ram8.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        
        if let Ok(clocked_ram) = ram8.as_any_mut().downcast_mut::<Ram8Chip>() {
            clocked_ram.tick(HIGH).unwrap();
            clocked_ram.tock(LOW).unwrap();
        }
    }
    
    // Reset should clear all memory
    ram8.reset().unwrap();
    
    // Check that all addresses now read 0
    for addr in 0..8 {
        ram8.get_pin("address").unwrap().borrow_mut().set_bus_voltage(addr);
        ram8.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
        ram8.eval().unwrap();
        
        let output = ram8.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, 0, "Address {} should be 0 after reset", addr);
    }
}

#[test]
fn test_memory_concurrent_access() {
    let builder = ChipBuilder::new();
    let mut ram64 = builder.build_builtin_chip("RAM64").unwrap();
    
    // Simulate concurrent read/write operations
    // Write to address 10
    ram64.get_pin("in").unwrap().borrow_mut().set_bus_voltage(1337);
    ram64.get_pin("address").unwrap().borrow_mut().set_bus_voltage(10);
    ram64.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
    
    if let Ok(clocked_ram) = ram64.as_any_mut().downcast_mut::<Ram64Chip>() {
        clocked_ram.tick(HIGH).unwrap();
        clocked_ram.tock(LOW).unwrap();
    }
    
    // Change to read from address 20 (different address)
    ram64.get_pin("address").unwrap().borrow_mut().set_bus_voltage(20);
    ram64.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
    ram64.eval().unwrap();
    
    let output = ram64.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 0, "Unwritten address should return 0");
    
    // Read from the address we wrote to
    ram64.get_pin("address").unwrap().borrow_mut().set_bus_voltage(10);
    ram64.eval().unwrap();
    
    let output = ram64.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 1337, "Written address should return written value");
}

#[test]
fn test_memory_address_decoding() {
    let builder = ChipBuilder::new();
    let mut ram64 = builder.build_builtin_chip("RAM64").unwrap();
    
    // Test that address decoding works correctly
    // Write to addresses that differ only in specific bits
    let addresses = [
        0b000000, // 0
        0b000001, // 1  
        0b000010, // 2
        0b000100, // 4
        0b001000, // 8
        0b010000, // 16
        0b100000, // 32
        0b111111, // 63
    ];
    
    for (i, &addr) in addresses.iter().enumerate() {
        let value = (i + 1) * 111;
        
        // Write unique value to each address
        ram64.get_pin("in").unwrap().borrow_mut().set_bus_voltage(value as u16);
        ram64.get_pin("address").unwrap().borrow_mut().set_bus_voltage(addr);
        ram64.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
        
        ram64.eval().unwrap();
    }
    
    // Verify each address contains its unique value
    for (i, &addr) in addresses.iter().enumerate() {
        let expected_value = (i + 1) * 111;
        
        ram64.get_pin("address").unwrap().borrow_mut().set_bus_voltage(addr);
        ram64.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
        ram64.eval().unwrap();
        
        let output = ram64.get_pin("out").unwrap().borrow().bus_voltage();
        assert_eq!(output, expected_value as u16, 
                  "Address 0b{:06b} should contain {}", addr, expected_value);
    }
}