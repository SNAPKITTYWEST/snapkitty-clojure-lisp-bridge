\ GRAVEYARD MAP — SNAPKITTYWEST/sovereign-attestation-protocol
\ 1 repos | rendered by AHMAD-BOT + Forth renderer
\ The graveyard in Forth. Every repo is a word.

\ ── sovereign-attestation-protocol (gravity: 0, status: orphan) ──
: crawl-sovereign-attestation-protocol ( -- )
  0 gravity
  dup alive? IF
    ." sovereign-attestation-protocol alive " cr
  ELSE dup broken? IF
    ." sovereign-attestation-protocol broken " cr
    "sovereign-attestation-protocol" repair
  ELSE
    ." sovereign-attestation-protocol orphan " cr
    "sovereign-attestation-protocol" flag
  THEN THEN
  drop
;

: crawl-graveyard ( -- )
  ." === SNAPKITTYWEST/sovereign-attestation-protocol GRAVEYARD CRAWL ===" cr
  crawl-sovereign-attestation-protocol
  ." === CRAWL COMPLETE ===" cr
;

crawl-graveyard