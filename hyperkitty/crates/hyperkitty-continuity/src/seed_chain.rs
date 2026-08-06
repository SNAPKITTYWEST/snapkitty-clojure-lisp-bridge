pub struct SeedChain {
    pub seed: [u8; 32],
    pub index: u64,
}

impl SeedChain {
    pub fn genesis(initial_seed: &[u8; 32]) -> Self {
        Self {
            seed: *initial_seed,
            index: 0,
        }
    }

    pub fn advance(&mut self, _operation: &[u8]) -> [u8; 32] {
        self.index += 1;
        self.seed
    }

    pub fn current_seed(&self) -> &[u8; 32] {
        &self.seed
    }

    pub fn index(&self) -> u64 {
        self.index
    }
}
