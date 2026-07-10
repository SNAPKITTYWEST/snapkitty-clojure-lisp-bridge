(ns doomsun.solarium.tools.search
  (:require [doomsun.solarium.qdrant.points :as points]
            [doomsun.solarium.embedding.pipeline :as embed]
            [doomsun.solarium.chunking :as chunking]
            [promesa.core :as p]))

(defn search-documents
  "Semantic search: embed query, search Qdrant, deduplicate by doc_id."
  [config {:keys [query limit]}]
  (let [limit    (or limit 10)
        prefixed (chunking/prefix-query query)]
    (p/let [vector  (embed/embed-single config prefixed)
            results (points/search config vector (* limit 3))
            ;; Deduplicate by doc_id, keep highest scoring chunk per doc
            deduped (vals (reduce (fn [acc pt]
                                    (let [doc-id (get-in pt [:payload :doc_id])]
                                      (if (contains? acc doc-id)
                                        acc
                                        (assoc acc doc-id pt))))
                                  {}
                                  results))
            ranked  (->> deduped
                         (sort-by :score >)
                         (take limit))]
      {:results (mapv (fn [pt]
                        {:doc_id  (get-in pt [:payload :doc_id])
                         :title   (get-in pt [:payload :title])
                         :tags    (get-in pt [:payload :tags])
                         :score   (:score pt)
                         :snippet (let [content (get-in pt [:payload :content])]
                                    (if (> (count content) 500)
                                      (str (subs content 0 500) "...")
                                      content))})
                      ranked)})))
