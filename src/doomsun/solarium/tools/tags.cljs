(ns doomsun.solarium.tools.tags
  (:require [doomsun.solarium.qdrant.points :as points]
            [promesa.core :as p]))

(defn list-tags
  "Aggregate all tags with counts from chunk_index=0 points."
  [config]
  (p/let [result (points/scroll config
                                :filter {:must [{:key   "chunk_index"
                                                 :match {:value 0}}]}
                                :limit 1000)
          tag-counts (reduce (fn [acc pt]
                               (reduce (fn [m tag]
                                         (update m tag (fnil inc 0)))
                                       acc
                                       (get-in pt [:payload :tags])))
                             {}
                             (:points result))]
    {:tags (mapv (fn [[tag count]]
                   {:tag tag :count count})
                 (sort-by val > tag-counts))}))

(defn tag-document!
  "Add tags to all chunks of a document."
  [config {:keys [id tags]}]
  (let [doc-filter {:must [{:key "doc_id" :match {:value id}}]}]
    ;; First get existing tags from any chunk
    (p/let [result    (points/scroll config
                                     :filter doc-filter
                                     :limit 1)
            existing  (get-in (first (:points result)) [:payload :tags] [])
            merged    (vec (distinct (concat existing tags)))
            _         (points/set-payload! config doc-filter {:tags merged})]
      {:id id :tags merged})))

(defn untag-document!
  "Remove tags from all chunks of a document."
  [config {:keys [id tags]}]
  (let [doc-filter {:must [{:key "doc_id" :match {:value id}}]}
        remove-set (set tags)]
    (p/let [result   (points/scroll config
                                    :filter doc-filter
                                    :limit 1)
            existing (get-in (first (:points result)) [:payload :tags] [])
            filtered (vec (remove remove-set existing))
            _        (points/set-payload! config doc-filter {:tags filtered})]
      {:id id :tags filtered})))
