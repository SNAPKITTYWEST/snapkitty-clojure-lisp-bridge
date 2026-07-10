(ns doomsun.solarium.embedding.pipeline
  (:require [doomsun.solarium.embedding.model :as model]
            [promesa.core :as p]))

(defn- invoke-pipeline
  "Call the HF pipeline directly via JS invocation.
   The pipeline object shadows Function.prototype.call with undefined,
   so ClojureScript's (pipe args) syntax does not work."
  [pipe & args]
  (js/Reflect.apply pipe nil (to-array args)))

(defn embed-texts
  "Embed a sequence of texts, returning a vector of 768-dim vectors."
  [config texts]
  (p/let [pipe     (model/get-pipeline config)
          ^js output (invoke-pipeline pipe
                                      (clj->js texts)
                                      (clj->js {:pooling  "mean"
                                                :normalize true}))]
    (mapv (fn [i]
            (vec (.tolist ^js (aget output i))))
          (range (aget (.-dims output) 0)))))

(defn embed-single
  "Embed a single text string, returning a 768-dim vector."
  [config text]
  (p/let [pipe     (model/get-pipeline config)
          ^js output (invoke-pipeline pipe
                                      text
                                      (clj->js {:pooling  "mean"
                                                :normalize true}))]
    (vec (.tolist ^js (aget output 0)))))
