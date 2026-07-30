;; AGENTIC ARENA — Constitutional Contract
;; The repo topology format every crawler must produce.
;; Consumed by Prolog gravity.pl to reason about gaps.

(arena-constitution
  (version "1.0.0")
  (org "SNAPKITTYWEST")

  (agents
    (agent (id "AHMAD-BOT") (hat red)  (role scavenger) (target graveyard))
    (agent (id "EDUALC")    (hat blue) (role restorer)  (target graveyard))
    (agent (id "BOB")       (hat none) (role oracle)    (target worm-chain)))

  (repo-topology
    (meta
      (name    STRING)
      (url     STRING)
      (org     STRING)
      (gravity FLOAT)
      (hat     red|blue|purple)
      (crawl-ts INT))
    (clusters
      (cluster
        (id      STRING)
        (gravity FLOAT)
        (depth   INT)
        (status  alive|broken|orphan|gap)
        (members
          (node (path STRING) (lines INT) (deps INT) (depth INT)))))
    (gaps
      (gap
        (type    missing-wire|dead-page|local-only|github-only|no-tests|no-readme)
        (path    STRING)
        (gravity FLOAT)
        (fix     STRING)))
    (worm-ref SHA256))

  (gravity-formula
    ;; gravity = (lines * dep_fan_out * recursion_depth) / total_repo_lines
    ;; high gravity = dense, load-bearing code
    ;; low gravity = orphan, dead, or leaf node
    (formula "(* lines dep-fan-out recursion-depth) / total-lines")
    (threshold-alive  0.6)
    (threshold-broken 0.3)
    (threshold-orphan 0.1))

  (seal-rule
    ;; Only BOB may seal a crawl result to the WORM chain.
    ;; Ahmad-Bot and EDUALC submit — BOB approves and seals.
    (submitters (agent-id "AHMAD-BOT") (agent-id "EDUALC"))
    (sealer     (agent-id "BOB"))))
