use std::collections::HashMap;

pub struct SharedMemory {
    pub data: HashMap<String, Vec<u8>>,
    pub version: u64,
}

impl SharedMemory {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            version: 0,
        }
    }

    pub fn write(&mut self, key: &str, value: &[u8]) {
        self.data.insert(key.to_string(), value.to_vec());
        self.version += 1;
    }

    pub fn read(&self, key: &str) -> Option<&[u8]> {
        self.data.get(key).map(|v| v.as_slice())
    }

    pub fn version(&self) -> u64 {
        self.version
    }
}

impl Default for SharedMemory {
    fn default() -> Self {
        Self::new()
    }
}
