;; SKC-LISP: Layered Truth Maintenance System (LTMS) — Clojure Implementation
;; Knowledge layer: Data-oriented symbolic reasoning
;; Author: Ahmad Parr (ahmedparr93@gmail.com)
;; Integration: Snapkitty LISP Bridge knowledge engine

(ns snapkitty.ltms.ltms
  (:require
    [clojure.set :as set]))

;; ============================================================================
;; DATA STRUCTURES
;; ============================================================================

(defrecord Fact [value source timestamp confidence priority])
(defrecord Sense [gloss context-pred confidence])
(defrecord Concept [name senses])

;; Global knowledge base (mutable atoms for runtime updates)
(defonce facts (atom #{}))
(defonce rules (atom {}))           ;; {module => [rules]}
(defonce concepts (atom {}))        ;; {concept-name => Concept}
(defonce module-stats (atom {}))    ;; {module => {count, updated}}

;; ============================================================================
;; 1. CONFLICT RESOLUTION
%% When multiple facts claim the same value, pick the winner by:
%% Priority > Confidence > Source recency
;; ============================================================================

(defn resolve-conflict
  "Return the winning fact for a given value (highest priority, then confidence)."
  [value]
  (->> @facts
       (filter #(= (:value %) value))
       (sort-by (juxt (comp - :priority) (comp - :confidence)))
       first))

(defn get-belief
  "Query belief about a value: returns winning fact or nil."
  [value]
  (resolve-conflict value))

(defn get-confidence
  "Get confidence level for a value."
  [value]
  (when-let [fact (resolve-conflict value)]
    (:confidence fact)))

;; ============================================================================
;; 2. OUTDATED DETECTION
%% Exponential decay: confidence decays over time
%% Decay function: Conf(t) = Conf(0) * exp(-0.0001 * age_in_ms)
;; ============================================================================

(defn half-life-ms
  "Time for confidence to drop to 50%."
  []
  (/ (Math/log 2.0) 0.0001))  ;; ~6931 ms

(defn outdated?
  "Check if fact is outdated based on exponential decay."
  [{:keys [timestamp confidence]} now-ms]
  (let [age-ms (- now-ms timestamp)
        decayed (* confidence (Math/exp (* -0.0001 age-ms)))]
    (< decayed 0.15)))

(defn prune-outdated-facts!
  "Remove facts that have decayed below threshold."
  [now-ms]
  (let [outdated-facts (filter #(outdated? % now-ms) @facts)
        count-before (count @facts)]
    (doseq [f outdated-facts]
      (swap! facts disj f))
    {:pruned (- count-before (count @facts))
     :remaining (count @facts)}))

(defn decay-at-time
  "Compute confidence decay at a given time."
  [{:keys [timestamp confidence]} now-ms]
  (let [age-ms (- now-ms timestamp)]
    (* confidence (Math/exp (* -0.0001 age-ms)))))

;; ============================================================================
;; 3. AMBIGUOUS CONCEPTS
%% Concepts have multiple senses, each with a context predicate
;; ============================================================================

(defn register-concept!
  "Register a concept with multiple senses."
  [concept-name glosses]
  (let [senses (mapv (fn [gloss]
                       (->Sense gloss nil 1.0))
                     glosses)]
    (swap! concepts assoc concept-name (->Concept concept-name senses))))

(defn register-concept-with-preds!
  "Register a concept with sense predicates."
  [concept-name sense-specs]
  ;; sense-specs: [{:gloss "...", :context-pred (fn [ctx] ...), :confidence 0.9}]
  (let [senses (mapv (fn [{:keys [gloss context-pred confidence]}]
                       (->Sense gloss context-pred (or confidence 1.0)))
                     sense-specs)]
    (swap! concepts assoc concept-name (->Concept concept-name senses))))

(defn disambiguate
  "Find senses of a concept that match the context."
  [concept-name context]
  (when-let [c (@concepts concept-name)]
    (->> (:senses c)
         (filter (fn [{:keys [context-pred]}]
                   (if context-pred
                     (try (context-pred context) (catch js/Error _ false))
                     true)))
         (sort-by (comp - :confidence)))))

(defn best-sense
  "Get the highest-confidence sense for a concept in context."
  [concept-name context]
  (first (disambiguate concept-name context)))

;; ============================================================================
;; 4. MAINTAINABILITY GUARD
%% Hard limit: no module can have > 80 rules
%% Prevents knowledge explosion and maintains readable rule sets
;; ============================================================================

(defn add-rule!
  "Add a rule to a module (throws if limit exceeded)."
  [module head body priority]
  (let [module-rules (get @rules module [])
        count (count module-rules)]
    (if (>= count 80)
      (throw (ex-info (str "Module " module " exceeded rule limit (80)")
                      {:module module :count count}))
      (do
        (swap! rules update module conj {:head head :body body :priority priority})
        (update-module-stats! module)))))

(defn update-module-stats!
  "Update complexity stats for a module."
  [module]
  (let [rule-count (count (get @rules module []))]
    (swap! module-stats assoc module
           {:count rule-count
            :updated (System/currentTimeMillis)
            :complexity (double (/ rule-count 80.0))})))

(defn module-complexity
  "Get percentage of module's rule limit (0-100)."
  [module]
  (when-let [stats (get @module-stats module)]
    (* (:complexity stats) 100)))

(defn modules-needing-refactor
  "List modules that exceed 70% complexity."
  []
  (->> @module-stats
       (filter (fn [[_ {:keys [complexity]}]]
                 (> complexity 0.7)))
       (map (fn [[module {:keys [count complexity]}]]
              {:module module
               :rules count
               :percentage (* complexity 100)}))))

(defn module-report
  "Pretty-print module statistics."
  []
  (doseq [[module {:keys [count]}] (sort @module-stats)]
    (println (str module ": " count "/80 rules")))
  (when-let [refactor-needed (modules-needing-refactor)]
    (println "\nModules needing refactor (>70%):")
    (doseq [{:keys [module percentage]} refactor-needed]
      (println (str "  " module ": " (long percentage) "%")))))

;; ============================================================================
;; 5. HYBRID / NON-RULE KNOWLEDGE
%% Fallback to embeddings when symbolic reasoning fails
;; ============================================================================

(defn prove-symbolic
  "Attempt pure symbolic proof (stub: check rules)."
  [goal]
  ;; In production: traverse rule base and apply SLD resolution
  ;; For now: simple fact lookup
  (when-let [fact (resolve-conflict goal)]
    fact))

(defn embedding-retrieve
  "Fallback to embedding search (stub)."
  [goal]
  ;; In production: query Qdrant vector DB or WORM ledger
  ;; For now: return nil to trigger failure
  nil)

(defn hybrid-query
  "Query with fallback: symbolic → embedding → fail."
  [goal]
  (or (prove-symbolic goal)
      (embedding-retrieve goal)))

(defn hybrid-query-with-confidence
  "Query returning result + confidence."
  [goal]
  (if-let [symbolic-result (prove-symbolic goal)]
    {:result symbolic-result :confidence 1.0 :method :symbolic}
    (if-let [{:keys [result confidence]} (embedding-retrieve goal)]
      {:result result :confidence confidence :method :embedding}
      {:result nil :confidence 0.0 :method :failed})))

;; ============================================================================
;; ASSERTION & QUERY API
;; ============================================================================

(defn assert-fact!
  "Assert a new fact with current timestamp."
  [value source confidence priority]
  (let [now-ms (System/currentTimeMillis)
        fact (->Fact value source now-ms confidence priority)]
    (swap! facts conj fact)
    fact))

(defn query
  "Query belief about a value."
  [value]
  (get-belief value))

(defn all-facts
  "Get all facts (for inspection)."
  []
  @facts)

(defn facts-by-source
  "Get facts from a specific source."
  [source]
  (filter #(= (:source %) source) @facts))

(defn all-concepts
  "Get all registered concepts."
  []
  @concepts)

(defn concept-senses
  "Get all senses for a concept."
  [concept-name]
  (when-let [c (@concepts concept-name)]
    (:senses c)))

;; ============================================================================
;; SUBSUMPTION & TAXONOMIES
;; ============================================================================

(defn assert-subsumption!
  "Assert that A is-a B (taxonomic relationship)."
  [a b confidence]
  (assert-fact! {:is-a [a b]} "taxonomy" confidence 50))

(defn all-subsumptions
  "Get all is-a relationships."
  []
  (filter #(and (map? (:value %)) (:is-a (:value %))) @facts))

(defn ancestors
  "Get all ancestors of a concept."
  [concept]
  (loop [current concept visited #{}]
    (if (visited current)
      visited
      (let [visited' (conj visited current)
            parents (comp (map #(second (:is-a (:value %))))
                         (filter #(= (first (:is-a (:value %))) current)))]
        (reduce (fn [acc parent]
                  (set/union acc (ancestors parent)))
                visited'
                (parents @facts))))))

;; ============================================================================
;; INSPECTION & DEBUGGING
;; ============================================================================

(defn fact-summary
  "Pretty summary of a fact."
  [{:keys [value source confidence priority]}]
  (str "Fact: " value "\n"
       "  Source: " source "\n"
       "  Confidence: " (long (* confidence 100)) "%\n"
       "  Priority: " priority))

(defn list-facts
  "Print all facts."
  []
  (doseq [f (sort-by :priority @facts)]
    (println (fact-summary f))))

(defn list-rules-in-module
  "Print all rules in a module."
  [module]
  (doseq [{:keys [head body priority]} (get @rules module [])]
    (println (str "Rule (priority " priority "): " head " :- " body))))

;; ============================================================================
;; TESTS
;; ============================================================================

(defn run-ltms-tests []
  (println "=== LTMS Tests (Clojure) ===\n")

  ;; Test 1: Conflict resolution
  (println "Test 1: Conflict resolution")
  (assert-fact! :color-car :source-a 0.8 10)
  (assert-fact! :color-car :source-b 0.9 5)
  (if-let [winner (resolve-conflict :color-car)]
    (println "✓ Winner:" (:source winner) "with confidence" (:confidence winner))
    (println "✗ No winner found"))
  (println)

  ;; Test 2: Outdated detection
  (println "Test 2: Outdated detection")
  (let [now-ms (System/currentTimeMillis)
        old-time (- now-ms 10000)
        old-fact (->Fact :old-value :old-source old-time 0.5 1)]
    (if (outdated? old-fact now-ms)
      (println "✓ Old fact marked as outdated")
      (println "✗ Old fact not detected")))
  (println)

  ;; Test 3: Concepts & disambiguation
  (println "Test 3: Concepts & disambiguation")
  (register-concept! :bank ["financial institution" "river bank" "pile of snow"])
  (if-let [senses (concept-senses :bank)]
    (println "✓ Bank concept has" (count senses) "senses")
    (println "✗ Bank concept not found"))
  (println)

  ;; Test 4: Module complexity
  (println "Test 4: Module complexity guard")
  (add-rule! :main :test-rule :test-body 1)
  (if-let [complexity (module-complexity :main)]
    (println (str "✓ Main module: " (long complexity) "% full"))
    (println "✗ Module stats not available"))
  (println)

  ;; Test 5: Hybrid query
  (println "Test 5: Hybrid query with fallback")
  (assert-fact! :sky-color :observation 0.95 100)
  (if-let [result (hybrid-query-with-confidence :sky-color)]
    (println (str "✓ Query result: " (:result result) " via " (:method result)))
    (println "✗ Query failed"))
  (println)

  (println "=== LTMS Tests Complete ==="))
