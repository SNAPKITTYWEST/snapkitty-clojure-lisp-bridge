(ns snapkitty.lisp.integration.world
  "Unified LISP-Clojure world coordinator"
  (:require [snapkitty.lisp.bridge.reader :as reader]
            [snapkitty.lisp.bridge.compiler :as compiler]
            [snapkitty.lisp.knowledge.store :as store]))

(defn register-lisp-dialect [dialect-name dialect-config]
  "Register a LISP dialect (from lisp-machine, apple-ii, etc.)"
  {:dialect dialect-name
   :config dialect-config
   :timestamp (js/Date.now)})

(defn ingest-lisp-world [sources]
  "Ingest LISP code from multiple world sources"
  (doseq [{:keys [dialect path]} sources]
    (js/console.log (str "[world] Ingesting " dialect " from " path))))

(defn knowledge-bridge [lisp-code]
  "Convert LISP → knowledge graph → searchable store"
  (let [forms (reader/read-lisp lisp-code)
        compiled (mapv compiler/compile-form forms)
        knowledge-id (str "lisp_" (js/Date.now))]
    {:id knowledge-id
     :forms forms
     :compiled compiled
     :searchable (str/join " " forms)}))

;; === LISP-Clojure World Registry ===
(defonce world-registry (atom
  {:dialects {}
   :sources {}
   :unified-index {}}))

(defn register-world-source! [world-name {:keys [dialect path repo-link]}]
  "Register a source in the unified LISP world"
  (swap! world-registry update-in [:sources world-name]
         assoc :dialect dialect :path path :repo-link repo-link))

(defn list-world-sources []
  "List all registered LISP world sources"
  (:sources @world-registry))
