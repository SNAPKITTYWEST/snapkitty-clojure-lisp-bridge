//! Topology selection for Jordan spectral decomposition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Topology {
    LeftFold,
    RightFold,
    BalancedTree,
    Star,
}

impl Topology {
    pub fn all() -> [Self; 4] {
        [Self::LeftFold, Self::RightFold, Self::BalancedTree, Self::Star]
    }

    pub fn spectral_gap_estimate(&self, n: usize) -> f64 {
        match self {
            Self::LeftFold => (std::f64::consts::PI / (n as f64 + 1.0)).sin() * 2.0,
            Self::RightFold => (std::f64::consts::PI / (n as f64 + 1.0)).sin() * 2.0,
            Self::BalancedTree => 1.5,
            Self::Star => 2.0,
        }
    }
}

pub fn find_best_topology(n: usize) -> Topology {
    Topology::all()
        .iter()
        .copied()
        .max_by(|a, b| {
            a.spectral_gap_estimate(n)
                .partial_cmp(&b.spectral_gap_estimate(n))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn star_maximizes() {
        assert_eq!(find_best_topology(4), Topology::Star);
    }
}
