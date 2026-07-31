\ GRAVEYARD MAP — SNAPKITTYWEST/operation-infinite-matrix
\ 1 repos | rendered by AHMAD-BOT + Forth renderer
\ The graveyard in Forth. Every repo is a word.

\ ── operation-infinite-matrix (gravity: 0.2, status: orphan) ──
: crawl-operation-infinite-matrix ( -- )
  0.2 gravity
  dup alive? IF
    ." operation-infinite-matrix alive " cr
  ELSE dup broken? IF
    ." operation-infinite-matrix broken " cr
    "operation-infinite-matrix" repair
  ELSE
    ." operation-infinite-matrix orphan " cr
    "operation-infinite-matrix" flag
  THEN THEN
  drop
;

: crawl-graveyard ( -- )
  ." === SNAPKITTYWEST/operation-infinite-matrix GRAVEYARD CRAWL ===" cr
  crawl-operation-infinite-matrix
  ." === CRAWL COMPLETE ===" cr
;

crawl-graveyard