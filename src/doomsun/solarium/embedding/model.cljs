(ns doomsun.solarium.embedding.model
  (:require ["@huggingface/transformers" :refer [pipeline env]]
            [doomsun.solarium.util :as util]
            [promesa.core :as p]))

;; Disable remote model downloads if already cached
;; (set! (.-allowRemoteModels env) false)

(defonce ^:private pipeline-atom (atom nil))

(defn get-pipeline
  "Return the embedding pipeline, loading the model on first call."
  [config]
  (if-let [p @pipeline-atom]
    (p/resolved p)
    (p/let [_    (util/log "Loading embedding model:" (:model-name config))
            pipe (pipeline "feature-extraction"
                           (:model-name config)
                           (clj->js {:dtype "fp32"}))]
      (util/log "Embedding model loaded.")
      (reset! pipeline-atom pipe)
      pipe)))
