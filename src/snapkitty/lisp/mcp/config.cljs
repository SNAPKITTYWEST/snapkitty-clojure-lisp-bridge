(ns snapkitty.lisp.mcp.config
  "Configuration loader — enforces required env vars"
  (:require [clojure.string :as str]))

(defn load-config []
  "Load config from environment, fail fast if required vars missing"
  (let [qdrant-url (or (js/process.env.QDRANT_URL) "http://localhost:6333")
        qdrant-key (js/process.env.QDRANT_API_KEY)
        collection-name (or (js/process.env.COLLECTION_NAME) "snapkitty-knowledge")
        model-name (or (js/process.env.MODEL_NAME) "Xenova/all-MiniLM-L6-v2")
        chunk-max (or (some-> (js/process.env.CHUNK_MAX_CHARS) js/parseInt) 500)
        chunk-overlap (or (some-> (js/process.env.CHUNK_OVERLAP) js/parseInt) 100)]

    ;; CRITICAL: Qdrant API key is required
    (when (str/blank? qdrant-key)
      (throw (ex-info "Missing QDRANT_API_KEY environment variable — cannot connect securely"
                      {:config-error true})))

    {:qdrant-url qdrant-url
     :qdrant-key qdrant-key
     :collection-name collection-name
     :model-name model-name
     :chunk-max chunk-max
     :chunk-overlap chunk-overlap}))
