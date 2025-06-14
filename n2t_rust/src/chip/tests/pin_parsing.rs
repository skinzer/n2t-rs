// Tests related to pin parsing functionality
// Covers pin range parsing, pin references, and SubBus operations

use crate::chip::subbus::{PinRange, parse_pin_range};
use crate::languages::hdl::HdlParser;

#[test]
fn test_pin_range_parsing() {
    // Test single bit pin range
    let range = parse_pin_range("a").unwrap();
    assert_eq!(range.pin_name, "a");
    assert_eq!(range.start, None);
    assert_eq!(range.end, None);
    
    // Test single bit with index
    let range = parse_pin_range("a[5]").unwrap();
    assert_eq!(range.pin_name, "a");
    assert_eq!(range.start, Some(5));
    assert_eq!(range.end, Some(5));
    
    // Test bit range
    let range = parse_pin_range("data[0..7]").unwrap();
    assert_eq!(range.pin_name, "data");
    assert_eq!(range.start, Some(0));
    assert_eq!(range.end, Some(7));
    
    // Test reverse bit range (should normalize)
    let range = parse_pin_range("data[7..0]").unwrap();
    assert_eq!(range.pin_name, "data");
    assert_eq!(range.start, Some(0));
    assert_eq!(range.end, Some(7));
}

#[test]
fn test_pin_range_creation() {
    // Test creating a single bit range
    let range = PinRange::new_single_bit("test".to_string(), 3);
    assert_eq!(range.pin_name, "test");
    assert_eq!(range.start, Some(3));
    assert_eq!(range.end, Some(3));
    assert_eq!(range.width(), 1);
    
    // Test creating a multi-bit range
    let range = PinRange::new_range("data".to_string(), 0, 7).unwrap();
    assert_eq!(range.pin_name, "data");
    assert_eq!(range.start, Some(0));
    assert_eq!(range.end, Some(7));
    assert_eq!(range.width(), 8);
    
    // Test creating a full pin range (no bit specification)
    let range = PinRange::new("input".to_string());
    assert_eq!(range.pin_name, "input");
    assert_eq!(range.start, None);
    assert_eq!(range.end, None);
}

#[test]
fn test_pin_range_validation() {
    // Valid ranges
    assert!(PinRange::new_range("test".to_string(), 0, 15).is_ok());
    // Single bit ranges are always valid
    let _range = PinRange::new_single_bit("test".to_string(), 7);
    // Full pin ranges are always valid
    let _range = PinRange::new("test".to_string());
    
    // Invalid ranges return Err
    let invalid_range = PinRange::new_range("test".to_string(), 15, 0); // End < start
    assert!(invalid_range.is_err());
}

#[test]
fn test_pin_range_in_hdl() {
    let mut parser = HdlParser::new().unwrap();
    
    // Test HDL with pin ranges
    let hdl = r#"
        CHIP TestPinRanges {
            IN data[16];
            OUT low[8], high[8], bit7;
            
            PARTS:
            // Connect lower 8 bits
            And16(a=data, b=true, out[0..7]=low);
            // Connect upper 8 bits  
            And16(a=data, b=true, out[8..15]=high);
            // Connect single bit
            And16(a=data, b=true, out[7]=bit7);
        }
    "#;
    
    let hdl_chip = parser.parse(hdl).unwrap();
    
    assert_eq!(hdl_chip.name, "TestPinRanges");
    assert_eq!(hdl_chip.inputs.len(), 1);
    assert_eq!(hdl_chip.outputs.len(), 3);
    
    // Verify pin widths
    assert_eq!(hdl_chip.inputs[0].width, Some(16)); // data[16]
    assert_eq!(hdl_chip.outputs[0].width, Some(8));  // low[8]
    assert_eq!(hdl_chip.outputs[1].width, Some(8));  // high[8]
    assert_eq!(hdl_chip.outputs[2].width, None);     // bit7 (single bit)
}

#[test]
fn test_pin_range_width_calculation() {
    // Single bit
    let range = PinRange::new_single_bit("test".to_string(), 5);
    assert_eq!(range.width(), 1);
    
    // Multi-bit range
    let range = PinRange::new_range("test".to_string(), 0, 7).unwrap();
    assert_eq!(range.width(), 8);
    
    let range = PinRange::new_range("test".to_string(), 3, 10).unwrap();
    assert_eq!(range.width(), 8); // 10 - 3 + 1
    
    // Full pin (width depends on context, defaults to 1 for calculation)
    let range = PinRange::new("test".to_string());
    assert_eq!(range.width(), 1); // Default assumption
}

// Note: overlap detection and contains_bit methods not yet implemented

#[test]
fn test_parse_pin_error_handling() {
    // Test invalid pin range strings
    assert!(parse_pin_range("").is_err());
    assert!(parse_pin_range("[5]").is_err()); // Missing pin name
    assert!(parse_pin_range("pin[").is_err()); // Incomplete bracket
    assert!(parse_pin_range("pin[5..7").is_err()); // Missing closing bracket
    assert!(parse_pin_range("pin[abc]").is_err()); // Non-numeric index
    assert!(parse_pin_range("pin[5..abc]").is_err()); // Non-numeric range
}

#[test]
fn test_complex_pin_expressions() {
    // Test parsing of more complex pin expressions
    let range = parse_pin_range("memory[1024]").unwrap();
    assert_eq!(range.pin_name, "memory");
    assert_eq!(range.start, Some(1024));
    assert_eq!(range.end, Some(1024));
    
    let range = parse_pin_range("data[0..15]").unwrap();
    assert_eq!(range.pin_name, "data");
    assert_eq!(range.start, Some(0));
    assert_eq!(range.end, Some(15));
    assert_eq!(range.width(), 16);
}