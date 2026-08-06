pub struct EnvironmentBitmask {
    pub mask: u64,
    pub env_var_name: String,
}

impl EnvironmentBitmask {
    pub fn new(var_name: &str) -> Self {
        Self {
            mask: 0,
            env_var_name: var_name.to_string(),
        }
    }

    pub fn set_bit(&mut self, position: u8, value: bool) {
        if position < 64 {
            if value {
                self.mask |= 1 << position;
            } else {
                self.mask &= !(1 << position);
            }
        }
    }

    pub fn get_bit(&self, position: u8) -> bool {
        if position < 64 {
            (self.mask & (1 << position)) != 0
        } else {
            false
        }
    }
}
