#[derive(Debug, PartialEq, Eq)]
pub enum AgentError {
    AgentDead,
    CompileError(String),
    NoSuchFunc(String),
    ExecError(String),
    DialectError(String),
}

impl std::fmt::Display for AgentError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AgentDead => formatter.write_str("soul agent is not running"),
            Self::CompileError(message) => write!(formatter, "compile error: {message}"),
            Self::NoSuchFunc(name) => write!(formatter, "no such function: {name}"),
            Self::ExecError(message) => write!(formatter, "execution error: {message}"),
            Self::DialectError(message) => write!(formatter, "dialect error: {message}"),
        }
    }
}

impl std::error::Error for AgentError {}
