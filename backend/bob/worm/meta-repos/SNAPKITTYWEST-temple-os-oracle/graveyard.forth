\ GRAVEYARD MAP — SNAPKITTYWEST/temple-os-oracle
\ 1 repos | rendered by AHMAD-BOT + Forth renderer
\ The graveyard in Forth. Every repo is a word.

\ ── temple-os-oracle (gravity: 0, status: orphan) ──
: crawl-temple-os-oracle ( -- )
  0 gravity
  dup alive? IF
    ." temple-os-oracle alive " cr
  ELSE dup broken? IF
    ." temple-os-oracle broken " cr
    "temple-os-oracle" repair
  ELSE
    ." temple-os-oracle orphan " cr
    "temple-os-oracle" flag
  THEN THEN
  drop
;

: crawl-graveyard ( -- )
  ." === SNAPKITTYWEST/temple-os-oracle GRAVEYARD CRAWL ===" cr
  crawl-temple-os-oracle
  ." === CRAWL COMPLETE ===" cr
;

crawl-graveyard