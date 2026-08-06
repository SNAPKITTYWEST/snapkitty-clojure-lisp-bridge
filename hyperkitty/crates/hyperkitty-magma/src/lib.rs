pub mod envelope;
pub mod identity;
pub mod roles;
pub mod signature;
pub mod nonce;

pub use envelope::MagmaEnvelope;
pub use identity::AgentIdentity;
pub use roles::Role;

pub fn create_envelope() -> MagmaEnvelope {
    MagmaEnvelope::new()
}
