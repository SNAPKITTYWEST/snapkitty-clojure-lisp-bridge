//! Seed chain: cryptographic state derivation

pub struct SeedChain {
    seeds: Vec<Vec<u8>>,
}

impl SeedChain {
    pub fn new(initial_seed: &[u8]) -> Self {
        SeedChain {
            seeds: vec![initial_seed.to_vec()],
        }
    }

    pub fn derive_next(&mut self) {
        let last = &self.seeds[self.seeds.len() - 1];
        let mut next = last.clone();
        // Simple derivation: increment first byte
        if let Some(first) = next.first_mut() {
            *first = first.wrapping_add(1);
        }
        self.seeds.push(next);
    }

    pub fn current_seed(&self) -> Vec<u8> {
        self.seeds.last().cloned().unwrap_or_default()
    }

    pub fn len(&self) -> usize {
        self.seeds.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_chain_derivation() {
        let mut chain = SeedChain::new(&[1, 2, 3]);
        chain.derive_next();
        assert_eq!(chain.len(), 2);
    }
}
