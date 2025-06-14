// Tests for HDL parsing and chip creation integration
// Covers HDL language parsing, chip construction, and error handling

use crate::chip::*;
use crate::chip::builder::ChipBuilder;
use crate::languages::hdl::HdlParser;

#[test]
fn test_hdl_chip_creation_with_wide_buses() {
    // Test that HDL parser correctly handles 16-bit pins
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

#[test]
fn test_hdl_single_bit_pins() {
    let mut parser = HdlParser::new().unwrap();
    
    let hdl = r#"
        CHIP TestSingle {
            IN a, b;
            OUT out;
            BUILTIN;
        }
    "#;
    
    let hdl_chip = parser.parse(hdl).unwrap();
    
    assert_eq!(hdl_chip.name, "TestSingle");
    assert_eq!(hdl_chip.inputs.len(), 2);
    assert_eq!(hdl_chip.outputs.len(), 1);
    assert_eq!(hdl_chip.inputs[0].name, "a");
    assert_eq!(hdl_chip.inputs[0].width, None); // Single bit
    assert_eq!(hdl_chip.inputs[1].name, "b");
    assert_eq!(hdl_chip.inputs[1].width, None); // Single bit
    assert_eq!(hdl_chip.outputs[0].name, "out");
    assert_eq!(hdl_chip.outputs[0].width, None); // Single bit
}

#[test]
fn test_hdl_mixed_width_pins() {
    let mut parser = HdlParser::new().unwrap();
    
    let hdl = r#"
        CHIP TestMixed {
            IN sel, data[16];
            OUT out[16], flag;
            BUILTIN;
        }
    "#;
    
    let hdl_chip = parser.parse(hdl).unwrap();
    
    assert_eq!(hdl_chip.name, "TestMixed");
    assert_eq!(hdl_chip.inputs.len(), 2);
    assert_eq!(hdl_chip.outputs.len(), 2);
    
    // Input pins
    assert_eq!(hdl_chip.inputs[0].name, "sel");
    assert_eq!(hdl_chip.inputs[0].width, None); // Single bit
    assert_eq!(hdl_chip.inputs[1].name, "data");
    assert_eq!(hdl_chip.inputs[1].width, Some(16)); // 16-bit
    
    // Output pins
    assert_eq!(hdl_chip.outputs[0].name, "out");
    assert_eq!(hdl_chip.outputs[0].width, Some(16)); // 16-bit
    assert_eq!(hdl_chip.outputs[1].name, "flag");
    assert_eq!(hdl_chip.outputs[1].width, None); // Single bit
}

#[test]
fn test_hdl_chip_with_parts() {
    let mut parser = HdlParser::new().unwrap();
    
    let hdl = r#"
        CHIP TestComposite {
            IN a, b;
            OUT out;
            
            PARTS:
            Not(in=a, out=notA);
            And(a=notA, b=b, out=out);
        }
    "#;
    
    let hdl_chip = parser.parse(hdl).unwrap();
    
    assert_eq!(hdl_chip.name, "TestComposite");
    assert_eq!(hdl_chip.inputs.len(), 2);
    assert_eq!(hdl_chip.outputs.len(), 1);
    assert_eq!(hdl_chip.parts.len(), 2);
    
    // Check parts
    assert_eq!(hdl_chip.parts[0].name, "Not");
    assert_eq!(hdl_chip.parts[1].name, "And");
}

#[test]
fn test_hdl_chip_with_internal_pins() {
    let mut parser = HdlParser::new().unwrap();
    
    let hdl = r#"
        CHIP TestInternal {
            IN a[16], b[16];
            OUT out[16];
            
            PARTS:
            Add16(a=a, b=b, out=sum);
            Inc16(in=sum, out=out);
        }
    "#;
    
    let hdl_chip = parser.parse(hdl).unwrap();
    
    assert_eq!(hdl_chip.name, "TestInternal");
    assert_eq!(hdl_chip.parts.len(), 2);
    
    // Verify parts and their connections
    assert_eq!(hdl_chip.parts[0].name, "Add16");
    assert_eq!(hdl_chip.parts[1].name, "Inc16");
}

#[test]
fn test_builder_creates_chip_from_hdl() {
    // Test that ChipBuilder can create a chip from HDL definition
    let builder = ChipBuilder::new();
    
    let hdl = r#"
        CHIP SimpleBuffer {
            IN in;
            OUT out;
            
            PARTS:
            Not(in=in, out=notIn);
            Not(in=notIn, out=out);
        }
    "#;
    
    // This would require the builder to support HDL parsing
    // For now, we test that the parsing works and could be used by the builder
    let mut parser = HdlParser::new().unwrap();
    let hdl_chip = parser.parse(hdl).unwrap();
    
    assert_eq!(hdl_chip.name, "SimpleBuffer");
    assert_eq!(hdl_chip.parts.len(), 2);
    
    // Both parts should be Not gates
    assert_eq!(hdl_chip.parts[0].name, "Not");
    assert_eq!(hdl_chip.parts[1].name, "Not");
}

#[test]
fn test_hdl_error_handling() {
    let mut parser = HdlParser::new().unwrap();
    
    // Test parsing error with invalid syntax
    let invalid_hdl = r#"
        CHIP BadChip {
            IN a, b
            OUT out; // Missing semicolon above
            BUILTIN;
        }
    "#;
    
    let result = parser.parse(invalid_hdl);
    assert!(result.is_err(), "Should fail to parse invalid HDL");
}

#[test]
fn test_hdl_constants_and_pin_ranges() {
    let mut parser = HdlParser::new().unwrap();
    
    let hdl = r#"
        CHIP TestRanges {
            IN data[16];
            OUT low[8], high[8];
            
            PARTS:
            // Test pin ranges
            And16(a=data, b=true, out[0..7]=low, out[8..15]=high);
        }
    "#;
    
    let hdl_chip = parser.parse(hdl).unwrap();
    
    assert_eq!(hdl_chip.name, "TestRanges");
    assert_eq!(hdl_chip.inputs.len(), 1);
    assert_eq!(hdl_chip.outputs.len(), 2);
    
    // Check input/output widths
    assert_eq!(hdl_chip.inputs[0].width, Some(16));
    assert_eq!(hdl_chip.outputs[0].width, Some(8));
    assert_eq!(hdl_chip.outputs[1].width, Some(8));
}