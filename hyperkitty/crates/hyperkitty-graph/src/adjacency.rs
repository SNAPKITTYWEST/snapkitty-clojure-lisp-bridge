//! Adjacency matrix from AST edges

pub struct AdjacencyMatrix {
    size: usize,
    matrix: Vec<Vec<f64>>,
}

impl AdjacencyMatrix {
    pub fn new(size: usize) -> Self {
        AdjacencyMatrix {
            size,
            matrix: vec![vec![0.0; size]; size],
        }
    }

    pub fn set_edge(&mut self, from: usize, to: usize, weight: f64) {
        if from < self.size && to < self.size {
            self.matrix[from][to] = weight;
        }
    }

    pub fn get_edge(&self, from: usize, to: usize) -> f64 {
        if from < self.size && to < self.size {
            self.matrix[from][to]
        } else {
            0.0
        }
    }

    pub fn matrix(&self) -> &[Vec<f64>] {
        &self.matrix
    }

    pub fn compute_out_degree(&self, node: usize) -> f64 {
        if node >= self.size {
            return 0.0;
        }
        self.matrix[node].iter().sum()
    }

    pub fn compute_in_degree(&self, node: usize) -> f64 {
        if node >= self.size {
            return 0.0;
        }
        self.matrix.iter().map(|row| row[node]).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjacency_creation() {
        let adj = AdjacencyMatrix::new(3);
        assert_eq!(adj.get_edge(0, 1), 0.0);
    }

    #[test]
    fn set_get_edge() {
        let mut adj = AdjacencyMatrix::new(3);
        adj.set_edge(0, 1, 0.5);
        assert_eq!(adj.get_edge(0, 1), 0.5);
    }

    #[test]
    fn degree_computation() {
        let mut adj = AdjacencyMatrix::new(3);
        adj.set_edge(0, 1, 0.5);
        adj.set_edge(0, 2, 0.3);
        assert_eq!(adj.compute_out_degree(0), 0.8);
    }
}
