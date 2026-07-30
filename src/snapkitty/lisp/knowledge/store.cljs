(ns snapkitty.lisp.knowledge.store
  "Document store with rate limiting"
  (:require [promesa.core :as p]
            [snapkitty.lisp.knowledge.qdrant :as qdrant]
            [snapkitty.lisp.knowledge.embedding :as embedding]
            [snapkitty.lisp.mcp.util :as util]))

;; Rate limiter: 10 docs/sec (token bucket)
(defonce rate-limiter (atom {:capacity 10 :tokens 10 :last-refill (js/Date.now)}))

(defn check-rate-limit! []
  "Check and update rate limit — token bucket"
  (let [now (js/Date.now)
        {:keys [tokens capacity last-refill]} @rate-limiter
        elapsed (/ (- now last-refill) 1000)
        refilled (min capacity (+ tokens (* elapsed 10)))]

    (if (< refilled 1)
      (throw (ex-info "Rate limit exceeded" {:retry-after 0.1}))
      (do
        (swap! rate-limiter assoc
               :tokens (dec refilled)
               :last-refill now)
        true))))

(defn upsert! [id doc-data]
  "Store document with embedding — RATE LIMITED"
  (check-rate-limit!)
  (p/let [vector (embedding/embed (:content doc-data))
          _ (qdrant/upsert-point! {}
                                  "snapkitty-knowledge"
                                  id
                                  {:vector vector
                                   :payload doc-data})]
    {:id id :stored true}))

(defn search [query {:keys [limit tags] :or {limit 10}}]
  "Semantic search — tags optional filter"
  (p/let [query-vector (embedding/embed query)
          results (qdrant/search-points {} "snapkitty-knowledge" query-vector
                                        {:limit limit})]
    results))

(defn delete! [id]
  "Delete document by ID"
  (qdrant/delete-point! {} "snapkitty-knowledge" id))
