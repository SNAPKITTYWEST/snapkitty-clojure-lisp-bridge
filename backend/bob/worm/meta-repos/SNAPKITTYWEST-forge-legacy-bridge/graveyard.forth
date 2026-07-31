\ GRAVEYARD MAP — SNAPKITTYWEST/forge-legacy-bridge
\ 1 repos | rendered by AHMAD-BOT + Forth renderer
\ The graveyard in Forth. Every repo is a word.

\ ── forge-legacy-bridge (gravity: 0.2, status: orphan) ──
: crawl-forge-legacy-bridge ( -- )
  0.2 gravity
  dup alive? IF
    ." forge-legacy-bridge alive " cr
  ELSE dup broken? IF
    ." forge-legacy-bridge broken " cr
    "forge-legacy-bridge" repair
  ELSE
    ." forge-legacy-bridge orphan " cr
    "forge-legacy-bridge" flag
  THEN THEN
  drop
;

: crawl-graveyard ( -- )
  ." === SNAPKITTYWEST/forge-legacy-bridge GRAVEYARD CRAWL ===" cr
  crawl-forge-legacy-bridge
  ." === CRAWL COMPLETE ===" cr
;

crawl-graveyard