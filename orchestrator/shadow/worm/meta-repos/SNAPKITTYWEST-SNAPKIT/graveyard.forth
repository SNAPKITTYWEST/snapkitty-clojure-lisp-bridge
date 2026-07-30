\ GRAVEYARD MAP — SNAPKITTYWEST/SNAPKIT
\ 1 repos | rendered by AHMAD-BOT + Forth renderer
\ The graveyard in Forth. Every repo is a word.

\ ── SNAPKIT (gravity: 0.4, status: broken) ──
: crawl-snapkit ( -- )
  0.4 gravity
  dup alive? IF
    ." SNAPKIT alive " cr
  ELSE dup broken? IF
    ." SNAPKIT broken " cr
    "SNAPKIT" repair
  ELSE
    ." SNAPKIT orphan " cr
    "SNAPKIT" flag
  THEN THEN
  drop
;

: crawl-graveyard ( -- )
  ." === SNAPKITTYWEST/SNAPKIT GRAVEYARD CRAWL ===" cr
  crawl-snapkit
  ." === CRAWL COMPLETE ===" cr
;

crawl-graveyard