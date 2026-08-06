use std::collections::HashMap;

/// Dispatcher orchestrates execution of routing nodes
pub struct Dispatcher {
    execution_count: usize,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            execution_count: 0,
        }
    }

    /// Dispatch a single node execution
    pub fn dispatch(&mut self, node_id: u64, message: &str) -> hyperkitty_core::Result<String> {
        if message.is_empty() {
            return Err(hyperkitty_core::Error::ParseError("empty_message".to_string()));
        }

        self.execution_count += 1;

        Ok(format!(
            "dispatch_{}(node: {}, msg: '{}', exec: {})",
            self.execution_count, node_id, message, self.execution_count
        ))
    }

    /// Batch dispatch multiple nodes
    pub fn dispatch_batch(
        &mut self,
        nodes: Vec<crate::nodes::RoutingNode>,
        message: &str,
    ) -> hyperkitty_core::Result<HashMap<u64, String>> {
        if nodes.is_empty() {
            return Err(hyperkitty_core::Error::NoValidRoutes);
        }

        if message.is_empty() {
            return Err(hyperkitty_core::Error::ParseError("empty_message".to_string()));
        }

        let mut results = HashMap::new();

        for node in nodes {
            if node.is_valid() {
                self.execution_count += 1;
                let result = format!(
                    "batch_exec_{}(node: {}, name: {}, weight: {:.4})",
                    self.execution_count, node.id, node.name, node.weight
                );
                results.insert(node.id, result);
            }
        }

        if results.is_empty() {
            return Err(hyperkitty_core::Error::NoValidRoutes);
        }

        Ok(results)
    }

    /// Get the number of dispatches executed
    pub fn execution_count(&self) -> usize {
        self.execution_count
    }

    /// Reset execution counter
    pub fn reset(&mut self) {
        self.execution_count = 0;
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatcher_creation() {
        let dispatcher = Dispatcher::new();
        assert_eq!(dispatcher.execution_count(), 0);
    }

    #[test]
    fn test_dispatcher_single_dispatch() {
        let mut dispatcher = Dispatcher::new();
        let result = dispatcher.dispatch(42, "test message");

        assert!(result.is_ok());
        assert_eq!(dispatcher.execution_count(), 1);
    }

    #[test]
    fn test_dispatcher_dispatch_rejects_empty_message() {
        let mut dispatcher = Dispatcher::new();
        let result = dispatcher.dispatch(1, "");

        assert!(result.is_err());
        assert_eq!(dispatcher.execution_count(), 0);
    }

    #[test]
    fn test_dispatcher_increments_count() {
        let mut dispatcher = Dispatcher::new();
        let _ = dispatcher.dispatch(1, "msg1");
        let _ = dispatcher.dispatch(2, "msg2");
        let _ = dispatcher.dispatch(3, "msg3");

        assert_eq!(dispatcher.execution_count(), 3);
    }

    #[test]
    fn test_dispatcher_batch_dispatch_empty_nodes() {
        let mut dispatcher = Dispatcher::new();
        let result = dispatcher.dispatch_batch(vec![], "test");

        assert!(result.is_err());
    }

    #[test]
    fn test_dispatcher_batch_dispatch_empty_message() {
        let mut dispatcher = Dispatcher::new();
        let nodes = vec![crate::nodes::RoutingNode::new(1, "node".to_string(), 0.5)];

        let result = dispatcher.dispatch_batch(nodes, "");
        assert!(result.is_err());
    }

    #[test]
    fn test_dispatcher_batch_dispatch_single_node() {
        let mut dispatcher = Dispatcher::new();
        let nodes = vec![crate::nodes::RoutingNode::new(100, "node1".to_string(), 0.5)];

        let result = dispatcher.dispatch_batch(nodes, "message");
        assert!(result.is_ok());

        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results.contains_key(&100));
    }

    #[test]
    fn test_dispatcher_batch_dispatch_multiple_nodes() {
        let mut dispatcher = Dispatcher::new();
        let nodes = vec![
            crate::nodes::RoutingNode::new(1, "n1".to_string(), 0.5),
            crate::nodes::RoutingNode::new(2, "n2".to_string(), 0.6),
            crate::nodes::RoutingNode::new(3, "n3".to_string(), 0.7),
        ];

        let result = dispatcher.dispatch_batch(nodes, "test");
        assert!(result.is_ok());

        let results = result.unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_dispatcher_batch_dispatch_skips_invalid() {
        let mut dispatcher = Dispatcher::new();
        let nodes = vec![
            crate::nodes::RoutingNode::new(1, "valid".to_string(), 0.5),
            crate::nodes::RoutingNode::new(2, "invalid".to_string(), 0.0),
        ];

        let result = dispatcher.dispatch_batch(nodes, "test");
        assert!(result.is_ok());

        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results.contains_key(&1));
    }

    #[test]
    fn test_dispatcher_reset() {
        let mut dispatcher = Dispatcher::new();
        let _ = dispatcher.dispatch(1, "msg");
        assert_eq!(dispatcher.execution_count(), 1);

        dispatcher.reset();
        assert_eq!(dispatcher.execution_count(), 0);
    }

    #[test]
    fn test_dispatcher_execution_count_increments_in_batch() {
        let mut dispatcher = Dispatcher::new();
        let nodes = vec![
            crate::nodes::RoutingNode::new(1, "n1".to_string(), 0.5),
            crate::nodes::RoutingNode::new(2, "n2".to_string(), 0.6),
        ];

        let _ = dispatcher.dispatch_batch(nodes, "test");
        assert_eq!(dispatcher.execution_count(), 2);
    }

    #[test]
    fn test_dispatcher_dispatch_includes_node_id() {
        let mut dispatcher = Dispatcher::new();
        let result = dispatcher.dispatch(999, "msg");

        let output = result.unwrap();
        assert!(output.contains("999"));
    }

    #[test]
    fn test_dispatcher_batch_includes_weights() {
        let mut dispatcher = Dispatcher::new();
        let nodes = vec![crate::nodes::RoutingNode::new(1, "node".to_string(), 0.7777)];

        let result = dispatcher.dispatch_batch(nodes, "test");
        let results = result.unwrap();

        let output = results.get(&1).unwrap();
        assert!(output.contains("0.7777"));
    }

    #[test]
    fn test_dispatcher_default_trait() {
        let dispatcher = Dispatcher::default();
        assert_eq!(dispatcher.execution_count(), 0);
    }
}
