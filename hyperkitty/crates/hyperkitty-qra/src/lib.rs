use hyperkitty_core::Glyph;

// Q[curr][prev]: paper Table 1, Ahmad Parr Zenodo 2026
// Index: Pi=0 Gamma=1 Delta=2 Omega=3 Lambda=4 Psi=5
//         prev: Pi  Ga  De  Om  La  Ps
pub const Q: [[usize; 6]; 6] = [
    /* Pi(0) */  [2,  2,  3,  3,  2,  2],
    /* Ga(1) */  [2,  3,  3,  3,  2,  3],
    /* De(2) */  [3,  3,  3,  3,  2,  3],
    /* Om(3) */  [3,  3,  3,  3,  3,  3],
    /* La(4) */  [0,  1,  2,  3,  4,  5],
    /* Ps(5) */  [2,  3,  3,  3,  2,  3],
];

pub fn next_glyph(curr: Glyph, prev: Glyph) -> Glyph {
    let c = curr.index(); let p = prev.index();
    Glyph::by_index(Q[c][p]).unwrap()
}

pub fn is_absorber(g: Glyph) -> bool { matches!(g, Glyph::Omega) }
pub fn is_identity(g: Glyph) -> bool { matches!(g, Glyph::Lambda) }

pub fn validate_identity_row() -> bool {
    let l = Glyph::Lambda.index();
    (0..6).all(|j| Q[l][j] == j)
}

pub fn validate_absorber_row() -> bool {
    let o = Glyph::Omega.index();
    (0..6).all(|j| Q[o][j] == o)
}

pub fn evolve_witness(w: &[Glyph]) -> Vec<Glyph> {
    if w.len() != 3 { return vec![]; }
    vec![next_glyph(w[0],w[1]), next_glyph(w[1],w[2]), next_glyph(w[2],w[0])]
}

pub fn canonical_witness() -> Vec<Glyph> {
    vec![Glyph::Pi, Glyph::Gamma, Glyph::Delta]
}

pub fn validate_canonical_exhaustion() -> bool {
    let w0 = canonical_witness();
    let w1 = evolve_witness(&w0);
    let w2 = evolve_witness(&w1);
    w2.iter().all(|&g| is_absorber(g))
}
