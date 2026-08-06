//! Environment bitmask persistence

pub struct EnvironmentBitmask {
    mask: u64,
}

impl EnvironmentBitmask {
    pub fn new() -> Self {
        EnvironmentBitmask { mask: 0 }
    }

    pub fn set_bit(&mut self, idx: usize) {
        if idx < 64 {
            self.mask |= 1 << idx;
        }
    }

    pub fn get_bit(&self, idx: usize) -> bool {
        if idx < 64 {
            (self.mask & (1 << idx)) != 0
        } else {
            false
        }
    }

    pub fn to_bytes(&self) -> [u8; 8] {
        self.mask.to_le_bytes()
    }

    pub fn from_bytes(bytes: &[u8; 8]) -> Self {
        EnvironmentBitmask {
            mask: u64::from_le_bytes(*bytes),
        }
    }
}

impl Default for EnvironmentBitmask {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_bit() {
        let mut mask = EnvironmentBitmask::new();
        mask.set_bit(5);
        assert!(mask.get_bit(5));
    }

    #[test]
    fn round_trip() {
        let mut mask = EnvironmentBitmask::new();
        mask.set_bit(10);
        let bytes = mask.to_bytes();
        let mask2 = EnvironmentBitmask::from_bytes(&bytes);
        assert!(mask2.get_bit(10));
    }
}
