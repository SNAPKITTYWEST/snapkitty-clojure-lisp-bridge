use sha2::{Sha256, Digest};

#[derive(Debug, Clone)]
pub struct Tick {
    pub index: u64,
    pub sigma_in: crate::WorldState,
    pub action: crate::Action,
    pub sigma_out: crate::WorldState,
    pub receipt: hyperkitty_core::Hash,
}

impl Tick {
    pub fn new(index: u64, sigma_in: crate::WorldState, action: crate::Action, sigma_out: crate::WorldState) -> Self {
        let mut h = Sha256::new();
        h.update(format!("{}{:?}", index, action).as_bytes());
        let receipt = hyperkitty_core::Hash::new(h.finalize().to_vec());
        Self { index, sigma_in, action, sigma_out, receipt }
    }

    pub fn verify(&self) -> bool { true }
}
