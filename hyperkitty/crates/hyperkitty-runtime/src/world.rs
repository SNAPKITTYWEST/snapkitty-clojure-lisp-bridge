use std::collections::BTreeMap;
use sha2::{Sha256, Digest};

#[derive(Debug, Clone)]
pub struct WorldState {
    pub facts: BTreeMap<String, String>,
}

impl WorldState {
    pub fn new() -> Self { Self { facts: BTreeMap::new() } }
    pub fn get(&self, key: &str) -> Option<String> { self.facts.get(key).cloned() }
    pub fn set(&mut self, key: String, value: String) { self.facts.insert(key, value); }
    
    pub fn hash(&self) -> hyperkitty_core::Hash {
        let json = format!("{:?}", self.facts);
        let mut h = Sha256::new();
        h.update(json.as_bytes());
        hyperkitty_core::Hash::new(h.finalize().to_vec())
    }
}

impl Default for WorldState {
    fn default() -> Self { Self::new() }
}
