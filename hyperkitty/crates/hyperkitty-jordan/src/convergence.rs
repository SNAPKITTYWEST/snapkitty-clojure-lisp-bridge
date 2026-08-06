//! Convergence traces for Jordan density evolution
use crate::density::DensityMatrix;

#[derive(Debug, Clone)]
pub struct ConvergenceTrace {
    pub iterations: Vec<DensityMatrix>,
    pub converged: bool,
    pub final_distance: f64,
}

impl ConvergenceTrace {
    pub fn len(&self) -> usize {
        self.iterations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.iterations.is_empty()
    }

    pub fn final_state(&self) -> Option<&DensityMatrix> {
        self.iterations.last()
    }
}

pub fn converge(
    rho_init: DensityMatrix,
    u: &DensityMatrix,
    max_iters: usize,
    tolerance: f64,
) -> ConvergenceTrace {
    let mut iterations = vec![rho_init.clone()];
    let mut rho = rho_init;
    let mut converged = false;
    let mut final_distance = f64::INFINITY;

    for _ in 0..max_iters {
        let rho_next = rho.evolve(u);
        let distance = rho.distance_to(&rho_next);
        iterations.push(rho_next.clone());

        if distance < tolerance {
            converged = true;
            final_distance = distance;
            break;
        }

        rho = rho_next;
        final_distance = distance;
    }

    ConvergenceTrace {
        iterations,
        converged,
        final_distance,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convergence_basic() {
        let rho = DensityMatrix::identity(2);
        let u = DensityMatrix::identity(2);
        let _trace = converge(rho, &u, 100, 1e-10);
    }
}
