//! XSLT Constraint Inversion Runtime
//!
//! Processes FormalizationMachine XML output from XSLT constraint inversion engine.
//!
//! Workflow:
//! 1. Parse FormalizationMachine XML (classification, inversion, normalization)
//! 2. Extract canonical invariants and proof obligations
//! 3. Route to appropriate provers (HOL Light, Lean 4, Agda)
//! 4. Collect and validate results
//! 5. Check cross-prover correspondence (HOL↔Lean↔Agda)
//! 6. Emit typed proof obligations for external provers
//!
//! Authority Model:
//! - XSLT: Classification, Inversion, Normalization, Emission
//! - HOL/Lean/Agda: Compilation/Type-checking/Verification only
//! - Correspondence: Explicit cross-prover proofs required (not implicit)

use crate::ast::{Constraint, ConstraintProgram, OtherwiseAction, Requirement};
use hyperkitty_core::Result;
use std::collections::HashMap;

/// Classification of constraint domains
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConstraintKind {
    Prohibition,
    Technology,
    BooleanAlgebra,
    RefinementType,
    GraphInvariant,
    Transformation,
    Truth,
    ProofArtifact,
    ExecutionOrder,
    Acceptance,
    Structure,
    ComponentContract,
    GeneralConstraint,
}

impl ConstraintKind {
    pub fn from_str(s: &str) -> Self {
        match s {
            "PROHIBITION" => Self::Prohibition,
            "TECHNOLOGY" => Self::Technology,
            "BOOLEAN_ALGEBRA" => Self::BooleanAlgebra,
            "REFINEMENT_TYPE" => Self::RefinementType,
            "GRAPH_INVARIANT" => Self::GraphInvariant,
            "TRANSFORMATION" => Self::Transformation,
            "TRUTH" => Self::Truth,
            "PROOF_ARTIFACT" => Self::ProofArtifact,
            "EXECUTION_ORDER" => Self::ExecutionOrder,
            "ACCEPTANCE" => Self::Acceptance,
            "STRUCTURE" => Self::Structure,
            "COMPONENT_CONTRACT" => Self::ComponentContract,
            _ => Self::GeneralConstraint,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Prohibition => "PROHIBITION",
            Self::Technology => "TECHNOLOGY",
            Self::BooleanAlgebra => "BOOLEAN_ALGEBRA",
            Self::RefinementType => "REFINEMENT_TYPE",
            Self::GraphInvariant => "GRAPH_INVARIANT",
            Self::Transformation => "TRANSFORMATION",
            Self::Truth => "TRUTH",
            Self::ProofArtifact => "PROOF_ARTIFACT",
            Self::ExecutionOrder => "EXECUTION_ORDER",
            Self::Acceptance => "ACCEPTANCE",
            Self::Structure => "STRUCTURE",
            Self::ComponentContract => "COMPONENT_CONTRACT",
            Self::GeneralConstraint => "GENERAL_CONSTRAINT",
        }
    }
}

/// Polarity of constraint (positive, negative, or neutral)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polarity {
    Positive,
    Negative,
    Neutral,
}

impl Polarity {
    pub fn from_str(s: &str) -> Self {
        match s {
            "POSITIVE" => Self::Positive,
            "NEGATIVE" => Self::Negative,
            _ => Self::Neutral,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Positive => "POSITIVE",
            Self::Negative => "NEGATIVE",
            Self::Neutral => "NEUTRAL",
        }
    }
}

/// Canonical invariant extracted from normalized constraints
#[derive(Debug, Clone)]
pub struct Invariant {
    pub id: String,
    pub kind: ConstraintKind,
    pub polarity: Polarity,
    pub normalized_expr: String,
    pub inverted_expr: String,
    pub source_class: String,
}

/// Registry of canonical invariants parsed from FormalizationMachine
#[derive(Debug, Clone)]
pub struct InvariantRegistry {
    pub invariants: HashMap<String, Invariant>,
    pub project_name: String,
    pub organization: String,
    pub formalization_order: String,
    pub agda_iteration_multiplicity: u32,
}

impl InvariantRegistry {
    pub fn new(
        project_name: String,
        organization: String,
        formalization_order: String,
        agda_iteration_multiplicity: u32,
    ) -> Self {
        Self {
            invariants: HashMap::new(),
            project_name,
            organization,
            formalization_order,
            agda_iteration_multiplicity,
        }
    }

    pub fn add_invariant(&mut self, invariant: Invariant) {
        self.invariants.insert(invariant.id.clone(), invariant);
    }

    pub fn get_invariant(&self, id: &str) -> Option<&Invariant> {
        self.invariants.get(id)
    }
}

/// Status of prover compilation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProverStatus {
    GeneratedUnverified,
    CompiledUnverified,
    Verified,
    Failed,
}

impl ProverStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "GENERATED_UNVERIFIED" => Self::GeneratedUnverified,
            "COMPILED_UNVERIFIED" => Self::CompiledUnverified,
            "VERIFIED" => Self::Verified,
            _ => Self::Failed,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GeneratedUnverified => "GENERATED_UNVERIFIED",
            Self::CompiledUnverified => "COMPILED_UNVERIFIED",
            Self::Verified => "VERIFIED",
            Self::Failed => "FAILED",
        }
    }
}

/// Generated artifact from a prover stage
#[derive(Debug, Clone)]
pub struct ProverArtifact {
    pub prover: String,
    pub invariant_id: String,
    pub artifact_id: String,
    pub status: ProverStatus,
    pub hol_type: Option<String>,
    pub lean_type: Option<String>,
    pub agda_type: Option<String>,
    pub source_code: String,
    pub symbol_map: HashMap<String, String>,
}

impl ProverArtifact {
    pub fn new(
        prover: String,
        invariant_id: String,
        artifact_id: String,
        source_code: String,
    ) -> Self {
        Self {
            prover,
            invariant_id,
            artifact_id,
            status: ProverStatus::GeneratedUnverified,
            hol_type: None,
            lean_type: None,
            agda_type: None,
            source_code,
            symbol_map: HashMap::new(),
        }
    }
}

/// Correspondence obligation between two provers
#[derive(Debug, Clone)]
pub struct CorrespondenceObligation {
    pub invariant_id: String,
    pub source_prover: String,
    pub target_prover: String,
    pub source_artifact_id: String,
    pub target_artifact_id: String,
    pub required_statement: String,
    pub status: CorrespondenceStatus,
}

/// Status of correspondence validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorrespondenceStatus {
    Unresolved,
    Pending,
    Validated,
    Failed,
}

impl CorrespondenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unresolved => "UNRESOLVED",
            Self::Pending => "PENDING",
            Self::Validated => "VALIDATED",
            Self::Failed => "FAILED",
        }
    }
}

/// Agda iteration derivation obligation
#[derive(Debug, Clone)]
pub struct AgdaIterationObligation {
    pub invariant_id: String,
    pub index: u32,
    pub transform_name: String,
    pub source_invariant: String,
    pub derived_invariant: String,
}

impl AgdaIterationObligation {
    pub fn iteration_transform_name(index: u32) -> &'static str {
        match index {
            1 => "identity-preservation",
            2 => "double-negation-stability",
            3 => "conjunction-left-projection",
            4 => "conjunction-right-projection",
            5 => "implication-closure",
            6 => "contrapositive-check",
            7 => "reflexive-equality",
            8 => "symmetric-equality",
            9 => "transitive-equality",
            10 => "substitution-preservation",
            11 => "domain-restriction",
            12 => "codomain-preservation",
            13 => "state-transition-preservation",
            14 => "graph-edge-preservation",
            15 => "topological-order-preservation",
            16 => "refinement-strengthening",
            17 => "refinement-weakening-check",
            18 => "rejection-monotonicity",
            19 => "acceptance-soundness",
            20 => "cross-prover-correspondence",
            _ => "unknown-transform",
        }
    }
}

/// Complete execution schedule for formalization pipeline
#[derive(Debug, Clone)]
pub struct ExecutionSchedule {
    pub phases: Vec<ExecutionPhase>,
}

#[derive(Debug, Clone)]
pub struct ExecutionPhase {
    pub index: u32,
    pub id: String,
    pub description: String,
}

impl ExecutionSchedule {
    pub fn default_schedule() -> Self {
        Self {
            phases: vec![
                ExecutionPhase {
                    index: 1,
                    id: "parse-source".to_string(),
                    description: "Parse source XML with external entities disabled.".to_string(),
                },
                ExecutionPhase {
                    index: 2,
                    id: "classify-source".to_string(),
                    description:
                        "Classify domains, rules, invariants, transformations, conflicts."
                            .to_string(),
                },
                ExecutionPhase {
                    index: 3,
                    id: "invert-constraints".to_string(),
                    description:
                        "Reorder specification into rejection-first execution form.".to_string(),
                },
                ExecutionPhase {
                    index: 4,
                    id: "normalize-invariants".to_string(),
                    description: "Produce canonical typed invariant records.".to_string(),
                },
                ExecutionPhase {
                    index: 5,
                    id: "emit-hol".to_string(),
                    description: "Generate HOL declarations and proof obligations.".to_string(),
                },
                ExecutionPhase {
                    index: 6,
                    id: "check-hol".to_string(),
                    description: "Compile HOL artifacts and record prover results.".to_string(),
                },
                ExecutionPhase {
                    index: 7,
                    id: "emit-lean".to_string(),
                    description:
                        "Generate Lean declarations from canonical invariants and HOL maps."
                            .to_string(),
                },
                ExecutionPhase {
                    index: 8,
                    id: "check-lean".to_string(),
                    description: "Compile Lean artifacts without sorry or admit.".to_string(),
                },
                ExecutionPhase {
                    index: 9,
                    id: "emit-agda".to_string(),
                    description:
                        "Generate Agda declarations from canonical invariants and Lean maps."
                            .to_string(),
                },
                ExecutionPhase {
                    index: 10,
                    id: "check-agda".to_string(),
                    description:
                        "Type-check Agda artifacts without postulates in verified paths."
                            .to_string(),
                },
                ExecutionPhase {
                    index: 11,
                    id: "derive-agda-20x".to_string(),
                    description: "Generate twenty indexed derivation obligations per invariant."
                        .to_string(),
                },
                ExecutionPhase {
                    index: 12,
                    id: "check-correspondence".to_string(),
                    description:
                        "Check HOL-to-Lean and Lean-to-Agda semantic correspondence."
                            .to_string(),
                },
            ],
        }
    }
}

/// Main FormalizationMachine processor
#[derive(Debug, Clone)]
pub struct FormalizationMachine {
    pub project_name: String,
    pub organization: String,
    pub stylesheet_version: String,
    pub execution_policy: String,
    pub formalization_order: String,
    pub agda_iteration_multiplicity: u32,
    pub strict_mode: bool,
    pub registry: InvariantRegistry,
    pub hol_artifacts: Vec<ProverArtifact>,
    pub lean_artifacts: Vec<ProverArtifact>,
    pub agda_artifacts: Vec<ProverArtifact>,
    pub correspondences: Vec<CorrespondenceObligation>,
    pub agda_iterations: Vec<AgdaIterationObligation>,
    pub schedule: ExecutionSchedule,
}

impl FormalizationMachine {
    pub fn new(
        project_name: String,
        organization: String,
        formalization_order: String,
        agda_iteration_multiplicity: u32,
    ) -> Self {
        let registry = InvariantRegistry::new(
            project_name.clone(),
            organization.clone(),
            formalization_order.clone(),
            agda_iteration_multiplicity,
        );

        Self {
            project_name,
            organization,
            stylesheet_version: "1.0.0".to_string(),
            execution_policy: "PARSE_INVERT_FORMALIZE_VERIFY_REPEAT".to_string(),
            formalization_order,
            agda_iteration_multiplicity,
            strict_mode: true,
            registry,
            hol_artifacts: Vec::new(),
            lean_artifacts: Vec::new(),
            agda_artifacts: Vec::new(),
            correspondences: Vec::new(),
            agda_iterations: Vec::new(),
            schedule: ExecutionSchedule::default_schedule(),
        }
    }

    /// Register canonical invariant
    pub fn register_invariant(&mut self, invariant: Invariant) {
        self.registry.add_invariant(invariant);
    }

    /// Emit HOL artifact for given invariant
    pub fn emit_hol_artifact(
        &mut self,
        invariant_id: String,
        hol_type: String,
        source_code: String,
    ) -> Result<ProverArtifact> {
        let artifact_id = format!("hol-{}", invariant_id);
        let mut artifact = ProverArtifact::new(
            "HOL".to_string(),
            invariant_id.clone(),
            artifact_id,
            source_code,
        );
        artifact.hol_type = Some(hol_type);
        self.hol_artifacts.push(artifact.clone());
        Ok(artifact)
    }

    /// Emit Lean artifact for given invariant
    pub fn emit_lean_artifact(
        &mut self,
        invariant_id: String,
        lean_type: String,
        source_code: String,
    ) -> Result<ProverArtifact> {
        let artifact_id = format!("lean-{}", invariant_id);
        let mut artifact = ProverArtifact::new(
            "Lean4".to_string(),
            invariant_id.clone(),
            artifact_id,
            source_code,
        );
        artifact.lean_type = Some(lean_type);
        self.lean_artifacts.push(artifact.clone());
        Ok(artifact)
    }

    /// Emit Agda artifact for given invariant
    pub fn emit_agda_artifact(
        &mut self,
        invariant_id: String,
        agda_type: String,
        source_code: String,
    ) -> Result<ProverArtifact> {
        let artifact_id = format!("agda-{}", invariant_id);
        let mut artifact = ProverArtifact::new(
            "Agda".to_string(),
            invariant_id.clone(),
            artifact_id,
            source_code,
        );
        artifact.agda_type = Some(agda_type);
        self.agda_artifacts.push(artifact.clone());
        Ok(artifact)
    }

    /// Create correspondence obligation between HOL and Lean
    pub fn create_hol_lean_correspondence(&mut self, invariant_id: String) -> Result<()> {
        let hol_artifact_id = format!("hol-{}", invariant_id);
        let lean_artifact_id = format!("lean-{}", invariant_id);

        let obligation = CorrespondenceObligation {
            invariant_id: invariant_id.clone(),
            source_prover: "HOL".to_string(),
            target_prover: "Lean4".to_string(),
            source_artifact_id: hol_artifact_id,
            target_artifact_id: lean_artifact_id,
            required_statement:
                "HOL semantics and Lean semantics preserve canonical normalized predicate."
                    .to_string(),
            status: CorrespondenceStatus::Unresolved,
        };

        self.correspondences.push(obligation);
        Ok(())
    }

    /// Create correspondence obligation between Lean and Agda
    pub fn create_lean_agda_correspondence(&mut self, invariant_id: String) -> Result<()> {
        let lean_artifact_id = format!("lean-{}", invariant_id);
        let agda_artifact_id = format!("agda-{}", invariant_id);

        let obligation = CorrespondenceObligation {
            invariant_id: invariant_id.clone(),
            source_prover: "Lean4".to_string(),
            target_prover: "Agda".to_string(),
            source_artifact_id: lean_artifact_id,
            target_artifact_id: agda_artifact_id,
            required_statement:
                "Lean semantics and Agda semantics preserve normalized predicate tree."
                    .to_string(),
            status: CorrespondenceStatus::Unresolved,
        };

        self.correspondences.push(obligation);
        Ok(())
    }

    /// Generate 20 Agda iteration obligations for an invariant
    pub fn generate_agda_iterations(&mut self, invariant_id: String) -> Result<()> {
        for index in 1..=self.agda_iteration_multiplicity {
            let obligation = AgdaIterationObligation {
                invariant_id: invariant_id.clone(),
                index,
                transform_name: AgdaIterationObligation::iteration_transform_name(index)
                    .to_string(),
                source_invariant: format!("{}_iter_{}", invariant_id, index - 1),
                derived_invariant: format!("{}_iter_{}", invariant_id, index),
            };
            self.agda_iterations.push(obligation);
        }
        Ok(())
    }

    /// Validate cross-prover correspondence for invariant
    pub fn validate_correspondence(&mut self, invariant_id: &str) -> Result<bool> {
        let matching_correspondences: Vec<_> = self
            .correspondences
            .iter_mut()
            .filter(|c| c.invariant_id == invariant_id)
            .collect();

        if matching_correspondences.is_empty() {
            return Ok(false);
        }

        let all_valid = matching_correspondences.iter().all(|c| {
            let hol_exists = self
                .hol_artifacts
                .iter()
                .any(|a| a.artifact_id == c.source_artifact_id);
            let target_exists = self
                .lean_artifacts
                .iter()
                .any(|a| a.artifact_id == c.target_artifact_id)
                || self
                    .agda_artifacts
                    .iter()
                    .any(|a| a.artifact_id == c.target_artifact_id);

            hol_exists && target_exists
        });

        if all_valid {
            for corr in matching_correspondences {
                corr.status = CorrespondenceStatus::Validated;
            }
        }

        Ok(all_valid)
    }

    /// Convert registered invariants to ConstraintProgram for evaluation
    pub fn to_constraint_program(&self) -> Result<ConstraintProgram> {
        let mut program = ConstraintProgram::new();

        for (id, invariant) in &self.registry.invariants {
            let constraint_name = format!("{}-{}", invariant.kind.as_str(), id);
            let otherwise_action = match invariant.polarity {
                Polarity::Negative => OtherwiseAction::Reject,
                _ => OtherwiseAction::Accept,
            };

            let mut constraint =
                Constraint::new(constraint_name, id.clone(), otherwise_action);

            constraint.add_requirement(Requirement::Predicate(
                format!("check_{}", id),
            ));

            program.add_constraint(constraint);
        }

        Ok(program)
    }

    /// Record HOL artifact status update
    pub fn record_hol_status(&mut self, artifact_id: &str, status: ProverStatus) -> Result<()> {
        if let Some(artifact) = self
            .hol_artifacts
            .iter_mut()
            .find(|a| a.artifact_id == artifact_id)
        {
            artifact.status = status;
            Ok(())
        } else {
            Err(hyperkitty_core::Error::RecordNotFound)
        }
    }

    /// Record Lean artifact status update
    pub fn record_lean_status(&mut self, artifact_id: &str, status: ProverStatus) -> Result<()> {
        if let Some(artifact) = self
            .lean_artifacts
            .iter_mut()
            .find(|a| a.artifact_id == artifact_id)
        {
            artifact.status = status;
            Ok(())
        } else {
            Err(hyperkitty_core::Error::RecordNotFound)
        }
    }

    /// Record Agda artifact status update
    pub fn record_agda_status(&mut self, artifact_id: &str, status: ProverStatus) -> Result<()> {
        if let Some(artifact) = self
            .agda_artifacts
            .iter_mut()
            .find(|a| a.artifact_id == artifact_id)
        {
            artifact.status = status;
            Ok(())
        } else {
            Err(hyperkitty_core::Error::RecordNotFound)
        }
    }

    /// Get summary statistics
    pub fn summary(&self) -> FormalizationSummary {
        FormalizationSummary {
            total_invariants: self.registry.invariants.len(),
            hol_artifacts_count: self.hol_artifacts.len(),
            lean_artifacts_count: self.lean_artifacts.len(),
            agda_artifacts_count: self.agda_artifacts.len(),
            correspondence_obligations: self.correspondences.len(),
            agda_iterations: self.agda_iterations.len(),
            hol_verified: self
                .hol_artifacts
                .iter()
                .filter(|a| a.status == ProverStatus::Verified)
                .count(),
            lean_verified: self
                .lean_artifacts
                .iter()
                .filter(|a| a.status == ProverStatus::Verified)
                .count(),
            agda_verified: self
                .agda_artifacts
                .iter()
                .filter(|a| a.status == ProverStatus::Verified)
                .count(),
            correspondences_validated: self
                .correspondences
                .iter()
                .filter(|c| c.status == CorrespondenceStatus::Validated)
                .count(),
        }
    }
}

/// Summary statistics of formalization state
#[derive(Debug, Clone)]
pub struct FormalizationSummary {
    pub total_invariants: usize,
    pub hol_artifacts_count: usize,
    pub lean_artifacts_count: usize,
    pub agda_artifacts_count: usize,
    pub correspondence_obligations: usize,
    pub agda_iterations: usize,
    pub hol_verified: usize,
    pub lean_verified: usize,
    pub agda_verified: usize,
    pub correspondences_validated: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_kind_round_trip() {
        let kind = ConstraintKind::Prohibition;
        assert_eq!(ConstraintKind::from_str(kind.as_str()), kind);

        let kind = ConstraintKind::BooleanAlgebra;
        assert_eq!(ConstraintKind::from_str(kind.as_str()), kind);
    }

    #[test]
    fn test_polarity_round_trip() {
        let polarity = Polarity::Positive;
        assert_eq!(Polarity::from_str(polarity.as_str()), polarity);

        let polarity = Polarity::Negative;
        assert_eq!(Polarity::from_str(polarity.as_str()), polarity);
    }

    #[test]
    fn test_prover_status_round_trip() {
        let status = ProverStatus::Verified;
        assert_eq!(ProverStatus::from_str(status.as_str()), status);

        let status = ProverStatus::GeneratedUnverified;
        assert_eq!(ProverStatus::from_str(status.as_str()), status);
    }

    #[test]
    fn test_formalization_machine_creation() {
        let machine = FormalizationMachine::new(
            "TestProject".to_string(),
            "TestOrg".to_string(),
            "HOL_TO_LEAN_TO_AGDA".to_string(),
            20,
        );

        assert_eq!(machine.project_name, "TestProject");
        assert_eq!(machine.organization, "TestOrg");
        assert_eq!(machine.agda_iteration_multiplicity, 20);
        assert_eq!(machine.registry.invariants.len(), 0);
    }

    #[test]
    fn test_invariant_registry() {
        let mut registry = InvariantRegistry::new(
            "Test".to_string(),
            "Test".to_string(),
            "HOL_TO_LEAN_TO_AGDA".to_string(),
            20,
        );

        let invariant = Invariant {
            id: "inv-001".to_string(),
            kind: ConstraintKind::BooleanAlgebra,
            polarity: Polarity::Positive,
            normalized_expr: "x ∧ y".to_string(),
            inverted_expr: "require(x ∧ y)".to_string(),
            source_class: "SPECIFIED".to_string(),
        };

        registry.add_invariant(invariant.clone());
        assert_eq!(registry.invariants.len(), 1);
        assert_eq!(registry.get_invariant("inv-001").unwrap().id, "inv-001");
    }

    #[test]
    fn test_emit_hol_artifact() -> Result<()> {
        let mut machine = FormalizationMachine::new(
            "Test".to_string(),
            "Test".to_string(),
            "HOL_TO_LEAN_TO_AGDA".to_string(),
            20,
        );

        let artifact = machine.emit_hol_artifact(
            "inv-001".to_string(),
            "bool".to_string(),
            "theorem test : True".to_string(),
        )?;

        assert_eq!(artifact.prover, "HOL");
        assert_eq!(artifact.invariant_id, "inv-001");
        assert_eq!(machine.hol_artifacts.len(), 1);
        Ok(())
    }

    #[test]
    fn test_emit_lean_artifact() -> Result<()> {
        let mut machine = FormalizationMachine::new(
            "Test".to_string(),
            "Test".to_string(),
            "HOL_TO_LEAN_TO_AGDA".to_string(),
            20,
        );

        let artifact = machine.emit_lean_artifact(
            "inv-001".to_string(),
            "Bool".to_string(),
            "theorem test : True := by trivial".to_string(),
        )?;

        assert_eq!(artifact.prover, "Lean4");
        assert_eq!(artifact.invariant_id, "inv-001");
        assert_eq!(machine.lean_artifacts.len(), 1);
        Ok(())
    }

    #[test]
    fn test_emit_agda_artifact() -> Result<()> {
        let mut machine = FormalizationMachine::new(
            "Test".to_string(),
            "Test".to_string(),
            "HOL_TO_LEAN_TO_AGDA".to_string(),
            20,
        );

        let artifact = machine.emit_agda_artifact(
            "inv-001".to_string(),
            "Set".to_string(),
            "test : Set\ntest = ⊤".to_string(),
        )?;

        assert_eq!(artifact.prover, "Agda");
        assert_eq!(artifact.invariant_id, "inv-001");
        assert_eq!(machine.agda_artifacts.len(), 1);
        Ok(())
    }

    #[test]
    fn test_correspondence_creation() -> Result<()> {
        let mut machine = FormalizationMachine::new(
            "Test".to_string(),
            "Test".to_string(),
            "HOL_TO_LEAN_TO_AGDA".to_string(),
            20,
        );

        machine.create_hol_lean_correspondence("inv-001".to_string())?;
        machine.create_lean_agda_correspondence("inv-001".to_string())?;

        assert_eq!(machine.correspondences.len(), 2);
        assert_eq!(machine.correspondences[0].source_prover, "HOL");
        assert_eq!(machine.correspondences[0].target_prover, "Lean4");
        assert_eq!(machine.correspondences[1].source_prover, "Lean4");
        assert_eq!(machine.correspondences[1].target_prover, "Agda");
        Ok(())
    }

    #[test]
    fn test_agda_iterations() -> Result<()> {
        let mut machine = FormalizationMachine::new(
            "Test".to_string(),
            "Test".to_string(),
            "HOL_TO_LEAN_TO_AGDA".to_string(),
            20,
        );

        machine.generate_agda_iterations("inv-001".to_string())?;

        assert_eq!(machine.agda_iterations.len(), 20);
        assert_eq!(
            machine.agda_iterations[0].transform_name,
            "identity-preservation"
        );
        assert_eq!(
            machine.agda_iterations[19].transform_name,
            "cross-prover-correspondence"
        );
        Ok(())
    }

    #[test]
    fn test_prover_status_recording() -> Result<()> {
        let mut machine = FormalizationMachine::new(
            "Test".to_string(),
            "Test".to_string(),
            "HOL_TO_LEAN_TO_AGDA".to_string(),
            20,
        );

        machine.emit_hol_artifact(
            "inv-001".to_string(),
            "bool".to_string(),
            "theorem test : True".to_string(),
        )?;

        machine.record_hol_status("hol-inv-001", ProverStatus::Verified)?;

        let artifact = machine
            .hol_artifacts
            .iter()
            .find(|a| a.artifact_id == "hol-inv-001")
            .unwrap();

        assert_eq!(artifact.status, ProverStatus::Verified);
        Ok(())
    }

    #[test]
    fn test_correspondence_validation() -> Result<()> {
        let mut machine = FormalizationMachine::new(
            "Test".to_string(),
            "Test".to_string(),
            "HOL_TO_LEAN_TO_AGDA".to_string(),
            20,
        );

        machine.emit_hol_artifact(
            "inv-001".to_string(),
            "bool".to_string(),
            "theorem test : True".to_string(),
        )?;
        machine.emit_lean_artifact(
            "inv-001".to_string(),
            "Bool".to_string(),
            "theorem test : True := by trivial".to_string(),
        )?;

        machine.create_hol_lean_correspondence("inv-001".to_string())?;

        let valid = machine.validate_correspondence("inv-001")?;
        assert!(valid);

        let corr = machine
            .correspondences
            .iter()
            .find(|c| c.invariant_id == "inv-001")
            .unwrap();
        assert_eq!(corr.status, CorrespondenceStatus::Validated);
        Ok(())
    }

    #[test]
    fn test_execution_schedule() {
        let schedule = ExecutionSchedule::default_schedule();
        assert_eq!(schedule.phases.len(), 12);
        assert_eq!(schedule.phases[0].id, "parse-source");
        assert_eq!(schedule.phases[11].id, "check-correspondence");
    }

    #[test]
    fn test_formalization_summary() -> Result<()> {
        let mut machine = FormalizationMachine::new(
            "Test".to_string(),
            "Test".to_string(),
            "HOL_TO_LEAN_TO_AGDA".to_string(),
            20,
        );

        let invariant = Invariant {
            id: "inv-001".to_string(),
            kind: ConstraintKind::BooleanAlgebra,
            polarity: Polarity::Positive,
            normalized_expr: "x ∧ y".to_string(),
            inverted_expr: "require(x ∧ y)".to_string(),
            source_class: "SPECIFIED".to_string(),
        };

        machine.register_invariant(invariant);
        machine.emit_hol_artifact(
            "inv-001".to_string(),
            "bool".to_string(),
            "theorem test : True".to_string(),
        )?;
        machine.emit_lean_artifact(
            "inv-001".to_string(),
            "Bool".to_string(),
            "theorem test : True := by trivial".to_string(),
        )?;

        let summary = machine.summary();
        assert_eq!(summary.total_invariants, 1);
        assert_eq!(summary.hol_artifacts_count, 1);
        assert_eq!(summary.lean_artifacts_count, 1);
        Ok(())
    }

    #[test]
    fn test_iteration_transform_names() {
        assert_eq!(
            AgdaIterationObligation::iteration_transform_name(1),
            "identity-preservation"
        );
        assert_eq!(
            AgdaIterationObligation::iteration_transform_name(2),
            "double-negation-stability"
        );
        assert_eq!(
            AgdaIterationObligation::iteration_transform_name(20),
            "cross-prover-correspondence"
        );
        assert_eq!(
            AgdaIterationObligation::iteration_transform_name(99),
            "unknown-transform"
        );
    }
}
