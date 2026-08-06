//! Core types for HyperKitty

use serde::{Serialize, Deserialize};
use std::fmt;

/// Identity type - wrapper around Vec<u8>
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Identity(pub Vec<u8>);

impl Identity {
    pub fn new(id: Vec<u8>) -> Self { Self(id) }
    pub fn as_bytes(&self) -> &[u8] { &self.0 }
}

/// Signature type - wrapper around Vec<u8>
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Signature(pub Vec<u8>);

impl Signature {
    pub fn new(sig: Vec<u8>) -> Self { Self(sig) }
    pub fn as_bytes(&self) -> &[u8] { &self.0 }
}

/// Hash type - wrapper around Vec<u8>
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash(pub Vec<u8>);

impl Hash {
    pub fn new(hash: Vec<u8>) -> Self { Self(hash) }
    pub fn as_bytes(&self) -> &[u8] { &self.0 }
}

/// Nonce type
pub type Nonce = u64;

/// Timestamp type
pub type Timestamp = u64;

/// Proof ID type - wrapper around Vec<u8>
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProofId(pub Vec<u8>);

impl ProofId {
    pub fn new(id: Vec<u8>) -> Self { Self(id) }
}

/// Receipt ID type - wrapper around Vec<u8>
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReceiptId(pub Vec<u8>);

impl ReceiptId {
    pub fn new(id: Vec<u8>) -> Self { Self(id) }
}
