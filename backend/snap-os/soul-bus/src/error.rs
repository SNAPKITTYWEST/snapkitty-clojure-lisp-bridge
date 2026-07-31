use soul_agent::AgentError;

#[derive(Debug, PartialEq, Eq)]
pub enum BusError {
    UnknownSoul(u64),
    AgentError(AgentError),
    BusPoisoned,
}

impl From<AgentError> for BusError {
    fn from(error: AgentError) -> Self {
        Self::AgentError(error)
    }
}

impl std::fmt::Display for BusError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownSoul(soul_id) => write!(formatter, "unknown soul: {soul_id}"),
            Self::AgentError(error) => write!(formatter, "agent error: {error}"),
            Self::BusPoisoned => formatter.write_str("soul bus registry poisoned"),
        }
    }
}

impl std::error::Error for BusError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::AgentError(error) => Some(error),
            Self::UnknownSoul(_) | Self::BusPoisoned => None,
        }
    }
}
