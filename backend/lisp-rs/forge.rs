/// FORGE — Agent Collision Registry
///
/// Append-only immutable ledger recording when two agents produce incompatible
/// outputs on the same input. Every CollisionRecord is SHA-256 sealed at write
/// time — the seal covers the full record so tampering is detectable.
///
/// Game-theoretic effect: agents self-optimize to avoid collisions; rank scores
/// reflect cooperation; the system learns toxic vs synergistic agent pairs.
///
/// Decision: dream-cycle-2 — "architectural primitive missing from the mesh"
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use chrono::Utc;

use crate::decision_seal;

// ── Conflict taxonomy ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConflictType {
    /// Agents returned mutually exclusive decisions (e.g., APPROVED vs HOLD).
    Incompatible,
    /// Agents agreed on action but with contradictory reasoning.
    Contradictory,
    /// One agent produced output; the other was silent (timeout / panic).
    Silent,
}

impl ConflictType {
    pub fn entropy_weight(self) -> f64 {
        match self {
            Self::Incompatible   => 1.0,
            Self::Contradictory  => 0.6,
            Self::Silent         => 0.4,
        }
    }
}

// ── CollisionRecord ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionRecord {
    /// Monotonic collision ID — never reused.
    pub id: u64,
    /// Ordered pair of agent names: (loser, winner) lexicographic sort.
    pub agent_pair: [String; 2],
    /// SHA-256 of the raw input that triggered the conflict.
    pub input_hash: String,
    /// Nature of the conflict.
    pub conflict_type: ConflictType,
    /// Which agent's decision was accepted as the resolution.
    pub resolution_winner: String,
    /// Shannon entropy cost: conflict_type.weight × log2(pair_collision_count + 1).
    pub entropy_cost: f64,
    /// ISO-8601 UTC timestamp at record creation.
    pub timestamp: String,
    /// SHA-256 seal over the full record (excluding this field).
    pub seal: String,
}

impl CollisionRecord {
    fn compute_seal(
        id: u64,
        agent_pair: &[String; 2],
        input_hash: &str,
        conflict_type: ConflictType,
        resolution_winner: &str,
        entropy_cost: f64,
        timestamp: &str,
    ) -> String {
        let raw = format!(
            "{}|{}|{}|{}|{:?}|{}|{:.6}",
            id,
            agent_pair[0],
            agent_pair[1],
            input_hash,
            conflict_type,
            resolution_winner,
            entropy_cost,
        );
        // Mix in timestamp to prevent replay of identical conflicts.
        let with_ts = format!("{}|{}", raw, timestamp);
        decision_seal(&with_ts)
    }
}

// ── Registry ──────────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
struct RegistryInner {
    records:       Vec<CollisionRecord>,
    next_id:       u64,
    /// pair_key → collision count, used for entropy calculation.
    pair_counts:   std::collections::HashMap<String, u64>,
}

#[derive(Debug, Clone, Default)]
pub struct ForgeRegistry(Arc<Mutex<RegistryInner>>);

impl ForgeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a new collision. Returns the sealed CollisionRecord.
    pub fn record(
        &self,
        agent_a:           &str,
        agent_b:           &str,
        input:             &str,
        conflict_type:     ConflictType,
        resolution_winner: &str,
    ) -> CollisionRecord {
        let mut inner = self.0.lock().unwrap();

        // Canonical pair key — always alphabetical so (A,B) == (B,A).
        let mut pair = [agent_a.to_string(), agent_b.to_string()];
        pair.sort();
        let pair_key = format!("{}:{}", pair[0], pair[1]);

        let count = inner.pair_counts.entry(pair_key).or_insert(0);
        *count += 1;
        let entropy_cost = conflict_type.entropy_weight() * (*count as f64 + 1.0_f64).log2();

        let id        = inner.next_id;
        inner.next_id += 1;

        // SHA-256 of the raw input — never store the input itself.
        let input_hash = {
            let mut h = Sha256::new();
            h.update(input.as_bytes());
            format!("{:x}", h.finalize())
        };

        let timestamp = Utc::now().to_rfc3339();

        let seal = CollisionRecord::compute_seal(
            id,
            &pair,
            &input_hash,
            conflict_type,
            resolution_winner,
            entropy_cost,
            &timestamp,
        );

        let rec = CollisionRecord {
            id,
            agent_pair: pair,
            input_hash,
            conflict_type,
            resolution_winner: resolution_winner.to_string(),
            entropy_cost,
            timestamp,
            seal,
        };

        inner.records.push(rec.clone());
        rec
    }

    /// All records for a specific agent pair (either order).
    pub fn pair_history(&self, agent_a: &str, agent_b: &str) -> Vec<CollisionRecord> {
        let mut pair = [agent_a.to_string(), agent_b.to_string()];
        pair.sort();
        let inner = self.0.lock().unwrap();
        inner
            .records
            .iter()
            .filter(|r| r.agent_pair == pair)
            .cloned()
            .collect()
    }

    /// Cumulative entropy cost for a pair — high = toxic combination.
    pub fn pair_entropy(&self, agent_a: &str, agent_b: &str) -> f64 {
        self.pair_history(agent_a, agent_b)
            .iter()
            .map(|r| r.entropy_cost)
            .sum()
    }

    /// Total collision count across all pairs.
    pub fn total_collisions(&self) -> usize {
        self.0.lock().unwrap().records.len()
    }

    /// Last N records, newest first.
    pub fn recent(&self, n: usize) -> Vec<CollisionRecord> {
        let inner = self.0.lock().unwrap();
        inner.records.iter().rev().take(n).cloned().collect()
    }

    /// Verify every record's seal is intact. Returns list of corrupted IDs.
    pub fn audit(&self) -> Vec<u64> {
        let inner = self.0.lock().unwrap();
        inner
            .records
            .iter()
            .filter_map(|r| {
                let expected = CollisionRecord::compute_seal(
                    r.id,
                    &r.agent_pair,
                    &r.input_hash,
                    r.conflict_type,
                    &r.resolution_winner,
                    r.entropy_cost,
                    &r.timestamp,
                );
                if expected != r.seal { Some(r.id) } else { None }
            })
            .collect()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_is_sealed() {
        let forge = ForgeRegistry::new();
        let rec   = forge.record("finance", "risk", "approve $50k wire", ConflictType::Incompatible, "risk");
        assert_eq!(rec.seal.len(), 64);
        assert!(rec.seal.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn pair_is_canonical() {
        let forge = ForgeRegistry::new();
        let rec   = forge.record("risk", "finance", "test", ConflictType::Silent, "finance");
        assert!(rec.agent_pair[0] <= rec.agent_pair[1], "pair must be sorted");
    }

    #[test]
    fn entropy_grows_with_collisions() {
        let forge = ForgeRegistry::new();
        let r1 = forge.record("a", "b", "input1", ConflictType::Incompatible, "a");
        let r2 = forge.record("a", "b", "input2", ConflictType::Incompatible, "a");
        assert!(r2.entropy_cost > r1.entropy_cost);
    }

    #[test]
    fn audit_passes_on_clean_registry() {
        let forge = ForgeRegistry::new();
        forge.record("oracle", "sentinel", "payload", ConflictType::Contradictory, "oracle");
        forge.record("forge",  "nexus",    "data",    ConflictType::Silent,        "nexus");
        assert!(forge.audit().is_empty(), "clean registry must pass audit");
    }

    #[test]
    fn total_collisions_count() {
        let forge = ForgeRegistry::new();
        forge.record("a", "b", "x", ConflictType::Incompatible, "a");
        forge.record("c", "d", "y", ConflictType::Silent, "c");
        assert_eq!(forge.total_collisions(), 2);
    }
}
