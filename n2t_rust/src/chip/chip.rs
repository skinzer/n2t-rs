use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::chip::pin::Pin;
use crate::chip::clock::ClockTick;
use crate::chip::subbus::{PinRange, create_input_subbus, create_output_subbus};
use crate::error::{Result, SimulatorError};
use tokio::sync::broadcast;

/// Represents one side of a wire connection
#[derive(Debug, Clone)]
pub struct PinSide {
    pub name: String,
    pub range: Option<PinRange>,
}

impl PinSide {
    pub fn new(name: String) -> Self {
        Self { name, range: None }
    }
    
    pub fn with_range(name: String, range: PinRange) -> Self {
        Self { name, range: Some(range) }
    }
    
    pub fn from_range(range: PinRange) -> Self {
        Self {
            name: range.pin_name.clone(),
            range: Some(range),
        }
    }
}

/// Represents a connection between pins or pin ranges
#[derive(Debug, Clone)]
pub struct Connection {
    pub from: PinSide,  // Source side (chip pin or constant)
    pub to: PinSide,    // Destination side (part pin)
}

impl Connection {
    pub fn new(from: PinSide, to: PinSide) -> Self {
        Self { from, to }
    }
}

/// Errors that can occur during wire operations
#[derive(Debug, Clone)]
pub enum WireError {
    PinNotFound { pin_name: String, chip_name: String },
    WidthMismatch { from_width: usize, to_width: usize, connection: String },
    InvalidRange { pin_name: String, error: String },
    MultipleAssignment { pin_name: String, conflict: String },
    CircularDependency { cycle: Vec<String> },
}

impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WireError::PinNotFound { pin_name, chip_name } => {
                write!(f, "Pin '{}' not found in chip '{}'", pin_name, chip_name)
            }
            WireError::WidthMismatch { from_width, to_width, connection } => {
                write!(f, "Width mismatch in connection '{}': {} -> {}", connection, from_width, to_width)
            }
            WireError::InvalidRange { pin_name, error } => {
                write!(f, "Invalid range for pin '{}': {}", pin_name, error)
            }
            WireError::MultipleAssignment { pin_name, conflict } => {
                write!(f, "Multiple assignment to pin '{}': {}", pin_name, conflict)
            }
            WireError::CircularDependency { cycle } => {
                write!(f, "Circular dependency detected: {}", cycle.join(" -> "))
            }
        }
    }
}

impl std::error::Error for WireError {}

pub trait ChipInterface: std::fmt::Debug {
    fn name(&self) -> &str;
    fn input_pins(&self) -> &HashMap<String, Rc<RefCell<dyn Pin>>>;
    fn output_pins(&self) -> &HashMap<String, Rc<RefCell<dyn Pin>>>;
    fn internal_pins(&self) -> &HashMap<String, Rc<RefCell<dyn Pin>>>;
    
    fn get_pin(&self, name: &str) -> Result<Rc<RefCell<dyn Pin>>>;
    fn is_input_pin(&self, name: &str) -> bool;
    fn is_output_pin(&self, name: &str) -> bool;
    fn eval(&mut self) -> Result<()>;
    fn reset(&mut self) -> Result<()>;
}

pub struct Chip {
    name: String,
    input_pins: HashMap<String, Rc<RefCell<dyn Pin>>>,
    output_pins: HashMap<String, Rc<RefCell<dyn Pin>>>,
    internal_pins: HashMap<String, Rc<RefCell<dyn Pin>>>,
    sub_chips: Vec<Box<dyn ChipInterface>>,
    clock_receiver: Option<broadcast::Receiver<ClockTick>>,
    // Track SubBus instances for propagation
    subbus_connections: Vec<Rc<RefCell<dyn Pin>>>,
}

impl Chip {
    pub fn new(name: String) -> Self {
        Self {
            name,
            input_pins: HashMap::new(),
            output_pins: HashMap::new(),
            internal_pins: HashMap::new(),
            sub_chips: Vec::new(),
            clock_receiver: None,
            subbus_connections: Vec::new(),
        }
    }
    
    pub fn add_input_pin(&mut self, name: String, pin: Rc<RefCell<dyn Pin>>) {
        self.input_pins.insert(name, pin);
    }
    
    pub fn add_output_pin(&mut self, name: String, pin: Rc<RefCell<dyn Pin>>) {
        self.output_pins.insert(name, pin);
    }
    
    pub fn add_internal_pin(&mut self, name: String, pin: Rc<RefCell<dyn Pin>>) {
        self.internal_pins.insert(name, pin);
    }
    
    pub fn add_sub_chip(&mut self, chip: Box<dyn ChipInterface>) {
        self.sub_chips.push(chip);
    }
    
    pub fn connect_pins(&mut self, from_pin: &str, to_pin: &str) -> Result<()> {
        let from = self.get_pin(from_pin)?;
        let to = self.get_pin(to_pin)?;
        
        // Create weak reference to avoid circular references
        let weak_to = Rc::downgrade(&to);
        from.borrow_mut().connect(weak_to);
        
        Ok(())
    }
    
    pub fn subscribe_to_clock(&mut self, receiver: broadcast::Receiver<ClockTick>) {
        self.clock_receiver = Some(receiver);
    }
    
    /// Propagate signals through all SubBus connections
    fn propagate_subbus_signals(&mut self) -> Result<()> {
        // Force all tracked SubBus instances to propagate their current values
        for subbus in &self.subbus_connections {
            if let Ok(mut subbus_pin) = subbus.try_borrow_mut() {
                // Trigger propagation by re-setting the current bus voltage
                let current_voltage = subbus_pin.bus_voltage();
                subbus_pin.set_bus_voltage(current_voltage);
            }
        }
        Ok(())
    }
    
    /// Wire a part chip to this chip with the given connections
    pub fn wire(&mut self, part: Box<dyn ChipInterface>, connections: Vec<Connection>) -> std::result::Result<(), WireError> {
        // Validate all connections first
        for connection in &connections {
            self.validate_connection(part.as_ref(), connection)?;
        }
        
        // Make all connections
        for connection in &connections {
            self.make_connection(part.as_ref(), connection)?;
        }
        
        // Add the part to our sub-chips
        self.sub_chips.push(part);
        
        Ok(())
    }
    
    /// Validate a single connection
    fn validate_connection(&self, part: &dyn ChipInterface, connection: &Connection) -> std::result::Result<(), WireError> {
        // Check if the part pin is an input or output to determine connection direction
        let is_part_input = part.is_input_pin(&connection.to.name);
        let is_part_output = part.is_output_pin(&connection.to.name);
        
        if !is_part_input && !is_part_output {
            return Err(WireError::PinNotFound {
                pin_name: connection.to.name.clone(),
                chip_name: part.name().to_string(),
            });
        }
        
        if is_part_input {
            // Input connection: host chip -> part's input pin
            self.validate_input_connection(part, connection)
        } else {
            // Output connection: part's output pin -> host chip  
            self.validate_output_connection(part, connection)
        }
    }
    
    /// Validate connection to part's input pin (host chip -> part)
    fn validate_input_connection(&self, part: &dyn ChipInterface, connection: &Connection) -> std::result::Result<(), WireError> {
        let from_pin = self.resolve_pin_side(&connection.from, "from")?;
        let to_pin = part.get_pin(&connection.to.name)
            .map_err(|_| WireError::PinNotFound {
                pin_name: connection.to.name.clone(),
                chip_name: part.name().to_string(),
            })?;
        
        // Calculate effective widths considering ranges
        let from_width = if let Some(range) = &connection.from.range {
            range.width()
        } else {
            from_pin.borrow().width()
        };
        
        let to_width = if let Some(range) = &connection.to.range {
            range.width()
        } else {
            to_pin.borrow().width()
        };
        
        // Check width compatibility
        if from_width != to_width {
            return Err(WireError::WidthMismatch {
                from_width,
                to_width,
                connection: format!("{}={}", connection.to.name, connection.from.name),
            });
        }
        
        Ok(())
    }
    
    /// Validate connection from part's output pin (part -> host chip)
    fn validate_output_connection(&self, part: &dyn ChipInterface, connection: &Connection) -> std::result::Result<(), WireError> {
        let from_pin = part.get_pin(&connection.to.name)  // Note: connection.to is the part pin name
            .map_err(|_| WireError::PinNotFound {
                pin_name: connection.to.name.clone(),
                chip_name: part.name().to_string(),
            })?;
        let to_pin = self.resolve_pin_side(&connection.from, "to")?; // Note: connection.from is the host pin name
        
        // Calculate effective widths considering ranges
        let from_width = if let Some(range) = &connection.to.range {
            range.width()
        } else {
            from_pin.borrow().width()
        };
        
        let to_width = if let Some(range) = &connection.from.range {
            range.width()
        } else {
            to_pin.borrow().width()
        };
        
        // Check width compatibility
        if from_width != to_width {
            return Err(WireError::WidthMismatch {
                from_width,
                to_width,
                connection: format!("{}={}", connection.to.name, connection.from.name),
            });
        }
        
        Ok(())
    }
    
    /// Make a single connection between pins
    fn make_connection(&mut self, part: &dyn ChipInterface, connection: &Connection) -> std::result::Result<(), WireError> {
        // Check if the part pin is an input or output to determine connection direction
        let is_part_input = part.is_input_pin(&connection.to.name);
        let is_part_output = part.is_output_pin(&connection.to.name);
        
        if !is_part_input && !is_part_output {
            return Err(WireError::PinNotFound {
                pin_name: connection.to.name.clone(),
                chip_name: part.name().to_string(),
            });
        }
        
        if is_part_input {
            // Connect FROM host chip TO part's input pin
            self.make_input_connection(part, connection)
        } else {
            // Connect FROM part's output pin TO host chip  
            self.make_output_connection(part, connection)
        }
    }
    
    /// Make connection to part's input pin (host chip -> part)
    fn make_input_connection(&mut self, part: &dyn ChipInterface, connection: &Connection) -> std::result::Result<(), WireError> {
        let from_pin = self.resolve_pin_side(&connection.from, "from")?;
        let to_pin = part.get_pin(&connection.to.name)
            .map_err(|_| WireError::PinNotFound {
                pin_name: connection.to.name.clone(),
                chip_name: part.name().to_string(),
            })?;
        
        // Create SubBus wrappers if needed
        let effective_from_pin = if let Some(range) = &connection.from.range {
            let subbus = create_output_subbus(from_pin, range)
                .map_err(|e| WireError::InvalidRange {
                    pin_name: connection.from.name.clone(),
                    error: e.to_string(),
                })?;
            // Track the SubBus for propagation
            self.subbus_connections.push(subbus.clone());
            subbus
        } else {
            from_pin
        };
        
        let effective_to_pin = if let Some(range) = &connection.to.range {
            let subbus = create_input_subbus(to_pin, range)
                .map_err(|e| WireError::InvalidRange {
                    pin_name: connection.to.name.clone(),
                    error: e.to_string(),
                })?;
            // Track the SubBus for propagation
            self.subbus_connections.push(subbus.clone());
            subbus
        } else {
            to_pin
        };
        
        // Make the connection: from host -> to part input
        let weak_to = Rc::downgrade(&effective_to_pin);
        effective_from_pin.borrow_mut().connect(weak_to);
        
        Ok(())
    }
    
    /// Make connection from part's output pin (part -> host chip)
    fn make_output_connection(&mut self, part: &dyn ChipInterface, connection: &Connection) -> std::result::Result<(), WireError> {
        let from_pin = part.get_pin(&connection.to.name)  // Note: connection.to is the part pin name
            .map_err(|_| WireError::PinNotFound {
                pin_name: connection.to.name.clone(),
                chip_name: part.name().to_string(),
            })?;
        let to_pin = self.resolve_pin_side(&connection.from, "to")?; // Note: connection.from is the host pin name
        
        // Create SubBus wrappers if needed  
        let effective_from_pin = if let Some(range) = &connection.to.range {
            let subbus = create_output_subbus(from_pin, range)
                .map_err(|e| WireError::InvalidRange {
                    pin_name: connection.to.name.clone(),
                    error: e.to_string(),
                })?;
            // Track the SubBus for propagation
            self.subbus_connections.push(subbus.clone());
            subbus
        } else {
            from_pin
        };
        
        let effective_to_pin = if let Some(range) = &connection.from.range {
            let subbus = create_input_subbus(to_pin, range)
                .map_err(|e| WireError::InvalidRange {
                    pin_name: connection.from.name.clone(),
                    error: e.to_string(),
                })?;
            // Track the SubBus for propagation
            self.subbus_connections.push(subbus.clone());
            subbus
        } else {
            to_pin
        };
        
        // Make the connection: from part output -> to host
        let weak_to = Rc::downgrade(&effective_to_pin);
        effective_from_pin.borrow_mut().connect(weak_to);
        
        Ok(())
    }
    
    /// Resolve a pin side to an actual pin, handling constants
    fn resolve_pin_side(&self, pin_side: &PinSide, _context: &str) -> std::result::Result<Rc<RefCell<dyn Pin>>, WireError> {
        match pin_side.name.as_str() {
            "true" => {
                // Create a constant HIGH pin
                use crate::chip::Bus;
                let constant_pin = Rc::new(RefCell::new(Bus::new("true".to_string(), 1)));
                constant_pin.borrow_mut().set_bus_voltage(1);
                Ok(constant_pin as Rc<RefCell<dyn Pin>>)
            }
            "false" => {
                // Create a constant LOW pin
                use crate::chip::Bus;
                let constant_pin = Rc::new(RefCell::new(Bus::new("false".to_string(), 1)));
                constant_pin.borrow_mut().set_bus_voltage(0);
                Ok(constant_pin as Rc<RefCell<dyn Pin>>)
            }
            _ => {
                self.get_pin(&pin_side.name)
                    .map_err(|_| WireError::PinNotFound {
                        pin_name: pin_side.name.clone(),
                        chip_name: self.name.clone(),
                    })
            }
        }
    }
}

impl ChipInterface for Chip {
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
        
        if let Some(pin) = self.internal_pins.get(name) {
            return Ok(pin.clone());
        }
        
        Err(SimulatorError::Hardware(
            format!("Pin '{}' not found in chip '{}'", name, self.name)
        ))
    }
    
    fn is_input_pin(&self, name: &str) -> bool {
        self.input_pins.contains_key(name)
    }
    
    fn is_output_pin(&self, name: &str) -> bool {
        self.output_pins.contains_key(name)
    }
    
    fn eval(&mut self) -> Result<()> {
        // First, propagate signals through SubBus connections
        self.propagate_subbus_signals()?;
        
        // Then evaluate all sub-chips in dependency order
        for sub_chip in &mut self.sub_chips {
            sub_chip.eval()?;
        }
        
        // Finally, propagate any output signals back through SubBus connections
        self.propagate_subbus_signals()?;
        
        Ok(())
    }
    
    fn reset(&mut self) -> Result<()> {
        // Reset all sub-chips
        for sub_chip in &mut self.sub_chips {
            sub_chip.reset()?;
        }
        
        // Reset all pins to LOW
        for pin in self.input_pins.values() {
            pin.borrow_mut().set_bus_voltage(0);
        }
        
        for pin in self.output_pins.values() {
            pin.borrow_mut().set_bus_voltage(0);
        }
        
        for pin in self.internal_pins.values() {
            pin.borrow_mut().set_bus_voltage(0);
        }
        
        Ok(())
    }
}

use std::fmt;

impl fmt::Debug for Chip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Chip")
            .field("name", &self.name)
            .field("input_pins", &self.input_pins.len())
            .field("output_pins", &self.output_pins.len())
            .field("internal_pins", &self.internal_pins.len())
            .field("sub_chips", &self.sub_chips.len())
            .finish()
    }
}

