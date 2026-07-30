#!/usr/bin/env python3
"""
Self-Modifying ASP → Lean 4 Proof Synthesizer
Converts partition number derivation trace to Lean 4 proof structure
"""

import json
import sys
from typing import Dict, List, Tuple

class LeanProofSynthesizer:
    def __init__(self):
        self.partition_values: Dict[int, int] = {}
        self.term_contributions: List[Tuple[int, int, int, int, int]] = []
        self.derivation_steps: List[Tuple[str, str, int, int]] = []

    def parse_clingo_output(self, json_data: str):
        """Parse Clingo JSON output"""
        try:
            data = json.loads(json_data)
            for model in data.get("Call", []):
                if "Witnesses" in model:
                    for atom in model["Witnesses"][0].get("Value", []):
                        self._parse_atom(atom)
        except json.JSONDecodeError:
            print("# Error: Invalid JSON from Clingo", file=sys.stderr)

    def _parse_atom(self, atom: str):
        """Parse individual Clingo atoms"""
        # partition(N, V)
        if atom.startswith("partition("):
            parts = atom[10:-1].split(",")
            if len(parts) == 2:
                try:
                    n = int(parts[0])
                    v = int(parts[1])
                    self.partition_values[n] = v
                except ValueError:
                    pass

        # term_contribution(N, K, G, S, V)
        elif atom.startswith("term_contribution("):
            parts = atom[18:-1].split(",")
            if len(parts) == 5:
                try:
                    n, k, g, s, v = map(int, parts)
                    self.term_contributions.append((n, k, g, s, v))
                except ValueError:
                    pass

        # derivation_step(Type, Desc, N, V)
        elif atom.startswith("derivation_step("):
            # Parse quoted strings carefully
            pass

    def generate_lean_header(self) -> str:
        """Generate Lean 4 file header"""
        return '''import Mathlib.Data.Nat.Basic
import Mathlib.Data.Finset.Basic
import Mathlib.Tactic.Omega
import Mathlib.Algebra.BigOperators.Basic

namespace PartitionProof

-- Partition function definition
def partition_num : ℕ → ℕ
  | 0 => 1
  | n + 1 => sorry  -- Will be defined by recurrence

-- Generalized pentagonal numbers: g_k = k(3k-1)/2
def gen_pentagonal (k : ℤ) : ℤ :=
  if k = 0 then 0 else k * (3 * k - 1) / 2

-- Sign function: (-1)^(k-1)
def sign_pentagonal (k : ℤ) : ℤ :=
  if k = 0 then 0 else if (k - 1) % 2 = 0 then 1 else -1

'''

    def generate_base_case(self) -> str:
        """Generate Lean 4 base case proof"""
        return '''-- Base Case: p(0) = 1
theorem partition_base : partition_num 0 = 1 := by
  unfold partition_num
  rfl

'''

    def generate_recurrence_steps(self) -> str:
        """Generate Lean 4 recurrence steps from derivation trace"""
        lines = []
        lines.append("-- Recurrence Steps")
        lines.append("theorem euler_pentagonal_recurrence (n : ℕ) (hn : n ≥ 1) :")
        lines.append("    partition_num n = ∑ k in Finset.range 100,")
        lines.append("      if k = 0 then 0")
        lines.append("      else ((-1 : ℤ) ^ (k - 1)).natAbs * partition_num ((n - gen_pentagonal k).natAbs) := by")
        lines.append("  intro n hn")
        lines.append("")

        # Generate `have` statements for each partition value computed
        for n in sorted(self.partition_values.keys()):
            if n == 0:
                continue
            v = self.partition_values[n]
            lines.append(f"  have h_{n} : partition_num {n} = {v} := by")

            # Group term contributions by N
            contributions = [tc for tc in self.term_contributions if tc[0] == n]
            if contributions:
                # Generate sum computation
                lines.append(f"    -- Sum of pentagonal terms for p({n})")
                for idx, (n_val, k, g, s, v_term) in enumerate(contributions):
                    sign_str = "+" if s > 0 else "-"
                    lines.append(f"    -- Term k={k}: g_k={g}, sign={sign_str}, p({n_val - g})={v_term}")
                lines.append(f"    sorry  -- Sum = {v}")
            else:
                lines.append(f"    sorry")
            lines.append("")

        lines.append("  sorry  -- Final proof by induction")
        return "\n".join(lines)

    def generate_verification(self) -> str:
        """Generate Lean 4 verification theorems"""
        lines = []
        lines.append("\n-- Verification Against Known Values")

        known = {
            0: 1, 1: 1, 2: 2, 3: 3, 4: 5, 5: 7,
            6: 11, 7: 15, 8: 22, 9: 30, 10: 42,
            15: 176, 20: 627
        }

        for n, expected in known.items():
            if n in self.partition_values:
                computed = self.partition_values[n]
                if computed == expected:
                    lines.append(f"-- ✓ p({n}) = {computed} (verified)")
                else:
                    lines.append(f"-- ✗ p({n}) computed as {computed}, expected {expected}")

        return "\n".join(lines)

    def generate_lean_footer(self) -> str:
        """Generate Lean 4 file footer"""
        return '''
end PartitionProof
'''

    def synthesize(self) -> str:
        """Synthesize complete Lean 4 proof file"""
        proof = self.generate_lean_header()
        proof += self.generate_base_case()
        proof += self.generate_recurrence_steps()
        proof += self.generate_verification()
        proof += self.generate_lean_footer()
        return proof


def main():
    """Main entry point"""
    synthesizer = LeanProofSynthesizer()

    # Read Clingo JSON from stdin
    clingo_json = sys.stdin.read()

    # Parse output
    synthesizer.parse_clingo_output(clingo_json)

    # Generate Lean 4 proof
    lean_proof = synthesizer.synthesize()

    # Output
    print(lean_proof)


if __name__ == "__main__":
    main()
