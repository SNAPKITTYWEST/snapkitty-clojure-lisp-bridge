;; PH2.S4 — Mutation Replay & Rollback
;; From SKC-LISP-WORLD-COQ-001 <step name="Phase 9" label="RoundTrip">

(ns snapkitty.lisp.mutation.replay
  "Deterministic replay and transactional rollback")

(defn replay-mutations [base-world mutations]
  "Deterministic replay from base world through all mutations"
  (reduce
    (fn [world evt]
      (if (not= (:generation-after evt) (inc (:world-generation world)))
        (throw (ex-info "Generation mismatch" {:event evt :world world})))
      (-> world
          (update :world-generation inc)
          (update :mutation-log conj evt)))
    base-world
    mutations))

(defn rollback-to-generation [current-world target-generation]
  "Restore to prior generation (must be in checkpoint store)"
  (if (>= target-generation (:world-generation current-world))
    (throw (ex-info "Cannot roll forward" {:target target-generation})))
  {:world-generation target-generation
   :mutation-log (filterv #(< (:generation-after %) target-generation)
                          (:mutation-log current-world))})

(defn validate-replay [base mutations expected-result]
  "Verify replay determinism"
  (let [replayed (replay-mutations base mutations)]
    (= (:world-generation replayed) (:world-generation expected-result))))
