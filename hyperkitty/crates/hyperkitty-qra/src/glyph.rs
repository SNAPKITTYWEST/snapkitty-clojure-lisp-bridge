//! QRA Glyphs: Six canonical routing tokens

use crate::Result;
use hyperkitty_core::Error;

/// Six canonical glyphs with wire encodings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Glyph {
    Pi = 0,      // 0x01
    Gamma = 1,   // 0x03
    Delta = 2,   // 0x04
    Omega = 3,   // 0x0A
    Lambda = 4,  // 0xFF
    Psi = 5,     // 0x0B
}

impl Glyph {
    /// Get wire encoding byte
    pub fn to_byte(&self) -> u8 {
        match self {
            Glyph::Pi => 0x01,
            Glyph::Gamma => 0x03,
            Glyph::Delta => 0x04,
            Glyph::Omega => 0x0A,
            Glyph::Lambda => 0xFF,
            Glyph::Psi => 0x0B,
        }
    }

    /// Parse from wire encoding
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            0x01 => Ok(Glyph::Pi),
            0x03 => Ok(Glyph::Gamma),
            0x04 => Ok(Glyph::Delta),
            0x0A => Ok(Glyph::Omega),
            0xFF => Ok(Glyph::Lambda),
            0x0B => Ok(Glyph::Psi),
            _ => Err(Error::QRATransition(format!("Invalid glyph byte: 0x{:02x}", b))),
        }
    }

    /// Get index (0-5) for tensor lookup
    pub fn index(&self) -> usize {
        *self as usize
    }
}

/// All six glyphs in order
pub fn all_glyphs() -> [Glyph; 6] {
    [
        Glyph::Pi,
        Glyph::Gamma,
        Glyph::Delta,
        Glyph::Omega,
        Glyph::Lambda,
        Glyph::Psi,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_round_trip() {
        for glyph in all_glyphs() {
            let byte = glyph.to_byte();
            let recovered = Glyph::from_byte(byte).unwrap();
            assert_eq!(glyph, recovered);
        }
    }

    #[test]
    fn index_range() {
        for (i, glyph) in all_glyphs().iter().enumerate() {
            assert_eq!(glyph.index(), i);
        }
    }

    #[test]
    fn invalid_byte() {
        assert!(Glyph::from_byte(0x99).is_err());
    }

    #[test]
    fn distinct_encodings() {
        let glyphs = all_glyphs();
        let mut bytes = vec![];
        for g in glyphs {
            bytes.push(g.to_byte());
        }
        // All distinct
        bytes.sort();
        bytes.dedup();
        assert_eq!(bytes.len(), 6);
    }
}
