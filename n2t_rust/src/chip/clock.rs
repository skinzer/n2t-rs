use tokio::sync::broadcast;
use crate::chip::pin::{Voltage, HIGH, LOW};
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct ClockTick {
    pub level: Voltage,
    pub ticks: u64,
}

#[derive(Debug)]
pub struct Clock {
    sender: broadcast::Sender<ClockTick>,
    level: Voltage,
    ticks: u64,
}

impl Clock {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        
        Self {
            sender,
            level: LOW,
            ticks: 0,
        }
    }
    
    pub fn subscribe(&self) -> broadcast::Receiver<ClockTick> {
        self.sender.subscribe()
    }
    
    pub fn tick(&mut self) -> Result<()> {
        self.ticks += 1;
        self.level = if self.level == LOW { HIGH } else { LOW };
        
        let tick = ClockTick {
            level: self.level,
            ticks: self.ticks,
        };
        
        // Ignore send errors (no active receivers)
        let _ = self.sender.send(tick);
        
        Ok(())
    }
    
    pub fn reset(&mut self) {
        self.level = LOW;
        self.ticks = 0;
        
        let tick = ClockTick {
            level: self.level,
            ticks: self.ticks,
        };
        
        let _ = self.sender.send(tick);
    }
    
    pub fn level(&self) -> Voltage {
        self.level
    }
    
    pub fn ticks(&self) -> u64 {
        self.ticks
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}