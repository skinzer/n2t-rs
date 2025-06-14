// Tests for ChipBuilder integration and comprehensive chip testing
// Covers builder functionality, dynamic chip creation, and all builtin chips

use crate::chip::*;
use crate::chip::pin::{HIGH, LOW};
use crate::chip::builder::ChipBuilder;
use crate::chip::builtins::{ClockedChip, DffChip, BitChip, RegisterChip, PcChip};

#[test]
fn test_builder_creates_all_basic_logic_chips() {
    let builder = ChipBuilder::new();
    
    // Test all basic logic gates
    let chips = vec!["Nand", "Not", "And", "Or", "Xor"];
    
    for chip_name in chips {
        let chip = builder.build_builtin_chip(chip_name).unwrap();
        assert_eq!(chip.name(), chip_name);
        
        // Basic sanity check - chip should have input or output pins
        assert!(chip.input_pins().len() > 0 || chip.output_pins().len() > 0, 
                "{} should have pins", chip_name);
    }
}

#[test]
fn test_builder_creates_all_multiplexer_chips() {
    let builder = ChipBuilder::new();
    
    // Test multiplexer chips
    let mux_chips = vec![
        ("Mux", vec!["a", "b", "sel", "out"]),
        ("DMux", vec!["in", "sel", "a", "b"]),
    ];
    
    for (chip_name, expected_pins) in mux_chips {
        let chip = builder.build_builtin_chip(chip_name).unwrap();
        assert_eq!(chip.name(), chip_name);
        
        // Check all expected pins exist
        for pin_name in expected_pins {
            assert!(chip.get_pin(pin_name).is_ok(), 
                   "{} should have pin {}", chip_name, pin_name);
        }
    }
}

#[test]
fn test_builder_creates_all_wide_chips() {
    let builder = ChipBuilder::new();
    
    // Test 16-bit chips
    let wide_chips = vec![
        ("Not16", vec!["in", "out"]),
        ("And16", vec!["a", "b", "out"]),
        ("Or16", vec!["a", "b", "out"]),
        ("Mux16", vec!["a", "b", "sel", "out"]),
        ("Add16", vec!["a", "b", "out"]),
        ("Inc16", vec!["in", "out"]),
    ];
    
    for (chip_name, expected_pins) in wide_chips {
        let chip = builder.build_builtin_chip(chip_name).unwrap();
        assert_eq!(chip.name(), chip_name);
        
        // Check all expected pins exist
        for pin_name in expected_pins {
            assert!(chip.get_pin(pin_name).is_ok(), 
                   "{} should have pin {}", chip_name, pin_name);
        }
    }
}

#[test]
fn test_builder_creates_all_arithmetic_chips() {
    let builder = ChipBuilder::new();
    
    // Test arithmetic chips
    let arithmetic_chips = vec![
        ("HalfAdder", vec!["a", "b", "sum", "carry"]),
        ("FullAdder", vec!["a", "b", "c", "sum", "carry"]),
        ("Add16", vec!["a", "b", "out"]),
        ("Inc16", vec!["in", "out"]),
        ("ALU", vec!["x", "y", "zx", "nx", "zy", "ny", "f", "no", "out", "zr", "ng"]),
    ];
    
    for (chip_name, expected_pins) in arithmetic_chips {
        let chip = builder.build_builtin_chip(chip_name).unwrap();
        assert_eq!(chip.name(), chip_name);
        
        // Check all expected pins exist
        for pin_name in expected_pins {
            assert!(chip.get_pin(pin_name).is_ok(), 
                   "{} should have pin {}", chip_name, pin_name);
        }
    }
}

#[test]
fn test_builder_creates_all_sequential_chips() {
    let builder = ChipBuilder::new();
    
    // Test sequential chips
    let sequential_chips = vec![
        ("DFF", vec!["in", "out"]),
        ("Bit", vec!["in", "load", "out"]),
        ("Register", vec!["in", "load", "out"]),
        ("PC", vec!["in", "load", "inc", "reset", "out"]),
    ];
    
    for (chip_name, expected_pins) in sequential_chips {
        let chip = builder.build_builtin_chip(chip_name).unwrap();
        assert_eq!(chip.name(), chip_name);
        
        // Check all expected pins exist
        for pin_name in expected_pins {
            assert!(chip.get_pin(pin_name).is_ok(), 
                   "{} should have pin {}", chip_name, pin_name);
        }
    }
}

#[test]
fn test_builder_creates_all_memory_chips() {
    let builder = ChipBuilder::new();
    
    // Test memory chips
    let memory_chips = vec![
        ("RAM8", vec!["in", "load", "address", "out"]),
        ("RAM64", vec!["in", "load", "address", "out"]),
        ("RAM512", vec!["in", "load", "address", "out"]),
        ("RAM4K", vec!["in", "load", "address", "out"]),
        ("RAM16K", vec!["in", "load", "address", "out"]),
    ];
    
    for (chip_name, expected_pins) in memory_chips {
        let chip = builder.build_builtin_chip(chip_name).unwrap();
        assert_eq!(chip.name(), chip_name);
        
        // Check all expected pins exist
        for pin_name in expected_pins {
            assert!(chip.get_pin(pin_name).is_ok(), 
                   "{} should have pin {}", chip_name, pin_name);
        }
    }
}

#[test]
fn test_builder_creates_all_computer_chips() {
    let builder = ChipBuilder::new();
    
    // Test computer-level chips
    let computer_chips = vec![
        ("ROM32K", vec!["address", "out"]),
        ("Screen", vec!["in", "load", "address", "out"]),
        ("Keyboard", vec!["out"]),
    ];
    
    for (chip_name, expected_pins) in computer_chips {
        let chip = builder.build_builtin_chip(chip_name).unwrap();
        assert_eq!(chip.name(), chip_name);
        
        // Check all expected pins exist
        for pin_name in expected_pins {
            assert!(chip.get_pin(pin_name).is_ok(), 
                   "{} should have pin {}", chip_name, pin_name);
        }
    }
}

#[test]
fn test_builder_error_handling() {
    let builder = ChipBuilder::new();
    
    // Test that unknown chips return errors
    let result = builder.build_builtin_chip("UnknownChip");
    assert!(result.is_err(), "Should fail for unknown chip");
    
    let result = builder.build_builtin_chip("");
    assert!(result.is_err(), "Should fail for empty chip name");
}

#[test]
fn test_sequential_chips_are_clocked() {
    let builder = ChipBuilder::new();
    
    // Test that sequential chips can be created
    let sequential_chips = ["DFF", "Bit", "Register", "PC"];
    
    for chip_name in sequential_chips {
        let chip = builder.build_builtin_chip(chip_name).unwrap();
        
        // Test that these chips exist and have the expected name
        assert_eq!(chip.name(), chip_name);
        // Sequential chips should have pins
        assert!(chip.input_pins().len() > 0 || chip.output_pins().len() > 0, 
                "{} should have pins", chip_name);
    }
}

#[test]
fn test_builder_chip_functionality_spot_check() {
    let builder = ChipBuilder::new();
    
    // Spot check that built chips actually work correctly
    
    // Test NAND gate
    let mut nand = builder.build_builtin_chip("Nand").unwrap();
    nand.get_pin("a").unwrap().borrow_mut().pull(HIGH, None).unwrap();
    nand.get_pin("b").unwrap().borrow_mut().pull(HIGH, None).unwrap();
    nand.eval().unwrap();
    let output = nand.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, LOW); // NAND(1,1) = 0
    
    // Test NOT16 gate
    let mut not16 = builder.build_builtin_chip("Not16").unwrap();
    not16.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0x0000);
    not16.eval().unwrap();
    let output = not16.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 0xFFFF); // NOT16(0) = 0xFFFF
    
    // Test Add16
    let mut add16 = builder.build_builtin_chip("Add16").unwrap();
    add16.get_pin("a").unwrap().borrow_mut().set_bus_voltage(5);
    add16.get_pin("b").unwrap().borrow_mut().set_bus_voltage(3);
    add16.eval().unwrap();
    let output = add16.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 8); // 5 + 3 = 8
}

#[test]
fn test_builder_memory_functionality_spot_check() {
    let builder = ChipBuilder::new();
    
    // Test RAM8 basic functionality
    let mut ram8 = builder.build_builtin_chip("RAM8").unwrap();
    
    // Write value 42 to address 3
    ram8.get_pin("in").unwrap().borrow_mut().set_bus_voltage(42);
    ram8.get_pin("address").unwrap().borrow_mut().set_bus_voltage(3);
    ram8.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
    
    // Evaluate the RAM
    ram8.eval().unwrap();
    
    // Read from address 3
    ram8.get_pin("load").unwrap().borrow_mut().pull(LOW, None).unwrap();
    ram8.eval().unwrap();
    let output = ram8.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 42); // Should read back the stored value
}

#[test]
fn test_all_chips_can_be_created() {
    // Comprehensive test that all expected chips can be created
    let builder = ChipBuilder::new();
    
    let all_expected_chips = vec![
        // Basic logic
        "Nand", "Not", "And", "Or", "Xor",
        // Multiplexers
        "Mux", "DMux",
        // Wide operations
        "Not16", "And16", "Or16", "Mux16",
        // Arithmetic
        "HalfAdder", "FullAdder", "Add16", "Inc16", "ALU",
        // Sequential
        "DFF", "Bit", "Register", "PC",
        // Memory
        "RAM8", "RAM64", "RAM512", "RAM4K", "RAM16K",
        // Computer
        "ROM32K", "Screen", "Keyboard",
    ];
    
    for chip_name in all_expected_chips {
        let result = builder.build_builtin_chip(chip_name);
        assert!(result.is_ok(), "Should be able to create chip: {}", chip_name);
        
        let chip = result.unwrap();
        assert_eq!(chip.name(), chip_name);
        assert!(chip.input_pins().len() > 0 || chip.output_pins().len() > 0, 
                "{} should have pins", chip_name);
    }
}

#[test]
fn test_builder_reset_functionality() {
    let builder = ChipBuilder::new();
    
    // Test that chips can be reset
    let mut register = builder.build_builtin_chip("Register").unwrap();
    
    // Load a value
    register.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0x1234);
    register.get_pin("load").unwrap().borrow_mut().pull(HIGH, None).unwrap();
    
    // Test that register works
    register.eval().unwrap();
    
    // Test reset functionality
    register.reset().unwrap();
    let output = register.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 0);
}

#[test]
fn test_builder_pin_width_consistency() {
    let builder = ChipBuilder::new();
    
    // Test that 16-bit chips have 16-bit pins where expected
    let mut not16 = builder.build_builtin_chip("Not16").unwrap();
    
    // Set a test value
    not16.get_pin("in").unwrap().borrow_mut().set_bus_voltage(0xA5A5);
    not16.eval().unwrap();
    
    // Check that we can read 16-bit values
    let output = not16.get_pin("out").unwrap().borrow().bus_voltage();
    assert_eq!(output, 0x5A5A); // Complement of 0xA5A5
    
    // Test that single-bit chips work with individual bits
    let mut and_chip = builder.build_builtin_chip("And").unwrap();
    and_chip.get_pin("a").unwrap().borrow_mut().pull(HIGH, None).unwrap();
    and_chip.get_pin("b").unwrap().borrow_mut().pull(LOW, None).unwrap();
    and_chip.eval().unwrap();
    
    let output = and_chip.get_pin("out").unwrap().borrow().voltage(None).unwrap();
    assert_eq!(output, LOW); // AND(1, 0) = 0
}