use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    pub soul_id:      u64,
    pub timestamp:    u64,   // unix seconds at hydration time
    pub chain_height: u64,
    pub loaded_funcs: Vec<String>,
    pub last_result:  Option<i64>,
    pub custom:       serde_json::Value,
}

impl ContextSnapshot {
    pub fn new(soul_id: u64) -> Self {
        Self {
            soul_id,
            timestamp:    now_secs(),
            chain_height: 0,
            loaded_funcs: Vec::new(),
            last_result:  None,
            custom:       serde_json::Value::Null,
        }
    }
}

#[derive(Clone, Default)]
pub struct ContextHydrator {
    snapshots: Arc<Mutex<HashMap<u64, ContextSnapshot>>>,
}

impl ContextHydrator {
    pub fn new() -> Self { Self::default() }

    pub fn hydrate(&self, soul_id: u64, snap: ContextSnapshot) {
        if let Ok(mut m) = self.snapshots.lock() {
            m.insert(soul_id, snap);
        }
    }

    pub fn get(&self, soul_id: u64) -> Option<ContextSnapshot> {
        self.snapshots.lock().ok()?.get(&soul_id).cloned()
    }

    /// Update a single key inside the `custom` JSON blob.
    pub fn freshen(&self, soul_id: u64, key: &str, value: serde_json::Value) {
        if let Ok(mut m) = self.snapshots.lock() {
            if let Some(snap) = m.get_mut(&soul_id) {
                snap.timestamp = now_secs();
                match &mut snap.custom {
                    serde_json::Value::Object(map) => { map.insert(key.to_string(), value); }
                    other => {
                        let mut map = serde_json::Map::new();
                        map.insert(key.to_string(), value);
                        *other = serde_json::Value::Object(map);
                    }
                }
            }
        }
    }

    pub fn age_seconds(&self, soul_id: u64) -> Option<u64> {
        let snap = self.get(soul_id)?;
        Some(now_secs().saturating_sub(snap.timestamp))
    }

    pub fn is_stale(&self, soul_id: u64, max_age_secs: u64) -> bool {
        self.age_seconds(soul_id)
            .map(|age| age >= max_age_secs)
            .unwrap_or(true) // unknown soul = stale
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(soul_id: u64) -> ContextSnapshot {
        ContextSnapshot {
            soul_id,
            timestamp:    now_secs(),
            chain_height: 5,
            loaded_funcs: vec!["emoji_fn_abc".into()],
            last_result:  Some(42),
            custom:       serde_json::json!({ "role": "architect" }),
        }
    }

    #[test]
    fn hydrate_and_retrieve() {
        let h = ContextHydrator::new();
        h.hydrate(1, snap(1));
        let got = h.get(1).expect("should exist");
        assert_eq!(got.soul_id, 1);
        assert_eq!(got.chain_height, 5);
        assert_eq!(got.last_result, Some(42));
        assert_eq!(got.loaded_funcs, vec!["emoji_fn_abc"]);
    }

    #[test]
    fn freshen_updates_key() {
        let h = ContextHydrator::new();
        h.hydrate(2, snap(2));
        h.freshen(2, "status", serde_json::json!("active"));
        let got = h.get(2).unwrap();
        assert_eq!(got.custom["status"], "active");
        assert_eq!(got.custom["role"], "architect"); // existing key preserved
    }

    #[test]
    fn stale_after_threshold() {
        let h = ContextHydrator::new();
        let mut old = snap(3);
        old.timestamp = now_secs().saturating_sub(120); // 2 minutes ago
        h.hydrate(3, old);
        assert!(h.is_stale(3, 60), "120s old should be stale with 60s threshold");
        assert!(!h.is_stale(3, 300), "120s old should be fresh with 300s threshold");
    }

    #[test]
    fn unknown_soul_returns_none() {
        let h = ContextHydrator::new();
        assert!(h.get(99).is_none());
        assert!(h.is_stale(99, 0), "unknown soul is always stale");
    }
}
