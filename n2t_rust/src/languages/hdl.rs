use crate::error::{Result, SimulatorError};

#[derive(Debug, Clone)]
pub struct HdlChip {
    pub name: String,
    pub inputs: Vec<PinDecl>,
    pub outputs: Vec<PinDecl>,
    pub parts: Vec<Part>,
    pub is_builtin: bool,
    pub clocked_pins: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PinDecl {
    pub name: String,
    pub width: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct Part {
    pub name: String,
    pub connections: Vec<Wire>,
}

#[derive(Debug, Clone)]
pub struct Wire {
    pub from: WireSide,
    pub to: WireSide,
}

#[derive(Debug, Clone)]
pub enum WireSide {
    Pin { name: String, range: Option<crate::chip::subbus::PinRange> },
    Constant(bool),
}

pub struct HdlParser {
    // For now, we'll implement a simple recursive descent parser
    // Later we can integrate Tree-sitter with pre-generated grammars
}

impl HdlParser {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
    
    pub fn parse(&mut self, source: &str) -> Result<HdlChip> {
        // Simple parser implementation for HDL
        // This is a placeholder that recognizes basic HDL structure
        
        let lines: Vec<&str> = source.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with("//"))
            .collect();
        
        if lines.is_empty() {
            return Err(SimulatorError::Parse("Empty HDL file".to_string()));
        }
        
        // Parse CHIP declaration
        let chip_line = lines.get(0)
            .ok_or_else(|| SimulatorError::Parse("No CHIP declaration found".to_string()))?;
        
        if !chip_line.starts_with("CHIP ") {
            return Err(SimulatorError::Parse("Expected CHIP declaration".to_string()));
        }
        
        let name = chip_line[5..].trim_end_matches(" {").trim().to_string();
        
        // Look for BUILTIN
        let is_builtin = lines.iter().any(|line| line.trim() == "BUILTIN;");
        
        // Parse input pins
        let inputs = self.parse_pin_section(&lines, "IN")?;
        
        // Parse output pins  
        let outputs = self.parse_pin_section(&lines, "OUT")?;
        
        // Parse parts
        let parts = if !is_builtin {
            self.parse_parts_section(&lines)?
        } else {
            Vec::new()
        };
        
        // Parse clocked pins (simplified)
        let clocked_pins = Vec::new(); // TODO: Implement clocked parsing
        
        Ok(HdlChip {
            name,
            inputs,
            outputs,
            parts,
            is_builtin,
            clocked_pins,
        })
    }
    
    fn parse_pin_section(&self, lines: &[&str], section: &str) -> Result<Vec<PinDecl>> {
        let mut pins = Vec::new();
        
        for line in lines {
            if line.starts_with(section) && line.contains(" ") {
                let pin_part = line[section.len()..].trim_start();
                if let Some(semicolon_pos) = pin_part.find(';') {
                    let pin_list = &pin_part[..semicolon_pos].trim();
                    
                    // Parse comma-separated pins
                    for pin_str in pin_list.split(',') {
                        let pin_str = pin_str.trim();
                        if !pin_str.is_empty() {
                            pins.push(self.parse_pin_decl(pin_str)?);
                        }
                    }
                }
                break;
            }
        }
        
        Ok(pins)
    }
    
    fn parse_pin_decl(&self, pin_str: &str) -> Result<PinDecl> {
        // Parse pin declarations like "a", "b[16]", etc.
        if let Some(bracket_pos) = pin_str.find('[') {
            let name = pin_str[..bracket_pos].trim().to_string();
            let width_str = &pin_str[bracket_pos + 1..];
            if let Some(end_bracket) = width_str.find(']') {
                let width_num = width_str[..end_bracket].trim();
                let width = width_num.parse::<u16>()
                    .map_err(|e| SimulatorError::Parse(format!("Invalid pin width '{}': {}", width_num, e)))?;
                Ok(PinDecl { name, width: Some(width) })
            } else {
                Err(SimulatorError::Parse(format!("Unclosed bracket in pin declaration: {}", pin_str)))
            }
        } else {
            Ok(PinDecl { name: pin_str.trim().to_string(), width: None })
        }
    }
    
    fn parse_parts_section(&self, lines: &[&str]) -> Result<Vec<Part>> {
        let mut parts = Vec::new();
        let mut in_parts = false;
        let mut current_part: Option<String> = None;
        let mut current_connections: Vec<Wire> = Vec::new();
        
        for line in lines {
            let line = line.trim();
            
            if line.starts_with("PARTS:") {
                in_parts = true;
                continue;
            }
            
            if !in_parts {
                continue;
            }
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with("//") {
                continue;
            }
            
            // End of chip
            if line == "}" {
                // Finalize current part if any
                if let Some(part_name) = current_part.take() {
                    parts.push(Part {
                        name: part_name,
                        connections: current_connections,
                    });
                }
                break;
            }
            
            // Check for part instantiation that starts and ends on same line
            if let Some(paren_pos) = line.find('(') {
                if line.ends_with(");") {
                    // Complete part on one line: "Not(in=in[0], out=out[0]);"
                    // Finalize previous part if any
                    if let Some(part_name) = current_part.take() {
                        parts.push(Part {
                            name: part_name,
                            connections: current_connections,
                        });
                        current_connections = Vec::new();
                    }
                    
                    // Extract part name and connections
                    let part_name = line[..paren_pos].trim().to_string();
                    let connections_str = &line[paren_pos + 1..line.len() - 2]; // Remove "(" and ");"
                    
                    // Parse connections
                    let mut part_connections = Vec::new();
                    if !connections_str.trim().is_empty() {
                        self.parse_connections_line(connections_str, &mut part_connections)?;
                    }
                    
                    // Add complete part
                    parts.push(Part {
                        name: part_name,
                        connections: part_connections,
                    });
                } else {
                    // Multi-line part: "Not("
                    // Finalize previous part if any
                    if let Some(part_name) = current_part.take() {
                        parts.push(Part {
                            name: part_name,
                            connections: current_connections,
                        });
                        current_connections = Vec::new();
                    }
                    
                    // Start new part
                    current_part = Some(line[..paren_pos].trim().to_string());
                    
                    // Parse connections on same line
                    let rest = &line[paren_pos + 1..];
                    if !rest.trim().is_empty() {
                        self.parse_connections_line(rest, &mut current_connections)?;
                    }
                }
            } else if line.ends_with(");") {
                // End of multi-line part
                let conn_line = &line[..line.len() - 2];
                if !conn_line.trim().is_empty() {
                    self.parse_connections_line(conn_line, &mut current_connections)?;
                }
                
                // Finalize current part
                if let Some(part_name) = current_part.take() {
                    parts.push(Part {
                        name: part_name,
                        connections: current_connections,
                    });
                    current_connections = Vec::new();
                }
            } else {
                // Continuation line with connections
                self.parse_connections_line(line, &mut current_connections)?;
            }
        }
        
        Ok(parts)
    }
    
    fn parse_connections_line(&self, line: &str, connections: &mut Vec<Wire>) -> Result<()> {
        // Parse connections like "in=a, out=b[0..7]"
        for conn in line.split(',') {
            let conn = conn.trim();
            if conn.is_empty() {
                continue;
            }
            
            if let Some(eq_pos) = conn.find('=') {
                let to_side = conn[..eq_pos].trim();
                let from_side = conn[eq_pos + 1..].trim();
                
                let to_wire = self.parse_wire_side(to_side)?;
                let from_wire = self.parse_wire_side(from_side)?;
                
                connections.push(Wire {
                    from: from_wire,
                    to: to_wire,
                });
            }
        }
        
        Ok(())
    }
    
    fn parse_wire_side(&self, side: &str) -> Result<WireSide> {
        let side = side.trim();
        
        // Check for boolean constants
        if side == "true" || side == "1" {
            return Ok(WireSide::Constant(true));
        }
        if side == "false" || side == "0" {
            return Ok(WireSide::Constant(false));
        }
        
        // Parse pin with optional range
        let pin_range = crate::chip::subbus::parse_pin_range(side)?;
        let pin_name = pin_range.pin_name.clone();
        let is_full_pin = pin_range.is_full_pin();
        
        Ok(WireSide::Pin {
            name: pin_name,
            range: if is_full_pin {
                None
            } else {
                Some(pin_range)
            },
        })
    }
}

impl Default for HdlParser {
    fn default() -> Self {
        Self::new().expect("Failed to create HDL parser")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_chip_parse() {
        let mut parser = HdlParser::new().unwrap();
        
        let hdl = r#"
            CHIP Not {
                IN in;
                OUT out;
                BUILTIN;
            }
        "#;
        
        let result = parser.parse(hdl).unwrap();
        assert_eq!(result.name, "Not");
        assert_eq!(result.inputs.len(), 1);
        assert_eq!(result.inputs[0].name, "in");
        assert_eq!(result.outputs.len(), 1);
        assert_eq!(result.outputs[0].name, "out");
        assert!(result.is_builtin);
    }
    
    #[test]
    fn test_chip_with_widths() {
        let mut parser = HdlParser::new().unwrap();
        
        let hdl = r#"
            CHIP Add16 {
                IN a[16], b[16];
                OUT out[16];
                BUILTIN;
            }
        "#;
        
        let result = parser.parse(hdl).unwrap();
        assert_eq!(result.name, "Add16");
        assert_eq!(result.inputs.len(), 2);
        assert_eq!(result.inputs[0].name, "a");
        assert_eq!(result.inputs[0].width, Some(16));
        assert_eq!(result.inputs[1].name, "b");
        assert_eq!(result.inputs[1].width, Some(16));
        assert_eq!(result.outputs[0].width, Some(16));
    }
    
    #[test]
    fn test_chip_with_parts_and_pin_ranges() {
        let mut parser = HdlParser::new().unwrap();
        
        let hdl = r#"
            CHIP Not2 {
                IN in[2];
                OUT out[2];
                PARTS:
                Not(in=in[0], out=out[0]);
                Not(in=in[1], out=out[1]);
            }
        "#;
        
        let result = parser.parse(hdl).unwrap();
        assert_eq!(result.name, "Not2");
        assert_eq!(result.inputs.len(), 1);
        assert_eq!(result.inputs[0].name, "in");
        assert_eq!(result.inputs[0].width, Some(2));
        assert_eq!(result.outputs.len(), 1);
        assert_eq!(result.outputs[0].name, "out");
        assert_eq!(result.outputs[0].width, Some(2));
        
        // Check parts
        assert_eq!(result.parts.len(), 2);
        assert_eq!(result.parts[0].name, "Not");
        assert_eq!(result.parts[1].name, "Not");
        
        // Check connections with pin ranges
        assert_eq!(result.parts[0].connections.len(), 2);
        
        // Check first connection: in=in[0]
        if let WireSide::Pin { name, range } = &result.parts[0].connections[0].to {
            assert_eq!(name, "in");
            assert!(range.is_none()); // input pin of Not chip is full width
        }
        if let WireSide::Pin { name, range } = &result.parts[0].connections[0].from {
            assert_eq!(name, "in");
            assert!(range.is_some());
            let range = range.as_ref().unwrap();
            assert_eq!(range.start_index(), 0);
            assert_eq!(range.end_index(), 0);
            assert!(range.is_single_bit());
        }
    }
    
    #[test]
    fn test_pin_range_parsing_in_hdl() {
        let parser = HdlParser::new().unwrap();
        
        // Test wire side parsing
        let wire_side = parser.parse_wire_side("a[0..7]").unwrap();
        if let WireSide::Pin { name, range } = wire_side {
            assert_eq!(name, "a");
            assert!(range.is_some());
            let range = range.unwrap();
            assert_eq!(range.start_index(), 0);
            assert_eq!(range.end_index(), 7);
            assert_eq!(range.width(), 8);
        }
        
        // Test single bit
        let wire_side = parser.parse_wire_side("b[5]").unwrap();
        if let WireSide::Pin { name, range } = wire_side {
            assert_eq!(name, "b");
            assert!(range.is_some());
            let range = range.unwrap();
            assert_eq!(range.start_index(), 5);
            assert_eq!(range.end_index(), 5);
            assert!(range.is_single_bit());
        }
        
        // Test constants
        let wire_side = parser.parse_wire_side("true").unwrap();
        assert!(matches!(wire_side, WireSide::Constant(true)));
        
        let wire_side = parser.parse_wire_side("false").unwrap();
        assert!(matches!(wire_side, WireSide::Constant(false)));
    }
}