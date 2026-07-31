\ GRAVEYARD MAP — SNAPKITTYWEST/myapplication
\ 1 repos | rendered by AHMAD-BOT + Forth renderer
\ The graveyard in Forth. Every repo is a word.

\ ── myapplication (gravity: 0.4, status: broken) ──
: crawl-myapplication ( -- )
  0.4 gravity
  dup alive? IF
    ." myapplication alive " cr
  ELSE dup broken? IF
    ." myapplication broken " cr
    "myapplication" repair
  ELSE
    ." myapplication orphan " cr
    "myapplication" flag
  THEN THEN
  drop
;

: crawl-graveyard ( -- )
  ." === SNAPKITTYWEST/myapplication GRAVEYARD CRAWL ===" cr
  crawl-myapplication
  ." === CRAWL COMPLETE ===" cr
;

crawl-graveyard