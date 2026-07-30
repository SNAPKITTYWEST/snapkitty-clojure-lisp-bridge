;; PH2.S2 — Mutation Journal
;; From SKC-LISP-WORLD-COQ-001 <mutation-event>

(ns snapkitty.lisp.mutation.journal
  "Typed, validated, journaled mutations")

(defrecord MutationEvent
  [mutation-id
   generation-before
   generation-after
   actor
   target
   operation
   old-digest
   new-digest
   precondition
   proof-receipt
   timestamp-policy])

(defonce mutation-counter (atom 0))

(defn next-mutation-id []
  (swap! mutation-counter inc))

(defn create-mutation-event
  [generation-before generation-after target operation old-digest new-digest]
  (->MutationEvent
    (next-mutation-id)
    generation-before
    generation-after
    0                    ; actor
    target
    operation
    old-digest
    new-digest
    ""                   ; precondition
    nil                  ; proof-receipt
    0))                  ; timestamp-policy

(defn mutation-gate-passed? [evt]
  (and (< (:generation-before evt) (:generation-after evt))
       (not= (:old-digest evt) (:new-digest evt))
       (not= (:operation evt) "")))

(defn journal-append [journal evt]
  "Append-only mutation log"
  (if (mutation-gate-passed? evt)
    (conj journal evt)
    (throw (ex-info "Mutation gate failed" {:event evt}))))
