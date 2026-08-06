/// A routing node represents an executable expert in the routing pipeline
#[derive(Clone, Debug)]
pub struct RoutingNode {
    pub id: u64,
    pub name: String,
    pub weight: f64,
    pub enabled: bool,
}

impl RoutingNode {
    /// Create a new routing node with id, name, and weight
    pub fn new(id: u64, name: String, weight: f64) -> Self {
        Self {
            id,
            name,
            weight,
            enabled: weight > 0.0,
        }
    }

    /// Execute this routing node on the given input
    pub fn execute(&self, input: &str) -> hyperkitty_core::Result<String> {
        if !self.enabled {
            return Err(hyperkitty_core::Error::InvalidRoute);
        }

        // Apply weighted transformation
        let weighted_input = format!(
            "{}(weight: {:.4}, input: '{}' )",
            self.name, self.weight, input
        );

        Ok(weighted_input)
    }

    /// Check if this node is valid for routing
    pub fn is_valid(&self) -> bool {
        self.enabled && self.weight > 0.0 && !self.name.is_empty()
    }

    /// Set the enabled state
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routing_node_creation() {
        let node = RoutingNode::new(42, "test_node".to_string(), 0.75);
        assert_eq!(node.id, 42);
        assert_eq!(node.name, "test_node");
        assert_eq!(node.weight, 0.75);
        assert!(node.enabled);
    }

    #[test]
    fn test_routing_node_disabled_when_weight_zero() {
        let node = RoutingNode::new(1, "node".to_string(), 0.0);
        assert!(!node.enabled);
    }

    #[test]
    fn test_routing_node_execute_produces_output() {
        let node = RoutingNode::new(100, "router".to_string(), 0.5);
        let result = node.execute("test input");

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("router"));
        assert!(output.contains("0.5"));
    }

    #[test]
    fn test_routing_node_execute_rejects_disabled() {
        let mut node = RoutingNode::new(1, "node".to_string(), 0.5);
        node.set_enabled(false);

        let result = node.execute("input");
        assert!(result.is_err());
    }

    #[test]
    fn test_routing_node_execute_preserves_input() {
        let node = RoutingNode::new(1, "node".to_string(), 1.0);
        let result = node.execute("hello world");

        assert!(result.is_ok());
        assert!(result.unwrap().contains("hello world"));
    }

    #[test]
    fn test_routing_node_is_valid_enabled() {
        let node = RoutingNode::new(1, "valid".to_string(), 0.8);
        assert!(node.is_valid());
    }

    #[test]
    fn test_routing_node_is_valid_zero_weight() {
        let node = RoutingNode::new(1, "invalid".to_string(), 0.0);
        assert!(!node.is_valid());
    }

    #[test]
    fn test_routing_node_is_valid_empty_name() {
        let node = RoutingNode::new(1, String::new(), 0.5);
        assert!(!node.is_valid());
    }

    #[test]
    fn test_routing_node_set_enabled() {
        let mut node = RoutingNode::new(1, "node".to_string(), 0.5);
        assert!(node.enabled);

        node.set_enabled(false);
        assert!(!node.enabled);

        node.set_enabled(true);
        assert!(node.enabled);
    }

    #[test]
    fn test_routing_node_high_weight() {
        let node = RoutingNode::new(1, "high_weight".to_string(), 0.99);
        assert!(node.is_valid());
        assert!(node.weight > 0.9);
    }

    #[test]
    fn test_routing_node_low_weight() {
        let node = RoutingNode::new(1, "low_weight".to_string(), 0.01);
        assert!(node.is_valid());
    }

    #[test]
    fn test_routing_node_cloneable() {
        let node = RoutingNode::new(42, "original".to_string(), 0.5);
        let cloned = node.clone();

        assert_eq!(cloned.id, node.id);
        assert_eq!(cloned.name, node.name);
        assert_eq!(cloned.weight, node.weight);
    }

    #[test]
    fn test_routing_node_execute_weight_precision() {
        let node = RoutingNode::new(1, "node".to_string(), 0.3333);
        let result = node.execute("test");

        let output = result.unwrap();
        assert!(output.contains("0.3333"));
    }
}
