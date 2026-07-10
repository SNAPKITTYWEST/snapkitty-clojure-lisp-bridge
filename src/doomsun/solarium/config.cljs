(ns doomsun.solarium.config)

(defn load-config []
  (let [env js/process.env]
    {:qdrant-url      (or (.-QDRANT_URL env) "http://localhost:6333")
     :qdrant-api-key  (.-QDRANT_API_KEY env)
     :collection-name (or (.-COLLECTION_NAME env) "knowledge")
     :model-name      (or (.-MODEL_NAME env) "nomic-ai/nomic-embed-text-v1.5")
     :chunk-max-chars (js/parseInt (or (.-CHUNK_MAX_CHARS env) "1600"))
     :chunk-overlap   (js/parseInt (or (.-CHUNK_OVERLAP env) "200"))
     :dashboard-port  (js/parseInt (or (.-DASHBOARD_PORT env) "3333"))}))
