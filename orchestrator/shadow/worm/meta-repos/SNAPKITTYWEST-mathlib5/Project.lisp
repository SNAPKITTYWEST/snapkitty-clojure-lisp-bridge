;;; Project.lisp — MATHLIB5 Sovereign Corpus Entry
;;; Corpus Record ID: MATHLIB5-20260710-MALICELA-001
;;; Injected by AGENTIC-ARENA ransom-worm
;;; Gravity: 1.0000 | Status: alive | Family: 9

(defproject mathlib5-malice-layer2
  :id       "MATHLIB5-20260710-MALICELA-001"
  :weight   1
  :gravity  1.0000
  :status   :alive
  :sovereign t
  :review-status :human_review_required

  :security-audit
  '((:metadata-integrity  "Yes")
    (:tamper-evidence      "SHA-256 chain (append-only)")
    (:encryption           "AES-256-GCM")
    (:plasma-gate          "Ed25519_Enforced"))

  :family-id 9

  :split
  '(:audit :kernel :proofs :runtime :witnesses
    :refinements :asp_gate :fol_engine :prism_skills)

  :worm-status :awaiting_worm_seal

  :provenance
  '((:chain-link   "Bifrost_WORM_Chain_20260710_01")
    (:source-path  "mathlib5/malice_layer2/")
    (:corpora-path "corpora/novel_methods/malice_refutation/"))

  :build-steps
  '((audit      . "run sovereign audit on malice_layer2 corpus")
    (kernel     . "verify kernel integrity via prism-skills SHA-256d")
    (proofs     . "run Lean 4 proof obligations against malice theorems")
    (runtime    . "execute runtime witnesses through sovereign-glue.rexx")
    (witnesses  . "generate UnifiedWitness for each split layer")
    (seal       . "WORM-seal all witnesses to Bifrost chain")
    (deploy     . "push to corpora/novel_methods/malice_refutation/"))

  :worm-hook
  '(:endpoint   "https://github.com/SNAPKITTYWEST/agentic-arena"
    :seal-on    :build-complete
    :chain      "Bifrost_WORM_Chain_20260710_01"
    :plasma-gate "Ed25519_Enforced")

  :created-by "Ahmad Ali Parr · SnapKitty Collective · the-49th-call SNAPKITTYWEST · 2026"
  :source-sha256 :auto_generated)
