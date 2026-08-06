#[derive(Debug, Clone)]
pub struct AgentIdentity {
    pub id: hyperkitty_core::Identity,
}

impl AgentIdentity {
    pub fn new(id: hyperkitty_core::Identity) -> Self {
        Self { id }
    }
}
