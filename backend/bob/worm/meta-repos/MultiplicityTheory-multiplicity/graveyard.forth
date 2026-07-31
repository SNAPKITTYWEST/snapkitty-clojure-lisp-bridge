\ GRAVEYARD MAP — MultiplicityTheory/multiplicity
\ 1 repos | rendered by AHMAD-BOT + Forth renderer
\ The graveyard in Forth. Every repo is a word.

\ ── multiplicity (gravity: 0.6000000000000001, status: alive) ──
: crawl-multiplicity ( -- )
  0.6000000000000001 gravity
  dup alive? IF
    ." multiplicity alive " cr
  ELSE dup broken? IF
    ." multiplicity broken " cr
    "multiplicity" repair
  ELSE
    ." multiplicity orphan " cr
    "multiplicity" flag
  THEN THEN
  drop
;

: crawl-graveyard ( -- )
  ." === MultiplicityTheory/multiplicity GRAVEYARD CRAWL ===" cr
  crawl-multiplicity
  ." === CRAWL COMPLETE ===" cr
;

crawl-graveyard