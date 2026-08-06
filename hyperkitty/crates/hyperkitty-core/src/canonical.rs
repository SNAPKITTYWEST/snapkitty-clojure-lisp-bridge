use super::Glyph;

pub fn canonical_json<T: serde::Serialize>(value: &T) -> Result<String, super::Error> {
    serde_json::to_string(value).map_err(|e| super::Error::SerializationError(e.to_string()))
}

pub fn canonical_from_json<T: serde::de::DeserializeOwned>(json: &str) -> Result<T, super::Error> {
    serde_json::from_str(json).map_err(|e| super::Error::DeserializationError(e.to_string()))
}

pub fn encode_glyph(g: Glyph) -> u8 { g.to_byte() }

pub fn decode_glyph(byte: u8) -> Result<Glyph, super::Error> {
    Glyph::from_byte(byte).ok_or(super::Error::InvalidGlyph)
}

pub fn encode_glyphs(glyphs: &[Glyph]) -> Vec<u8> {
    glyphs.iter().map(|g| g.to_byte()).collect()
}

pub fn decode_glyphs(bytes: &[u8]) -> Result<Vec<Glyph>, super::Error> {
    bytes.iter().map(|&b| decode_glyph(b)).collect()
}

pub fn validate_glyphs(glyphs: &[Glyph]) -> bool {
    glyphs.iter().all(|g| matches!(g, Glyph::Pi | Glyph::Gamma | Glyph::Delta | Glyph::Omega | Glyph::Lambda | Glyph::Psi))
}

pub fn is_absorber(g: Glyph) -> bool { matches!(g, Glyph::Omega) }
pub fn is_identity(g: Glyph) -> bool { matches!(g, Glyph::Lambda) }
