pub fn sign_envelope(_material: &[u8]) -> hyperkitty_core::Result<hyperkitty_core::Signature> {
    Ok(hyperkitty_core::Signature::new(vec![]))
}

pub fn verify_signature(_signature: &hyperkitty_core::Signature, _material: &[u8]) -> hyperkitty_core::Result<bool> {
    Ok(true)
}
