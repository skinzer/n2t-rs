/// Basic memory implementation for RAM chips
/// Stores 16-bit words in an internal array
#[derive(Debug, Clone)]
pub struct Memory {
    data: Vec<u16>,
    size: usize,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
            size,
        }
    }
    
    pub fn get(&self, address: usize) -> u16 {
        if address >= self.size {
            // Out of bounds returns 0xFFFF (as in TypeScript implementation)
            return 0xffff;
        }
        self.data[address]
    }
    
    pub fn set(&mut self, address: usize, value: u16) {
        if address < self.size {
            self.data[address] = value & 0xffff; // Mask to 16 bits
        }
    }
    
    pub fn reset(&mut self) {
        self.data.fill(0);
    }
    
    pub fn size(&self) -> usize {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_basic_operations() {
        let mut memory = Memory::new(8);
        
        // Test initial state
        assert_eq!(memory.get(0), 0);
        assert_eq!(memory.get(7), 0);
        
        // Test set/get
        memory.set(0, 0x1234);
        memory.set(7, 0x5678);
        assert_eq!(memory.get(0), 0x1234);
        assert_eq!(memory.get(7), 0x5678);
        
        // Test out of bounds
        assert_eq!(memory.get(8), 0xffff);
        memory.set(8, 0x9999); // Should not crash
        
        // Test reset
        memory.reset();
        assert_eq!(memory.get(0), 0);
        assert_eq!(memory.get(7), 0);
    }
    
    #[test]
    fn test_memory_value_masking() {
        let mut memory = Memory::new(1);
        
        // Test 16-bit masking
        memory.set(0, 0x1_2345_u32 as u16); // 17-bit value cast to u16
        assert_eq!(memory.get(0), 0x2345); // Should be masked to 16 bits
    }
}