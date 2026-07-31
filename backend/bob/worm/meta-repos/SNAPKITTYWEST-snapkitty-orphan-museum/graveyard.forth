\ GRAVEYARD MAP — SNAPKITTYWEST/snapkitty-orphan-museum
\ 1 repos | rendered by AHMAD-BOT + Forth renderer
\ The graveyard in Forth. Every repo is a word.

\ ── snapkitty-orphan-museum (gravity: 0.2, status: orphan) ──
: crawl-snapkitty-orphan-museum ( -- )
  0.2 gravity
  dup alive? IF
    ." snapkitty-orphan-museum alive " cr
  ELSE dup broken? IF
    ." snapkitty-orphan-museum broken " cr
    "snapkitty-orphan-museum" repair
  ELSE
    ." snapkitty-orphan-museum orphan " cr
    "snapkitty-orphan-museum" flag
  THEN THEN
  drop
;

: crawl-graveyard ( -- )
  ." === SNAPKITTYWEST/snapkitty-orphan-museum GRAVEYARD CRAWL ===" cr
  crawl-snapkitty-orphan-museum
  ." === CRAWL COMPLETE ===" cr
;

crawl-graveyard