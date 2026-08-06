//! Expert activation gating (Stage 07)

#[derive(Debug, Clone)]
pub struct Expert {
    pub name: String,
    pub weight: f64,
}

pub struct ActivationSet {
    experts: Vec<Expert>,
}

impl ActivationSet {
    pub fn new() -> Self {
        ActivationSet {
            experts: Vec::new(),
        }
    }

    pub fn activate(&mut self, expert: Expert) {
        if expert.weight > 0.0 {
            self.experts.push(expert);
        }
    }

    pub fn len(&self) -> usize {
        self.experts.len()
    }

    pub fn experts(&self) -> &[Expert] {
        &self.experts
    }
}

impl Default for ActivationSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activation_set() {
        let mut set = ActivationSet::new();
        set.activate(Expert {
            name: "expert1".to_string(),
            weight: 0.5,
        });
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn reject_zero_weight() {
        let mut set = ActivationSet::new();
        set.activate(Expert {
            name: "expert0".to_string(),
            weight: 0.0,
        });
        assert_eq!(set.len(), 0);
    }
}
