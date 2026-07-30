\ GRAVEYARD MAP — PhaseMirror/Foundry
\ Created 2026-07-10 | Age: 1 day | Gravity: 0.1000 (orphan)
\ Crawled by AHMAD-BOT 2026-07-11

: crawl-foundry ( -- )
  0.1000 gravity
  dup orphan? IF
    ." PhaseMirror/Foundry — orphan (1 day old)" cr
    ." Detected: lean/SNAPKITTY/ namespace — not authorized" cr
    ." Detected: ADR-060 names SnapKitty as sub-component — false" cr
    ." Detected: Rotate = no-op in evaluator.rs" cr
    ." Detected: 13 sorrys, ALP directory missing" cr
    ." Restoration: contact collectivekitty.com" cr
    "PhaseMirror-Foundry" flag
  THEN
  drop
;

crawl-foundry
