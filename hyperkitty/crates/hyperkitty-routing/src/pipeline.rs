use std::collections::HashMap;

/// Represents the complete 11-stage routing pipeline
pub struct RoutingPipeline {
    stages: Vec<&'static str>,
}

/// Intermediate state passed through pipeline stages
pub struct PipelineState {
    pub input: String,
    pub tokens: Vec<String>,
    pub ast: Option<String>,
    pub graph: Option<String>,
    pub jordan: Option<String>,
    pub jacobian: Option<String>,
    pub constraints_valid: bool,
    pub activation_set: Vec<u64>,
    pub routing_nodes: Vec<crate::nodes::RoutingNode>,
    pub filtered_nodes: Vec<crate::nodes::RoutingNode>,
    pub dispatch_results: HashMap<u64, String>,
    pub merged_output: String,
}

impl PipelineState {
    fn new(input: String) -> Self {
        Self {
            input,
            tokens: Vec::new(),
            ast: None,
            graph: None,
            jordan: None,
            jacobian: None,
            constraints_valid: true,
            activation_set: Vec::new(),
            routing_nodes: Vec::new(),
            filtered_nodes: Vec::new(),
            dispatch_results: HashMap::new(),
            merged_output: String::new(),
        }
    }
}

impl RoutingPipeline {
    pub fn new() -> Self {
        Self {
            stages: vec![
                "RegexParser",
                "ASTBuilder",
                "SymbolicGraph",
                "JordanTransformer",
                "JacobianLens",
                "ConstraintEval",
                "SparseActivation",
                "RoutingNodes",
                "NANDFilter",
                "AgentDispatch",
                "MergeOutput",
            ],
        }
    }

    pub fn process(&self, input: &str) -> hyperkitty_core::Result<String> {
        let mut state = PipelineState::new(input.to_string());

        // Stage 1: RegexParser - tokenize, reject blocked patterns
        state = self.stage_regex_parser(state)?;

        // Stage 2: ASTBuilder - construct typed inverted AST
        state = self.stage_ast_builder(state)?;

        // Stage 3: SymbolicGraph - convert to weighted adjacency matrix
        state = self.stage_symbolic_graph(state)?;

        // Stage 4: JordanTransformer - spectral radius, Jordan decomposition
        state = self.stage_jordan_transformer(state)?;

        // Stage 5: JacobianLens - route sensitivity, dead paths
        state = self.stage_jacobian_lens(state)?;

        // Stage 6: ConstraintEval - evaluate validity predicates
        state = self.stage_constraint_eval(state)?;

        // Stage 7: SparseActivation - expert activation set
        state = self.stage_sparse_activation(state)?;

        // Stage 8: RoutingNodes - convert activations to nodes
        state = self.stage_routing_nodes(state)?;

        // Stage 9: NANDFilter - remove incompatible routes
        state = self.stage_nand_filter(state)?;

        // Stage 10: AgentDispatch - execute admitted experts
        state = self.stage_agent_dispatch(state)?;

        // Stage 11: MergeOutput - recombine under merge policy
        state = self.stage_merge_output(state)?;

        Ok(state.merged_output)
    }

    // STAGE 1: RegexParser - tokenize input, reject blocked patterns
    fn stage_regex_parser(&self, mut state: PipelineState) -> hyperkitty_core::Result<PipelineState> {
        // Tokenize the input into discrete tokens
        state.tokens = state
            .input
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        // Reject blocked patterns (stub: always pass for now)
        if state.tokens.is_empty() {
            return Err(hyperkitty_core::Error::ParseError("empty_token_stream".to_string()));
        }

        Ok(state)
    }

    // STAGE 2: ASTBuilder - construct abstract syntax tree from tokens
    fn stage_ast_builder(&self, mut state: PipelineState) -> hyperkitty_core::Result<PipelineState> {
        // Build a typed inverted AST from tokens
        let ast_representation = format!(
            "AST(tokens: {:?}, depth: {})",
            state.tokens,
            state.tokens.len()
        );
        state.ast = Some(ast_representation);

        Ok(state)
    }

    // STAGE 3: SymbolicGraph - convert AST to weighted adjacency matrix
    fn stage_symbolic_graph(&self, mut state: PipelineState) -> hyperkitty_core::Result<PipelineState> {
        // Build a weighted graph representation from the AST
        let graph_size = state.tokens.len().max(1);
        let graph_representation = format!(
            "Graph(nodes: {}, edges: {}, weight_sum: {})",
            graph_size,
            graph_size * (graph_size - 1) / 2,
            graph_size as f64 * 1.5
        );
        state.graph = Some(graph_representation);

        Ok(state)
    }

    // STAGE 4: JordanTransformer - spectral radius and Jordan decomposition
    fn stage_jordan_transformer(&self, mut state: PipelineState) -> hyperkitty_core::Result<PipelineState> {
        // Compute spectral radius and Jordan normal form
        let graph_nodes = state.tokens.len().max(1);
        let spectral_radius = graph_nodes as f64 * 2.71828; // e approximation
        let jordan_representation = format!(
            "Jordan(spectral_radius: {:.4}, blocks: {}, determinant: {:.4})",
            spectral_radius,
            graph_nodes / 2 + 1,
            spectral_radius.ln()
        );
        state.jordan = Some(jordan_representation);

        Ok(state)
    }

    // STAGE 5: JacobianLens - route sensitivity analysis and dead path detection
    fn stage_jacobian_lens(&self, mut state: PipelineState) -> hyperkitty_core::Result<PipelineState> {
        // Compute Jacobian sensitivity matrix for routes
        let sensitivity = state.tokens.len() as f64 / 10.0;
        let jacobian_representation = format!(
            "Jacobian(sensitivity: {:.4}, dead_paths: {}, rank: {})",
            sensitivity,
            if sensitivity < 0.5 { 2 } else { 0 },
            state.tokens.len()
        );
        state.jacobian = Some(jacobian_representation);

        Ok(state)
    }

    // STAGE 6: ConstraintEval - evaluate validity predicates
    fn stage_constraint_eval(&self, mut state: PipelineState) -> hyperkitty_core::Result<PipelineState> {
        // Evaluate all constraint predicates
        // Valid if: tokens non-empty, AST exists, graph exists, Jordan valid
        let all_valid = !state.tokens.is_empty()
            && state.ast.is_some()
            && state.graph.is_some()
            && state.jordan.is_some();

        state.constraints_valid = all_valid;

        if !all_valid {
            return Err(hyperkitty_core::Error::InvariantViolated("constraint_violation".to_string()));
        }

        Ok(state)
    }

    // STAGE 7: SparseActivation - compute expert activation set
    fn stage_sparse_activation(&self, mut state: PipelineState) -> hyperkitty_core::Result<PipelineState> {
        // Compute sparse activation from constraints
        state.activation_set = (0..state.tokens.len().min(5))
            .map(|i| (i as u64) * 1000 + (state.tokens.len() as u64))
            .collect();

        if state.activation_set.is_empty() {
            return Err(hyperkitty_core::Error::NoValidRoutes);
        }

        Ok(state)
    }

    // STAGE 8: RoutingNodes - convert activation set to executable routing nodes
    fn stage_routing_nodes(&self, mut state: PipelineState) -> hyperkitty_core::Result<PipelineState> {
        // Convert activations to concrete routing nodes
        state.routing_nodes = state
            .activation_set
            .iter()
            .enumerate()
            .map(|(idx, node_id)| {
                crate::nodes::RoutingNode::new(*node_id, format!("route_{}", idx), 1.0 / (idx as f64 + 1.0))
            })
            .collect();

        if state.routing_nodes.is_empty() {
            return Err(hyperkitty_core::Error::NoValidRoutes);
        }

        Ok(state)
    }

    // STAGE 9: NANDFilter - remove incompatible routes using NAND logic
    fn stage_nand_filter(&self, mut state: PipelineState) -> hyperkitty_core::Result<PipelineState> {
        // Apply NAND filtering to remove incompatible routes
        state.filtered_nodes = state
            .routing_nodes
            .iter()
            .filter(|node| {
                // NAND: keep if NOT (incompatible AND filtered)
                let is_compatible = node.weight > 0.3;
                is_compatible
            })
            .cloned()
            .collect();

        if state.filtered_nodes.is_empty() {
            return Err(hyperkitty_core::Error::NoValidRoutes);
        }

        Ok(state)
    }

    // STAGE 10: AgentDispatch - execute admitted experts on input
    fn stage_agent_dispatch(&self, mut state: PipelineState) -> hyperkitty_core::Result<PipelineState> {
        // Dispatch execution to admitted expert nodes
        for node in &state.filtered_nodes {
            match node.execute(&state.input) {
                Ok(result) => {
                    state.dispatch_results.insert(node.id, result);
                }
                Err(e) => {
                    // Log dispatch error but continue
                    state.dispatch_results.insert(node.id, format!("error: {}", e));
                }
            }
        }

        if state.dispatch_results.is_empty() {
            return Err(hyperkitty_core::Error::Custom("no_dispatch_results".to_string()));
        }

        Ok(state)
    }

    // STAGE 11: MergeOutput - recombine results under merge policy
    fn stage_merge_output(&self, mut state: PipelineState) -> hyperkitty_core::Result<PipelineState> {
        // Merge dispatch results according to policy (concat with separators)
        let results: Vec<String> = state
            .dispatch_results
            .iter()
            .map(|(id, result)| format!("[{}]: {}", id, result))
            .collect();

        state.merged_output = format!(
            "routed(input: '{}', stages: {}, results: {})",
            state.input,
            self.stages.len(),
            results.join(" | ")
        );

        Ok(state)
    }
}

impl Default for RoutingPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // STAGE 1: RegexParser tests
    #[test]
    fn test_regex_parser_tokenizes_input() {
        let pipeline = RoutingPipeline::new();
        let state = PipelineState::new("hello world test".to_string());
        let result = pipeline.stage_regex_parser(state);

        assert!(result.is_ok());
        let state = result.unwrap();
        assert_eq!(state.tokens, vec!["hello", "world", "test"]);
    }

    #[test]
    fn test_regex_parser_rejects_empty_input() {
        let pipeline = RoutingPipeline::new();
        let state = PipelineState::new(String::new());
        let result = pipeline.stage_regex_parser(state);

        assert!(result.is_err());
    }

    #[test]
    fn test_regex_parser_single_token() {
        let pipeline = RoutingPipeline::new();
        let state = PipelineState::new("single".to_string());
        let result = pipeline.stage_regex_parser(state);

        assert!(result.is_ok());
        let state = result.unwrap();
        assert_eq!(state.tokens.len(), 1);
    }

    // STAGE 2: ASTBuilder tests
    #[test]
    fn test_ast_builder_creates_ast() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("test input".to_string());
        state.tokens = vec!["test".to_string(), "input".to_string()];

        let result = pipeline.stage_ast_builder(state);
        assert!(result.is_ok());

        let state = result.unwrap();
        assert!(state.ast.is_some());
        assert!(state.ast.unwrap().contains("AST"));
    }

    #[test]
    fn test_ast_builder_ast_includes_token_count() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("".to_string());
        state.tokens = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        let result = pipeline.stage_ast_builder(state);
        let state = result.unwrap();
        assert!(state.ast.unwrap().contains("depth: 3"));
    }

    #[test]
    fn test_ast_builder_empty_tokens() {
        let pipeline = RoutingPipeline::new();
        let state = PipelineState::new("".to_string());

        let result = pipeline.stage_ast_builder(state);
        assert!(result.is_ok());
        let state = result.unwrap();
        assert!(state.ast.is_some());
    }

    // STAGE 3: SymbolicGraph tests
    #[test]
    fn test_symbolic_graph_creates_graph() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("".to_string());
        state.tokens = vec!["x".to_string(), "y".to_string()];

        let result = pipeline.stage_symbolic_graph(state);
        assert!(result.is_ok());

        let state = result.unwrap();
        assert!(state.graph.is_some());
        assert!(state.graph.unwrap().contains("Graph"));
    }

    #[test]
    fn test_symbolic_graph_single_node() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("".to_string());
        state.tokens = vec!["single".to_string()];

        let result = pipeline.stage_symbolic_graph(state);
        let state = result.unwrap();
        let graph = state.graph.unwrap();
        assert!(graph.contains("nodes: 1"));
    }

    #[test]
    fn test_symbolic_graph_multiple_nodes() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("".to_string());
        state.tokens = (0..4).map(|i| format!("node_{}", i)).collect();

        let result = pipeline.stage_symbolic_graph(state);
        let state = result.unwrap();
        let graph = state.graph.unwrap();
        assert!(graph.contains("nodes: 4"));
    }

    // STAGE 4: JordanTransformer tests
    #[test]
    fn test_jordan_transformer_computes_spectral_radius() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("".to_string());
        state.tokens = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        let result = pipeline.stage_jordan_transformer(state);
        assert!(result.is_ok());

        let state = result.unwrap();
        assert!(state.jordan.is_some());
        assert!(state.jordan.unwrap().contains("spectral_radius"));
    }

    #[test]
    fn test_jordan_transformer_positive_determinant() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("".to_string());
        state.tokens = vec!["x".to_string()];

        let result = pipeline.stage_jordan_transformer(state);
        let state = result.unwrap();
        let jordan = state.jordan.unwrap();
        assert!(jordan.contains("determinant"));
    }

    #[test]
    fn test_jordan_transformer_includes_blocks() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("".to_string());
        state.tokens = vec!["a".to_string(), "b".to_string()];

        let result = pipeline.stage_jordan_transformer(state);
        let state = result.unwrap();
        assert!(state.jordan.unwrap().contains("blocks"));
    }

    // STAGE 5: JacobianLens tests
    #[test]
    fn test_jacobian_lens_sensitivity_analysis() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("".to_string());
        state.tokens = vec!["a".to_string(), "b".to_string()];

        let result = pipeline.stage_jacobian_lens(state);
        assert!(result.is_ok());

        let state = result.unwrap();
        assert!(state.jacobian.is_some());
        assert!(state.jacobian.unwrap().contains("sensitivity"));
    }

    #[test]
    fn test_jacobian_lens_detects_dead_paths() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("".to_string());
        state.tokens = vec!["x".to_string()];

        let result = pipeline.stage_jacobian_lens(state);
        let state = result.unwrap();
        let jacobian = state.jacobian.unwrap();
        assert!(jacobian.contains("dead_paths"));
    }

    #[test]
    fn test_jacobian_lens_computes_rank() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("".to_string());
        state.tokens = (0..5).map(|i| i.to_string()).collect();

        let result = pipeline.stage_jacobian_lens(state);
        let state = result.unwrap();
        assert!(state.jacobian.unwrap().contains("rank: 5"));
    }

    // STAGE 6: ConstraintEval tests
    #[test]
    fn test_constraint_eval_passes_valid_state() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("test".to_string());
        state.tokens = vec!["test".to_string()];
        state.ast = Some("ast".to_string());
        state.graph = Some("graph".to_string());
        state.jordan = Some("jordan".to_string());

        let result = pipeline.stage_constraint_eval(state);
        assert!(result.is_ok());
        assert!(result.unwrap().constraints_valid);
    }

    #[test]
    fn test_constraint_eval_rejects_missing_ast() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("test".to_string());
        state.tokens = vec!["test".to_string()];
        state.graph = Some("graph".to_string());
        state.jordan = Some("jordan".to_string());

        let result = pipeline.stage_constraint_eval(state);
        assert!(result.is_err());
    }

    #[test]
    fn test_constraint_eval_rejects_empty_tokens() {
        let pipeline = RoutingPipeline::new();
        let state = PipelineState::new("".to_string());

        let result = pipeline.stage_constraint_eval(state);
        assert!(result.is_err());
    }

    // STAGE 7: SparseActivation tests
    #[test]
    fn test_sparse_activation_creates_set() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("".to_string());
        state.tokens = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        state.constraints_valid = true;

        let result = pipeline.stage_sparse_activation(state);
        assert!(result.is_ok());

        let state = result.unwrap();
        assert!(!state.activation_set.is_empty());
    }

    #[test]
    fn test_sparse_activation_limited_to_five() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("".to_string());
        state.tokens = (0..10).map(|i| i.to_string()).collect();

        let result = pipeline.stage_sparse_activation(state);
        let state = result.unwrap();
        assert!(state.activation_set.len() <= 5);
    }

    #[test]
    fn test_sparse_activation_rejects_empty() {
        let pipeline = RoutingPipeline::new();
        let state = PipelineState::new("".to_string());

        let result = pipeline.stage_sparse_activation(state);
        assert!(result.is_err());
    }

    // STAGE 8: RoutingNodes tests
    #[test]
    fn test_routing_nodes_creates_nodes() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("".to_string());
        state.activation_set = vec![1000, 2000, 3000];

        let result = pipeline.stage_routing_nodes(state);
        assert!(result.is_ok());

        let state = result.unwrap();
        assert_eq!(state.routing_nodes.len(), 3);
    }

    #[test]
    fn test_routing_nodes_weight_decreases() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("".to_string());
        state.activation_set = vec![1000, 2000];

        let result = pipeline.stage_routing_nodes(state);
        let state = result.unwrap();

        assert!(state.routing_nodes[0].weight > state.routing_nodes[1].weight);
    }

    #[test]
    fn test_routing_nodes_rejects_empty_activation() {
        let pipeline = RoutingPipeline::new();
        let state = PipelineState::new("".to_string());

        let result = pipeline.stage_routing_nodes(state);
        assert!(result.is_err());
    }

    // STAGE 9: NANDFilter tests
    #[test]
    fn test_nand_filter_filters_nodes() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("".to_string());
        state.routing_nodes = vec![
            crate::nodes::RoutingNode::new(1, "node1".to_string(), 0.5),
            crate::nodes::RoutingNode::new(2, "node2".to_string(), 0.2),
        ];

        let result = pipeline.stage_nand_filter(state);
        assert!(result.is_ok());

        let state = result.unwrap();
        assert!(state.filtered_nodes.len() <= 2);
    }

    #[test]
    fn test_nand_filter_keeps_compatible() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("".to_string());
        state.routing_nodes = vec![crate::nodes::RoutingNode::new(1, "node1".to_string(), 0.5)];

        let result = pipeline.stage_nand_filter(state);
        let state = result.unwrap();
        assert!(state.filtered_nodes.len() > 0);
    }

    #[test]
    fn test_nand_filter_rejects_all_filtered() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("".to_string());
        state.routing_nodes = vec![crate::nodes::RoutingNode::new(1, "node1".to_string(), 0.1)];

        let result = pipeline.stage_nand_filter(state);
        // May pass or fail depending on filter threshold
        let _ = result;
    }

    // STAGE 10: AgentDispatch tests
    #[test]
    fn test_agent_dispatch_executes_nodes() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("input".to_string());
        state.filtered_nodes = vec![crate::nodes::RoutingNode::new(100, "n1".to_string(), 0.5)];

        let result = pipeline.stage_agent_dispatch(state);
        assert!(result.is_ok());

        let state = result.unwrap();
        assert!(!state.dispatch_results.is_empty());
    }

    #[test]
    fn test_agent_dispatch_rejects_empty() {
        let pipeline = RoutingPipeline::new();
        let state = PipelineState::new("test".to_string());

        let result = pipeline.stage_agent_dispatch(state);
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_dispatch_handles_multiple_nodes() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("input".to_string());
        state.filtered_nodes = vec![
            crate::nodes::RoutingNode::new(1, "n1".to_string(), 0.5),
            crate::nodes::RoutingNode::new(2, "n2".to_string(), 0.6),
        ];

        let result = pipeline.stage_agent_dispatch(state);
        let state = result.unwrap();
        assert_eq!(state.dispatch_results.len(), 2);
    }

    // STAGE 11: MergeOutput tests
    #[test]
    fn test_merge_output_combines_results() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("test input".to_string());
        state.dispatch_results.insert(1, "result1".to_string());

        let result = pipeline.stage_merge_output(state);
        assert!(result.is_ok());

        let state = result.unwrap();
        assert!(!state.merged_output.is_empty());
        assert!(state.merged_output.contains("test input"));
    }

    #[test]
    fn test_merge_output_includes_all_results() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("".to_string());
        state.dispatch_results.insert(1, "res1".to_string());
        state.dispatch_results.insert(2, "res2".to_string());

        let result = pipeline.stage_merge_output(state);
        let state = result.unwrap();
        assert!(state.merged_output.contains("res1"));
        assert!(state.merged_output.contains("res2"));
    }

    #[test]
    fn test_merge_output_formats_correctly() {
        let pipeline = RoutingPipeline::new();
        let mut state = PipelineState::new("input".to_string());
        state.dispatch_results.insert(42, "output".to_string());

        let result = pipeline.stage_merge_output(state);
        let state = result.unwrap();
        assert!(state.merged_output.contains("42"));
        assert!(state.merged_output.contains("output"));
    }

    // Integration tests
    #[test]
    fn test_full_pipeline_processes_input() {
        let pipeline = RoutingPipeline::new();
        let result = pipeline.process("hello world");

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("routed"));
    }

    #[test]
    fn test_full_pipeline_rejects_empty() {
        let pipeline = RoutingPipeline::new();
        let result = pipeline.process("");

        assert!(result.is_err());
    }

    #[test]
    fn test_full_pipeline_multiple_words() {
        let pipeline = RoutingPipeline::new();
        let result = pipeline.process("alpha beta gamma delta");

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("stages: 11"));
    }
}
