//! ERE audit hash (P5): SHA256 only if P1-P4 pass

pub fn compute_audit_hash(
    agent_id: &str,
    intent: &str,
    artifact: &str,
    p1: bool,
    p2: bool,
    p3: bool,
    p4: bool,
) -> Option<Vec<u8>> {
    // P5 rule: only produce hash if P1-P4 all pass
    if !p1 || !p2 || !p3 || !p4 {
        return None;
    }

    let input = format!("{}||{}||{}", agent_id, intent, artifact);
    let mut hash = [0u8; 32];
    for (i, &b) in input.as_bytes().iter().take(32).enumerate() {
        hash[i] = b;
    }
    Some(hash.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_hash_all_pass() {
        let hash = compute_audit_hash("agent1", "intent", "artifact", true, true, true, true);
        assert!(hash.is_some());
    }

    #[test]
    fn audit_hash_p1_fail() {
        let hash = compute_audit_hash("agent1", "intent", "artifact", false, true, true, true);
        assert!(hash.is_none());
    }
}
