use std::collections::HashSet;

pub struct NonceRegistry {
    pub seen: HashSet<u64>,
}

impl NonceRegistry {
    pub fn new() -> Self {
        Self {
            seen: HashSet::new(),
        }
    }

    pub fn check_and_register(&mut self, nonce: u64) -> hyperkitty_core::Result<()> {
        if self.seen.contains(&nonce) {
            Err(hyperkitty_core::Error::ReplayedNonce)
        } else {
            self.seen.insert(nonce);
            Ok(())
        }
    }
}

impl Default for NonceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
