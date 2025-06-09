pub type Result<T> = std::result::Result<T, SimulatorError>;

#[derive(Debug, thiserror::Error)]
pub enum SimulatorError {
    #[error("Compilation error: {message}")]
    Compilation {
        message: String,
        span: Option<Span>,
    },
    
    #[error("Hardware error: {0}")]
    Hardware(String),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Test error: {0}")]
    Test(String),
    
    #[error("VM error: {0}")]
    Vm(String),
    
    #[error("CPU error: {0}")]
    Cpu(String),
    
    #[error("Pin '{pin}' not found in chip '{chip}'")]
    PinNotFound {
        pin: String,
        chip: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub source: Option<String>,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            source: None,
        }
    }
    
    pub fn with_source(start: usize, end: usize, source: String) -> Self {
        Self {
            start,
            end,
            source: Some(source),
        }
    }
}