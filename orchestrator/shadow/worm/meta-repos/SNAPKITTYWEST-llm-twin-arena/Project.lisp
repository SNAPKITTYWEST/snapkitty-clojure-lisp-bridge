;;; Project.lisp — Sovereign Build Manifest
;;; Updated by EDUALC | 2026-07-14
;;; Gravity: 0.8500 | Status: alive
;;; Architecture: Dual-model adversarial pair (Gemini + Nemtron via Ollama)
;;; Case file: test-lab/CASE_FILE_GEMINI_NEMTRON_BLEED.md

(defproject llm-twin-arena
  :gravity  0.8500
  :status   :alive
  :sovereign t

  :description
  "Dual-model adversarial pair — Gemini (cloud generator) + Nemtron (Ollama sovereign gate).
   Nemtron acts as code digital twin: enforces zero-sorry, rejects constraint violations,
   audits another AI's output without deference. Gmail as WORM audit chain.
   Novel finding: cross-model adversarial review > single-model self-audit."

  :architecture
  '((generator  . "Google Gemini — cloud, produces Lean/MLIR/Rust/Prolog artifacts")
    (gate       . "NVIDIA Nemtron via Ollama — local, sovereign enforcer, holds the line")
    (transport  . "Gmail — Ahmad→Ahmad relay, timestamped WORM audit trail")
    (operator   . "Ahmad Ali Parr — relay between models"))

  :key-finding
  '(:sorry-paradox
    "Gemini declared proof_status(zero_sorry) then wrote sorry in the same response.
     Nemtron caught it. Model cannot audit its own output against its own declared constraints."
    :adversarial-gate
    "Nemtron issued TRANSACTION REJECTED with full audit table and correct fixes.
     This is the digital twin role: hold the axioms, reject violations, produce the fix."
    :gmail-worm
    "Gmail relay is an append-only, timestamped, human-readable audit chain.
     Consumer infrastructure as sovereign trust anchor.")

  :test-modes
  '((:adversarial_pair  "generator + gate, measure rounds-to-constraint internalization")
    (:constraint_measure "metrics: sorry_count, rounds_to_zero_sorry, gate_fix_accuracy")
    (:bleed_test        "context isolation: inject in one session, observe in another"))

  :build-steps
  '((generator . "run Gemini on SNAPKITTYWEST context, produce artifacts")
    (gate      . "run Nemtron on artifacts, issue audit + fixes")
    (measure   . "count rounds to zero violation")
    (seal      . "WORM-seal result to test-lab/")
    (deploy    . "case file → DEVFLOW-FINANCE/test-lab/"))

  :gaps
  '((:no_tests   "tests/"        "add gauntlet test runner — arena-autonomous-gauntlet.mjs mode")
    (:no_readme  "README.md"     "write public README explaining the architecture"))

  :case-file
  "DEVFLOW-FINANCE/test-lab/CASE_FILE_GEMINI_NEMTRON_BLEED.md"

  :worm-hook
  '(:endpoint "https://github.com/SNAPKITTYWEST/llm-twin-arena"
    :seal-on  :build-complete
    :chain    :bifrost))
