-- backend/relational-engine/examples/TreeInvert.lean
--
-- Tree Inversion Correctness Proof
-- ==================================
-- Formalizes the Lean 4 certificate from exec-tree-invert-001.
--
-- Two properties proven:
--   1. tree_invert preserves depth
--   2. tree_invert is the mirror (structural inverse)
--
-- Connects to:
--   examples/tree-invert.mjs              (runtime verification)
--   examples/exec-tree-invert-001.sgml    (execution trace)
--   relational-refinement-engine.mjs      (synthesis engine)
--
-- Ahmad Ali Parr -- Bel Esprit D'Accord Irrevocable Trust -- EIN 42-697643

import Mathlib.Data.Nat.Basic
import Mathlib.Tactic

namespace TreeInvert

-- ── Tree type ─────────────────────────────────────────────────────────────────

inductive Tree (α : Type) where
  | leaf : Tree α
  | node : α → Tree α → Tree α → Tree α
deriving Repr

-- ── tree_invert: swap left and right at every node ───────────────────────────

def tree_invert : Tree α → Tree α
  | .leaf          => .leaf
  | .node v l r    => .node v (tree_invert r) (tree_invert l)

-- ── depth: number of levels ───────────────────────────────────────────────────

def depth : Tree α → ℕ
  | .leaf       => 0
  | .node _ l r => 1 + max (depth l) (depth r)

-- ── mirror: structural definition of tree inversion ──────────────────────────
-- mirror t = tree_invert t (the two definitions coincide)

def mirror : Tree α → Tree α := tree_invert

-- ── THEOREM 1: tree_invert preserves depth ───────────────────────────────────

theorem tree_invert_preserves_depth (t : Tree α) :
    depth (tree_invert t) = depth t := by
  induction t with
  | leaf => rfl
  | node v l r ihl ihr =>
    simp [tree_invert, depth, Nat.max_comm]
    omega

-- ── THEOREM 2: tree_invert is an involution: invert (invert t) = t ───────────

theorem tree_invert_involution (t : Tree α) :
    tree_invert (tree_invert t) = t := by
  induction t with
  | leaf => rfl
  | node v l r ihl ihr =>
    simp [tree_invert, ihl, ihr]

-- ── THEOREM 3: The synthesis certificate from exec-tree-invert-001 ───────────
-- depth is preserved AND invert is its own inverse

theorem tree_invert_correct (t : Tree α) :
    depth (tree_invert t) = depth t ∧
    tree_invert (tree_invert t) = t := by
  exact ⟨tree_invert_preserves_depth t, tree_invert_involution t⟩

-- ── THEOREM 4: Example from execution trace ──────────────────────────────────
-- Input:  node 1 (node 2 leaf leaf) (node 3 leaf leaf)
-- Output: node 1 (node 3 leaf leaf) (node 2 leaf leaf)

theorem tree_invert_example :
    tree_invert (.node 1 (.node 2 .leaf .leaf) (.node 3 .leaf .leaf))
    = .node 1 (.node 3 .leaf .leaf) (.node 2 .leaf .leaf) := by
  simp [tree_invert]

end TreeInvert
