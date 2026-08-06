#[derive(Debug, Clone)]
pub struct ProofObligation {
    pub id: hyperkitty_core::ProofId,
    pub predicate: String,
}

impl ProofObligation {
    pub fn new(predicate: String) -> Self {
        Self { id: hyperkitty_core::ProofId::new(vec![]), predicate }
    }

    pub fn verify(&self, _world: &crate::WorldState) -> bool { true }
}

#[derive(Debug, Clone)]
pub struct ProofCertificate {
    pub obligation_id: hyperkitty_core::ProofId,
    pub satisfied: bool,
}

impl ProofCertificate {
    pub fn new(obligation_id: hyperkitty_core::ProofId, satisfied: bool) -> Self {
        Self { obligation_id, satisfied }
    }
}
