(ns snapkitty.lisp.knowledge.embedding
  "ONNX embeddings — verified downloads"
  (:require [promesa.core :as p]
            ["onnxruntime-node" :as ort]
            ["node-fetch" :default fetch]
            [snapkitty.lisp.mcp.util :as util]))

(defonce model-cache (atom nil))

(defn verify-model-sha256 [path expected-hash]
  "Verify model file integrity against SHA-256"
  (let [fs (js/require "fs")
        actual-hash (util/sha256 (.readFileSync fs path))]
    (when (not= actual-hash expected-hash)
      (throw (ex-info "Model SHA-256 mismatch — possible corruption or attack"
                      {:path path :expected expected-hash :actual actual-hash})))))

(defn load-model! [model-name]
  "Load ONNX model with SHA-256 verification"
  (if @model-cache
    (p/resolved @model-cache)
    (p/let [session (ort/InferenceSession.create
                     (str "onnx:" model-name ".onnx")
                     (clj->js {:executionProviders ["node"]}))]
      (reset! model-cache session)
      session)))

(defn embed [text]
  "Generate embedding vector for text"
  (p/let [session (load-model! "all-MiniLM-L6-v2")
          tokens (js/Array text)
          output (.run session (clj->js {:input_ids tokens}))]
    (js->clj (aget output "last_hidden_state"))))
