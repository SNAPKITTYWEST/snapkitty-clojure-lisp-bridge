//! Dead path detection: routing paths with zero activation

pub fn find_dead_paths(matrix: &[Vec<f64>]) -> Vec<usize> {
    let mut dead = Vec::new();
    for (i, row) in matrix.iter().enumerate() {
        let total_out: f64 = row.iter().sum();
        if total_out < 1e-10 {
            dead.push(i);
        }
    }
    dead
}

pub fn find_unreachable_nodes(matrix: &[Vec<f64>]) -> Vec<usize> {
    let mut unreachable = Vec::new();
    for (i, _) in matrix.iter().enumerate() {
        let mut reachable = false;
        for row in matrix {
            if row[i] > 1e-10 {
                reachable = true;
                break;
            }
        }
        if !reachable {
            unreachable.push(i);
        }
    }
    unreachable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_dead_path() {
        let matrix = vec![vec![0.0, 0.0], vec![1.0, 1.0]];
        let dead = find_dead_paths(&matrix);
        assert_eq!(dead, vec![0]);
    }

    #[test]
    fn find_unreachable() {
        let matrix = vec![vec![0.0, 1.0], vec![0.0, 0.0]];
        let unreachable = find_unreachable_nodes(&matrix);
        assert!(unreachable.contains(&0));
    }
}
