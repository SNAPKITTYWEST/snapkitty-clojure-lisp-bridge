\ GRAVEYARD MAP — SNAPKITTYWEST/snapkitty-hacker-kits-episode-1-broken-ledger
\ 1 repos | rendered by AHMAD-BOT + Forth renderer
\ The graveyard in Forth. Every repo is a word.

\ ── snapkitty-hacker-kits-episode-1-broken-ledger (gravity: 0.2, status: orphan) ──
: crawl-snapkitty-hacker-kits-episode-1-broken-ledger ( -- )
  0.2 gravity
  dup alive? IF
    ." snapkitty-hacker-kits-episode-1-broken-ledger alive " cr
  ELSE dup broken? IF
    ." snapkitty-hacker-kits-episode-1-broken-ledger broken " cr
    "snapkitty-hacker-kits-episode-1-broken-ledger" repair
  ELSE
    ." snapkitty-hacker-kits-episode-1-broken-ledger orphan " cr
    "snapkitty-hacker-kits-episode-1-broken-ledger" flag
  THEN THEN
  drop
;

: crawl-graveyard ( -- )
  ." === SNAPKITTYWEST/snapkitty-hacker-kits-episode-1-broken-ledger GRAVEYARD CRAWL ===" cr
  crawl-snapkitty-hacker-kits-episode-1-broken-ledger
  ." === CRAWL COMPLETE ===" cr
;

crawl-graveyard