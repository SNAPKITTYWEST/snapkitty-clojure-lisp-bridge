//! `BifrostError` — the single error type for the entire bifrost crate.
//!
//! # Design contract
//! Every fallible bifrost operation returns `Result<T, BifrostError>`.
//! Callers MUST treat `WormViolation` as a security event — it means something
//! attempted to overwrite a sealed, content-addressed blob.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BifrostError {
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("sled: {0}")]
    Sled(#[from] sled::Error),

    #[error("CID mismatch — expected {expected}, computed {computed}")]
    CidMismatch { expected: String, computed: String },

    /// A blob whose CID is already in WORM_FS was presented for sealing again.
    /// This MUST be logged as a security event upstream.
    #[error("WORM violation — attempted overwrite of sealed blob {cid}")]
    WormViolation { cid: String },

    #[error("blob not found: {cid}")]
    NotFound { cid: String },

    #[error("signature verification failed on event {cid}")]
    BadSignature { cid: String },

    #[error("broken chain at {cid} — prev {prev:?} missing from DAG")]
    BrokenChain { cid: String, prev: Option<String> },

    #[error("genesis event already exists")]
    GenesisExists,

    #[error("no genesis event — chain is empty")]
    EmptyChain,

    #[error("serialization: {0}")]
    Serialize(String),
}

pub type BifrostResult<T> = Result<T, BifrostError>;
