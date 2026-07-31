\ GRAVEYARD MAP — SNAPKITTYWEST/saint-errant-digital-society
\ 1 repos | rendered by AHMAD-BOT + Forth renderer
\ The graveyard in Forth. Every repo is a word.

\ ── saint-errant-digital-society (gravity: 0.4, status: broken) ──
: crawl-saint-errant-digital-society ( -- )
  0.4 gravity
  dup alive? IF
    ." saint-errant-digital-society alive " cr
  ELSE dup broken? IF
    ." saint-errant-digital-society broken " cr
    "saint-errant-digital-society" repair
  ELSE
    ." saint-errant-digital-society orphan " cr
    "saint-errant-digital-society" flag
  THEN THEN
  drop
;

: crawl-graveyard ( -- )
  ." === SNAPKITTYWEST/saint-errant-digital-society GRAVEYARD CRAWL ===" cr
  crawl-saint-errant-digital-society
  ." === CRAWL COMPLETE ===" cr
;

crawl-graveyard