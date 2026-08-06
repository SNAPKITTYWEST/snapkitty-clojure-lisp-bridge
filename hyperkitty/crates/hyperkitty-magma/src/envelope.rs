#[derive(Debug, Clone)]
pub struct MagmaEnvelope {
    pub protocol_version: u8,
    pub message_id: [u8; 16],
    pub sender_id: hyperkitty_core::Identity,
    pub recipient_id: hyperkitty_core::Identity,
}

impl MagmaEnvelope {
    pub fn new() -> Self {
        Self {
            protocol_version: 1,
            message_id: [0u8; 16],
            sender_id: hyperkitty_core::Identity::new(vec![]),
            recipient_id: hyperkitty_core::Identity::new(vec![]),
        }
    }

    pub fn sign(&mut self) -> hyperkitty_core::Result<()> {
        Ok(())
    }

    pub fn verify_signature(&self) -> hyperkitty_core::Result<bool> {
        Ok(true)
    }
}

impl Default for MagmaEnvelope {
    fn default() -> Self {
        Self::new()
    }
}
