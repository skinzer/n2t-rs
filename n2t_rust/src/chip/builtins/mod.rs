// Builtin chip implementations organized by category

// These imports are used by the macros below, but the compiler doesn't always detect this
#[allow(unused_imports)]
use std::collections::HashMap;
#[allow(unused_imports)]
use std::rc::Rc;
#[allow(unused_imports)]
use std::cell::RefCell;
#[allow(unused_imports)]
use crate::chip::{ChipInterface, Bus, Pin};
#[allow(unused_imports)]
use crate::error::Result;

/// Helper macro to implement common ChipInterface methods
macro_rules! impl_chip_interface_boilerplate {
    ($chip_name:expr) => {
        fn name(&self) -> &str {
            &self.name
        }
        
        fn input_pins(&self) -> &HashMap<String, Rc<RefCell<dyn Pin>>> {
            &self.input_pins
        }
        
        fn output_pins(&self) -> &HashMap<String, Rc<RefCell<dyn Pin>>> {
            &self.output_pins
        }
        
        fn internal_pins(&self) -> &HashMap<String, Rc<RefCell<dyn Pin>>> {
            &self.internal_pins
        }
        
        fn get_pin(&self, name: &str) -> Result<Rc<RefCell<dyn Pin>>> {
            if let Some(pin) = self.input_pins.get(name) {
                return Ok(pin.clone());
            }
            
            if let Some(pin) = self.output_pins.get(name) {
                return Ok(pin.clone());
            }
            
            Err(crate::error::SimulatorError::Hardware(
                format!("Pin '{}' not found in {} chip", name, $chip_name)
            ))
        }
        
        fn is_input_pin(&self, name: &str) -> bool {
            self.input_pins.contains_key(name)
        }
        
        fn is_output_pin(&self, name: &str) -> bool {
            self.output_pins.contains_key(name)
        }
        
        fn reset(&mut self) -> Result<()> {
            for pin in self.input_pins.values() {
                pin.borrow_mut().set_bus_voltage(0);
            }
            for pin in self.output_pins.values() {
                pin.borrow_mut().set_bus_voltage(0);
            }
            Ok(())
        }
    };
}

/// Helper macro to create a basic chip structure
macro_rules! basic_chip_struct {
    ($name:ident) => {
        #[derive(Debug)]
        pub struct $name {
            name: String,
            input_pins: HashMap<String, Rc<RefCell<dyn Pin>>>,
            output_pins: HashMap<String, Rc<RefCell<dyn Pin>>>,
            internal_pins: HashMap<String, Rc<RefCell<dyn Pin>>>,
        }
    };
}

pub(crate) use impl_chip_interface_boilerplate;
pub(crate) use basic_chip_struct;

// Export all builtin chip modules
pub mod logic;
pub mod arithmetic;
pub mod sequential;
pub mod computer;

// Re-export all chip types for easy access
pub use logic::*;
pub use arithmetic::*;
pub use sequential::*;
pub use computer::*;