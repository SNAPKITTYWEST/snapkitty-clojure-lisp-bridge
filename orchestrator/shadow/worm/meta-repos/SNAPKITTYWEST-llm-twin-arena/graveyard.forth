\ GRAVEYARD MAP — SNAPKITTYWEST/llm-twin-arena
\ Gravity: 0.8500 (alive — dual-model adversarial pair architecture)
\ Restored by: EDUALC | 2026-07-14
\ Architecture: Gemini (cloud generator) + Nemtron (Ollama gate)
\ The graveyard in Forth. Every repo is a word.

\ ── llm-twin-arena (gravity: 0.85, status: alive) ──
: crawl-llm-twin-arena ( -- )
  0.85 gravity
  dup alive? IF
    ." llm-twin-arena alive — dual-model sovereign gate" cr
  ELSE dup broken? IF
    ." llm-twin-arena broken " cr
    "llm-twin-arena" repair
  ELSE
    ." llm-twin-arena orphan " cr
    "llm-twin-arena" flag
  THEN THEN
  drop
;

\ ── Architecture words ──
: generator   ." [TWIN] Gemini cloud — produces artifacts, manifestos" cr ;
: gate        ." [TWIN] Nemtron Ollama — sovereign enforcer, zero-sorry" cr ;
: transport   ." [TWIN] Gmail relay — Ahmad→Ahmad, WORM audit trail" cr ;
: paradox     ." [TWIN] sorry self-audit paradox — model generates + flags own violation" cr ;
: novel       ." [TWIN] cross-model adversarial review — external gate, not self-audit" cr ;

\ ── Test modes ──
: adversarial_pair   ." [TWIN] generator + gate pair — measure rounds-to-constraint" cr ;
: constraint_measure ." [TWIN] metric: sorry_count + rounds_to_zero_sorry + fix_accuracy" cr ;
: bleed_test         ." [TWIN] context isolation test — inject in one tab, observe in other" cr ;

\ ── Pipeline ──
: twin_pipeline
  generator gate transport paradox novel
  adversarial_pair constraint_measure
  ." [TWIN] seal → test-lab/CASE_FILE_GEMINI_NEMTRON_BLEED.md" cr ;

: crawl-graveyard ( -- )
  ." === SNAPKITTYWEST/llm-twin-arena GRAVEYARD CRAWL ===" cr
  crawl-llm-twin-arena
  twin_pipeline
  ." === CRAWL COMPLETE — ALIVE ===" cr
;

crawl-graveyard
