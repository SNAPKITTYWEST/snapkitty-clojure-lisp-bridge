\ MATHLIB5 MALICE LAYER — Dependency Graph
\ SNAPKITTYWEST/mathlib5/malice_layer2
\ Gravity: 1.0000 (alive, load-bearing)

: audit       ." [MALICE] run sovereign audit" cr ;
: kernel      ." [MALICE] prism-skills SHA-256d verification" cr ;
: proofs      ." [MALICE] Lean 4 malice refutation theorems" cr ;
: runtime     ." [MALICE] sovereign-glue.rexx execution" cr ;
: witnesses   ." [MALICE] UnifiedWitness generation per split" cr ;
: asp_gate    ." [MALICE] carto-gate.lp constitutional check" cr ;
: fol_engine  ." [MALICE] first-order logic engine pass" cr ;
: prism_skills ." [MALICE] prism canonical + SHA-256d + psi-pipeline" cr ;
: seal        ." [MALICE] WORM seal → Bifrost_WORM_Chain_20260710_01" cr ;

: malice_pipeline
  audit kernel proofs runtime witnesses asp_gate fol_engine prism_skills seal ;

malice_pipeline
