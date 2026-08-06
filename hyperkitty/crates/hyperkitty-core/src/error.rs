use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidRoute, UnauthorizedRoute, RouteConflict, NoValidRoutes,
    InvalidSignature, ReplayedNonce, ProofFailed,
    InvariantViolated(String), BalanceViolated, EntropyExceeded,
    InvalidGlyph, InvalidLedger, InvalidWitness, InvalidTick, InvalidReceipt,
    ParseError(String), LexerError(String), SyntaxError(String), TypeError(String),
    SerializationError(String), DeserializationError(String),
    HashMismatch, SignatureMismatch, TamperDetected,
    StorageError(String), ChainBroken, RecordNotFound,
    NumericalError(String), MatrixError(String), SpectralError(String),
    NotInitialized, AlreadyInitialized, Locked, Timeout,
    Custom(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidRoute => write!(f, "Invalid route"),
            Error::UnauthorizedRoute => write!(f, "Unauthorized route"),
            Error::RouteConflict => write!(f, "Route conflict"),
            Error::NoValidRoutes => write!(f, "No valid routes"),
            Error::InvalidSignature => write!(f, "Invalid signature"),
            Error::ReplayedNonce => write!(f, "Replayed nonce"),
            Error::ProofFailed => write!(f, "Proof failed"),
            Error::InvariantViolated(msg) => write!(f, "Invariant violated: {}", msg),
            Error::BalanceViolated => write!(f, "Balance violated"),
            Error::EntropyExceeded => write!(f, "Entropy exceeded maximum"),
            Error::InvalidGlyph => write!(f, "Invalid glyph"),
            Error::InvalidLedger => write!(f, "Invalid ledger"),
            Error::InvalidWitness => write!(f, "Invalid witness"),
            Error::InvalidTick => write!(f, "Invalid tick"),
            Error::InvalidReceipt => write!(f, "Invalid receipt"),
            Error::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Error::LexerError(msg) => write!(f, "Lexer error: {}", msg),
            Error::SyntaxError(msg) => write!(f, "Syntax error: {}", msg),
            Error::TypeError(msg) => write!(f, "Type error: {}", msg),
            Error::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Error::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
            Error::HashMismatch => write!(f, "Hash mismatch"),
            Error::SignatureMismatch => write!(f, "Signature mismatch"),
            Error::TamperDetected => write!(f, "Tampering detected"),
            Error::StorageError(msg) => write!(f, "Storage error: {}", msg),
            Error::ChainBroken => write!(f, "Chain broken"),
            Error::RecordNotFound => write!(f, "Record not found"),
            Error::NumericalError(msg) => write!(f, "Numerical error: {}", msg),
            Error::MatrixError(msg) => write!(f, "Matrix error: {}", msg),
            Error::SpectralError(msg) => write!(f, "Spectral error: {}", msg),
            Error::NotInitialized => write!(f, "Not initialized"),
            Error::AlreadyInitialized => write!(f, "Already initialized"),
            Error::Locked => write!(f, "Locked"),
            Error::Timeout => write!(f, "Timeout"),
            Error::Custom(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}
