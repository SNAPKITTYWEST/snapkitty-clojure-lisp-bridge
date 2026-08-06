//! B₃ Root System and exceptional algebra correspondence

use crate::Point;

/// Map canonical point to B₃ root index (0-5)
pub fn to_root_index(p: &Point) -> usize {
    match *p {
        Point(1, 0, 0) => 0,
        Point(-1, 0, 0) => 1,
        Point(0, 1, 0) => 2,
        Point(0, -1, 0) => 3,
        Point(0, 0, 1) => 4,
        Point(0, 0, -1) => 5,
        _ => panic!("Point not in B₃ root system"),
    }
}

/// Map B₃ root index back to canonical point
pub fn from_root_index(idx: usize) -> Point {
    match idx {
        0 => Point(1, 0, 0),
        1 => Point(-1, 0, 0),
        2 => Point(0, 1, 0),
        3 => Point(0, -1, 0),
        4 => Point(0, 0, 1),
        5 => Point(0, 0, -1),
        _ => panic!("Root index out of range"),
    }
}

/// Compute dot product of two B₃ roots
pub fn dot_product(p1: &Point, p2: &Point) -> i32 {
    p1.0 * p2.0 + p1.1 * p2.1 + p1.2 * p2.2
}

/// Check if two roots are orthogonal
pub fn are_orthogonal(p1: &Point, p2: &Point) -> bool {
    dot_product(p1, p2) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_round_trip() {
        for i in 0..6 {
            let p = from_root_index(i);
            assert_eq!(to_root_index(&p), i);
        }
    }

    #[test]
    fn orthogonality() {
        let p1 = Point(1, 0, 0);
        let p2 = Point(0, 1, 0);
        assert!(are_orthogonal(&p1, &p2));
    }

    #[test]
    fn antiparallel_not_orthogonal() {
        let p1 = Point(1, 0, 0);
        let p2 = Point(-1, 0, 0);
        assert!(!are_orthogonal(&p1, &p2));
        assert_eq!(dot_product(&p1, &p2), -1);
    }
}
