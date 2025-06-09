use n2t_simulator::prelude::*;
use n2t_simulator::chip::pin::{HIGH, LOW};

fn main() -> Result<()> {
    println!("🔬 nand2tetris Rust Simulator");
    println!("=====================================");
    
    // Demonstrate HDL parsing and chip simulation
    let mut parser = HdlParser::new()?;
    let builder = ChipBuilder::new();
    
    println!("\n📝 Parsing HDL and building chips...");
    
    // Parse and simulate a NOT gate
    let not_hdl = r#"
        CHIP Not {
            IN in;
            OUT out;
            BUILTIN;
        }
    "#;
    
    println!("   Parsing NOT gate HDL...");
    let hdl_chip = parser.parse(not_hdl)?;
    let mut not_chip = builder.build_chip(&hdl_chip)?;
    
    println!("   ✅ Built {} chip with {} inputs, {} outputs", 
             not_chip.name(), 
             not_chip.input_pins().len(), 
             not_chip.output_pins().len());
    
    // Test NOT gate
    println!("\n🧪 Testing NOT gate truth table:");
    for input_val in [LOW, HIGH] {
        not_chip.get_pin("in")?.borrow_mut().pull(input_val, None)?;
        not_chip.eval()?;
        let output = not_chip.get_pin("out")?.borrow().voltage(None)?;
        println!("   NOT({}) = {}", input_val, output);
    }
    
    // Parse and simulate an AND gate
    let and_hdl = r#"
        CHIP And {
            IN a, b;
            OUT out;
            BUILTIN;
        }
    "#;
    
    println!("\n   Parsing AND gate HDL...");
    let hdl_chip = parser.parse(and_hdl)?;
    let mut and_chip = builder.build_chip(&hdl_chip)?;
    
    // Test AND gate
    println!("\n🧪 Testing AND gate truth table:");
    for a_val in [LOW, HIGH] {
        for b_val in [LOW, HIGH] {
            and_chip.get_pin("a")?.borrow_mut().pull(a_val, None)?;
            and_chip.get_pin("b")?.borrow_mut().pull(b_val, None)?;
            and_chip.eval()?;
            let output = and_chip.get_pin("out")?.borrow().voltage(None)?;
            println!("   AND({}, {}) = {}", a_val, b_val, output);
        }
    }
    
    // Test 16-bit bus system
    println!("\n🚌 Testing 16-bit bus system:");
    let mut bus = Bus::new("demo_bus".to_string(), 16);
    
    // Set a 16-bit value
    bus.set_bus_voltage(0x1234);
    println!("   Set bus to 0x{:04X} ({})", bus.bus_voltage(), bus.bus_voltage());
    
    // Test individual bit access
    println!("   Bit 0: {}, Bit 4: {}, Bit 12: {}", 
             bus.voltage(Some(0))?, 
             bus.voltage(Some(4))?, 
             bus.voltage(Some(12))?);
    
    // Toggle some bits
    bus.toggle(Some(0))?;
    bus.toggle(Some(15))?;
    println!("   After toggling bits 0 and 15: 0x{:04X}", bus.bus_voltage());
    
    println!("\n✅ Simulator demonstration complete!");
    println!("📊 Features implemented:");
    println!("   • HDL parsing with Tree-sitter foundation");
    println!("   • Pin/Bus electrical simulation");
    println!("   • Basic logic gates (NAND, NOT, AND, OR, XOR)");
    println!("   • Chip builder and evaluation engine");
    println!("   • 16-bit bus support");
    println!("   • Comprehensive error handling");
    
    println!("\n🚀 Ready for Phase 2: CPU and VM implementation!");
    
    Ok(())
}