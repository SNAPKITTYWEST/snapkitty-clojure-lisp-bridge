//! Shared memory state persistence

use std::collections::HashMap;

pub struct SharedMemory {
    state: HashMap<String, Vec<u8>>,
}

impl SharedMemory {
    pub fn new() -> Self {
        SharedMemory {
            state: HashMap::new(),
        }
    }

    pub fn write(&mut self, key: &str, value: Vec<u8>) {
        self.state.insert(key.to_string(), value);
    }

    pub fn read(&self, key: &str) -> Option<&[u8]> {
        self.state.get(key).map(|v| v.as_slice())
    }

    pub fn clear(&mut self) {
        self.state.clear();
    }
}

impl Default for SharedMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_memory() {
        let mut mem = SharedMemory::new();
        mem.write("key1", vec![1, 2, 3]);
        assert_eq!(mem.read("key1"), Some(&[1, 2, 3][..]));
    }
}
