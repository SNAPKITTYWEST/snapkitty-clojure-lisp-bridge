use hyperkitty_core::{Error, Glyph, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vec3 { pub x: i64, pub y: i64, pub z: i64 }

impl Vec3 {
    pub const fn new(x: i64, y: i64, z: i64) -> Self { Self { x, y, z } }
    pub fn norm_sq(&self) -> i64 { self.x*self.x + self.y*self.y + self.z*self.z }
    pub fn is_canonical(&self) -> bool { self.norm_sq() == K_QLG }
    pub fn neg(&self) -> Self { Self::new(-self.x, -self.y, -self.z) }
    pub fn dot(&self, o: &Self) -> i64 { self.x*o.x + self.y*o.y + self.z*o.z }
}

pub const K_QLG: i64 = 1;

pub fn vec3_from_glyph(g: Glyph) -> Vec3 {
    match g {
        Glyph::Pi => Vec3::new(1,0,0), Glyph::Gamma => Vec3::new(-1,0,0),
        Glyph::Delta => Vec3::new(0,1,0), Glyph::Psi => Vec3::new(0,-1,0),
        Glyph::Lambda => Vec3::new(0,0,1), Glyph::Omega => Vec3::new(0,0,-1),
    }
}

pub fn glyph_from_vec3(v: &Vec3) -> Option<Glyph> {
    match (v.x,v.y,v.z) {
        (1,0,0) => Some(Glyph::Pi), (-1,0,0) => Some(Glyph::Gamma),
        (0,1,0) => Some(Glyph::Delta), (0,-1,0) => Some(Glyph::Psi),
        (0,0,1) => Some(Glyph::Lambda), (0,0,-1) => Some(Glyph::Omega),
        _ => None,
    }
}

pub fn canonical_points() -> [Vec3; 6] {
    [Vec3::new(1,0,0), Vec3::new(-1,0,0), Vec3::new(0,1,0),
     Vec3::new(0,-1,0), Vec3::new(0,0,1), Vec3::new(0,0,-1)]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QLGCertificate { pub point: Vec3, pub norm_sq: i64, pub is_valid: bool }

impl QLGCertificate {
    pub fn new(p: Vec3) -> Self {
        let ns = p.norm_sq(); Self { point: p, norm_sq: ns, is_valid: ns == K_QLG }
    }
    pub fn validate(&self) -> bool { self.norm_sq == K_QLG && self.is_valid }
}
