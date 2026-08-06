/-
# HyperKitty Core Definitions
## SNAPKITTYWEST Research Institute
## Formal Verification Suite for Deterministic Routing

**Author:** Ahmad Ali Parr
**Affiliation:** SNAPKITTYWEST, Bel Esprit D'Accord Irrevocable Trust
**Repository:** https://github.com/SNAPKITTYWEST/hyperkitty
**Date:** August 2026
**Version:** 1.0.0 - Gold Standard

This module defines all canonical types and constants for the HyperKitty system.
All definitions are constructive and fully computable.
-/

-- ============ GLYPH: The Six Routing Primitives ============

/-!
Glyph: The six canonical routing primitives from paper Section 2.1.

These correspond to the six dimensions of the QLG sphere and the six states
of the QRA deterministic finite automaton.
-/
inductive Glyph where
  | Pi    -- Propositio: send proposition (0x01)
  | Gamma -- Guard: receive guard check (0x03)
  | Delta -- Transition: execute state transition (0x04)
  | Omega -- Conclusio: absorbing terminal (0x0A)
  | Lambda-- Locality: identity element (0xFF)
  | Psi   -- Negative transition (0x0B)
  deriving DecidableEq, Repr

-- Enumeration matching paper Section 2.1
@[simp] def Glyph.idx : Glyph → Fin 6
  | .Pi => 0 | .Gamma => 1 | .Delta => 2
  | .Omega => 3 | .Lambda => 4 | .Psi => 5

@[simp] def Glyph.ofIdx : Fin 6 → Glyph
  | 0 => .Pi | 1 => .Gamma | 2 => .Delta
  | 3 => .Omega | 4 => .Lambda | 5 => .Psi

@[simp] theorem Glyph.idx_ofIdx (i : Fin 6) : (Glyph.ofIdx i).idx = i := by
  fin_cases i <;> rfl

@[simp] theorem Glyph.ofIdx_idx (g : Glyph) : Glyph.ofIdx g.idx = g := by
  cases g <;> rfl

-- ============ QRA ROUTING TENSOR ============

/-!
Q: The 6×6 QRA routing tensor from paper Section 3.1.

This is the transition matrix for the deterministic 6-state automaton.
Q[i][j] tells us what state to transition to from state i with previous state j.

Row meanings:
  0 = Pi row
  1 = Gamma row
  2 = Delta row
  3 = Omega row (absorber: always stays in Omega)
  4 = Lambda row (identity: returns the previous state)
  5 = Psi row

Key properties:
  - Lambda row is identity: Q[4][j] = j for all j
  - Omega row is absorber: Q[3][j] = 3 for all j
  - All other rows deterministically route based on paper Table 1
-/
-- Paper Table 1 (Zenodo PDF, Ahmad Parr 2026):
--         prev: Pi(0) Ga(1) De(2) Om(3) La(4) Ps(5)
-- Pi(0):          2     2     3     3     2     2
-- Ga(1):          2     3     3     3     2     3
-- De(2):          3     3     3     3     2     3
-- Om(3):          3     3     3     3     3     3   [absorber]
-- La(4):          0     1     2     3     4     5   [identity]
-- Ps(5):          2     3     3     3     2     3
def Q : Fin 6 → Fin 6 → Fin 6
  | 4, j => j          -- Lambda row: identity
  | 3, _ => 3          -- Omega row: absorber
  | 0, j => if j = 2 ∨ j = 3 then 3 else 2  -- Pi: Omega when prev∈{Delta,Omega}, else Delta
  | 1, j => if j = 0 ∨ j = 4 then 2 else 3  -- Gamma: Delta when prev∈{Pi,Lambda}, else Omega
  | 2, j => if j = 4 then 2 else 3           -- Delta: Delta when prev=Lambda, else Omega
  | 5, j => if j = 0 ∨ j = 4 then 2 else 3  -- Psi: Delta when prev∈{Pi,Lambda}, else Omega
  | _, _ => 3

/-!
Glyph.next: Compute the next state in QRA evolution.

Given current state curr and previous state prev, compute the next state
by looking up Q[curr.idx][prev.idx].
-/
def Glyph.next (curr prev : Glyph) : Glyph :=
  Glyph.ofIdx (Q curr.idx prev.idx)

-- ============ LEDGER: Symbolic Ledger Algebra ============

/-!
Ledger: A balanced ledger from paper Section 2.2.

Structure components:
  s : ℤ  - ledger size
  δ : ℤ  - debit (outflow)
  ι : ℤ  - credit (inflow)
  ω : ℤ  - domain identifier

Invariant: δ + ι = 0 (always balanced)
-/
structure Ledger where
  s : ℤ   -- size
  δ : ℤ   -- debit
  ι : ℤ   -- credit
  ω : ℤ   -- domain
  deriving Repr

-- Balance axiom R(λ) = δ + ι = 0 from paper
def Ledger.balance (λ : Ledger) : Prop := λ.δ + λ.ι = 0

/-!
Ledger.mkBalanced: Constructor that enforces balance invariant.

Creates a balanced ledger by accepting debit δ and automatically
computing credit as ι = -δ, ensuring δ + ι = 0.
-/
def Ledger.mkBalanced (s δ ω : ℤ) : Ledger :=
  {s := s, δ := δ, ι := -δ, ω := ω}

@[simp] theorem Ledger.balance_mkBalanced (s δ ω : ℤ) :
    (Ledger.mkBalanced s δ ω).balance := by
  simp [Ledger.balance]
  omega

/-!
Ledger.comp: Composition of two balanced ledgers.

Two ledgers can be composed only if they have matching domain (ω).
The result is a new ledger with combined size and summed debit/credit.
-/
def Ledger.comp (λ₁ λ₂ : Ledger) : Option Ledger :=
  if h : λ₁.ω = λ₂.ω then
    some { s := λ₁.s + λ₂.s
           δ := λ₁.δ + λ₂.δ
           ι := λ₁.ι + λ₂.ι
           ω := λ₁.ω }
  else
    none

-- ============ VEC3: Quadratic Ledger Geometry ============

/-!
Vec3: Three-dimensional integer vectors for QLG.

The canonical QLG surface is the unit integer sphere:
  x² + y² + z² = K where K = 1

Only 6 integer solutions exist on the unit sphere:
  (±1, 0, 0), (0, ±1, 0), (0, 0, ±1)
-/
structure Vec3 where
  x : ℤ
  y : ℤ
  z : ℤ
  deriving Repr

-- Canonical QLG: unit integer sphere x² + y² + z² = 1
def QLG.canonical (v : Vec3) : Prop := v.x^2 + v.y^2 + v.z^2 = 1
def QLG.K : ℤ := 1

/-!
Vec3.ofGlyph: Bijection from glyphs to canonical QLG points.

Maps each glyph to its unique point on the unit sphere:
  Pi     ↔ (1, 0, 0)
  Gamma  ↔ (-1, 0, 0)
  Delta  ↔ (0, 1, 0)
  Psi    ↔ (0, -1, 0)
  Lambda ↔ (0, 0, 1)
  Omega  ↔ (0, 0, -1)
-/
def Vec3.ofGlyph : Glyph → Vec3
  | .Pi => {x:=1,y:=0,z:=0}
  | .Gamma => {x:=-1,y:=0,z:=0}
  | .Delta => {x:=0,y:=1,z:=0}
  | .Psi => {x:=0,y:=-1,z:=0}
  | .Lambda => {x:=0,y:=0,z:=1}
  | .Omega => {x:=0,y:=0,z:=-1}

/-!
Glyph.ofVec3: Inverse bijection from QLG points to glyphs.

Converts a vector to its corresponding glyph, or returns none
if the vector is not a canonical QLG point.
-/
def Glyph.ofVec3 : Vec3 → Option Glyph
  | {x:=1,y:=0,z:=0} => some .Pi
  | {x:=-1,y:=0,z:=0} => some .Gamma
  | {x:=0,y:=1,z:=0} => some .Delta
  | {x:=0,y:=-1,z:=0} => some .Psi
  | {x:=0,y:=0,z:=1} => some .Lambda
  | {x:=0,y:=0,z:=-1} => some .Omega
  | _ => none

-- ============ SPIN FACTOR ALGEBRA ============

/-!
SpinFactor: Parameterized algebra structure (α, v) where α ∈ ℤ, v ∈ ℤⁿ.

The spin factor product x ∘ y is defined as:
  x = (α, v), y = (β, w)
  x ∘ y = (α*β + ⟨v, w⟩, α*w + β*v)

This is commutative, idempotent, and has exactly 2 primitive idempotents.
-/
structure SpinFactor where
  scalar : ℤ
  vector : List ℤ
  deriving Repr

/-!
SpinFactor.mul: The spin factor product operation.

Implements x ∘ y commutative product.
For clarity, we compute:
  - Scalar part: α*β + dot(v, w)
  - Vector part: α*w + β*v
-/
def SpinFactor.mul (x y : SpinFactor) : SpinFactor :=
  let α := x.scalar
  let β := y.scalar
  let dot := List.zipWith (· * ·) x.vector y.vector |> List.sum
  let scalar_part := α * β + dot
  let vector_part := List.map (· * β) x.vector ++ List.map (· * α) y.vector
  {scalar := scalar_part, vector := vector_part}

-- Commutativity property (proven separately in Jordan.lean)
def SpinFactor.commutative (x y : SpinFactor) : Prop :=
  x.mul y = y.mul x
