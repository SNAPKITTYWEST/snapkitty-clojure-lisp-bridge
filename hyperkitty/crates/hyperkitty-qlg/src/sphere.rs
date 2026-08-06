//! QLG Sphere: Canonical six-point integer geometry
//!
//! S_can = {x ∈ Z³ : x₀² + x₁² + x₂² = 1}
//! Solutions: {(±1,0,0), (0,±1,0), (0,0,±1)}

use crate::Point;

/// Verify membership in canonical sphere.
pub fn is_on_sphere(p: &Point) -> bool {
    p.0 * p.0 + p.1 * p.1 + p.2 * p.2 == 1
}

/// All six canonical points
pub fn canonical_points() -> [Point; 6] {
    [
        Point(1, 0, 0),
        Point(-1, 0, 0),
        Point(0, 1, 0),
        Point(0, -1, 0),
        Point(0, 0, 1),
        Point(0, 0, -1),
    ]
}

/// Encode point to 3-byte array (little-endian signed integers)
pub fn encode(p: &Point) -> [u8; 3] {
    [
        (p.0 as i8) as u8,
        (p.1 as i8) as u8,
        (p.2 as i8) as u8,
    ]
}

/// Decode point from 3-byte array
pub fn decode(bytes: &[u8; 3]) -> Point {
    Point(
        bytes[0] as i8 as i32,
        bytes[1] as i8 as i32,
        bytes[2] as i8 as i32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_on_sphere() {
        for p in canonical_points() {
            assert!(is_on_sphere(&p));
        }
    }

    #[test]
    fn six_solutions() {
        assert_eq!(canonical_points().len(), 6);
    }

    #[test]
    fn round_trip() {
        for p in canonical_points() {
            let encoded = encode(&p);
            let decoded = decode(&encoded);
            assert_eq!(p, decoded);
        }
    }

    #[test]
    fn invalid_rejected() {
        assert!(!is_on_sphere(&Point(1, 1, 0)));
        assert!(!is_on_sphere(&Point(2, 0, 0)));
        assert!(!is_on_sphere(&Point(0, 0, 0)));
    }
}
