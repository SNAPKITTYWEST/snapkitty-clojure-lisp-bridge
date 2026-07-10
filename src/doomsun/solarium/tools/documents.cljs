(ns doomsun.solarium.tools.documents
  (:require [clojure.string :as str]
            [doomsun.solarium.qdrant.points :as points]
            [doomsun.solarium.embedding.pipeline :as embed]
            [doomsun.solarium.chunking :as chunking]
            [doomsun.solarium.util :as util]
            [promesa.core :as p]))

(defn store-document!
  "Chunk, embed, and upsert a document into Qdrant."
  [config {:keys [title content tags source_url]}]
  (let [doc-id    (util/gen-uuid)
        now       (util/now-iso)
        raw-chunks (chunking/chunk-text config content)
        prefixed   (chunking/prefix-document-chunks raw-chunks)
        tags       (or tags [])]
    (p/let [vectors (embed/embed-texts config prefixed)
            points  (mapv (fn [i raw-chunk vector]
                            {:id      (util/uuid-v5 doc-id i)
                             :vector  vector
                             :payload {:doc_id      doc-id
                                       :chunk_index i
                                       :chunk_count (count raw-chunks)
                                       :content     raw-chunk
                                       :title       title
                                       :tags        tags
                                       :source_url  (or source_url nil)
                                       :created_at  now
                                       :updated_at  now}})
                          (range) raw-chunks vectors)
            _       (points/upsert! config points)]
      {:id     doc-id
       :title  title
       :chunks (count raw-chunks)
       :tags   tags})))

(defn read-document
  "Read all chunks for a document and concatenate content."
  [config id]
  (p/let [result (points/scroll config
                                :filter {:must [{:key   "doc_id"
                                                 :match {:value id}}]}
                                :limit 1000)
          pts    (sort-by #(get-in % [:payload :chunk_index]) (:points result))]
    (when (seq pts)
      (let [first-payload (:payload (first pts))]
        {:id         id
         :title      (:title first-payload)
         :tags       (:tags first-payload)
         :source_url (:source_url first-payload)
         :created_at (:created_at first-payload)
         :updated_at (:updated_at first-payload)
         :content    (str/join "\n\n" (map #(get-in % [:payload :content]) pts))}))))

(defn list-documents
  "List document metadata (chunk_index=0 points)."
  [config {:keys [limit offset]}]
  (p/let [result (points/scroll config
                                :filter {:must [{:key   "chunk_index"
                                                 :match {:value 0}}]}
                                :limit  (or limit 50)
                                :offset offset)
          docs   (mapv (fn [pt]
                         (let [p (:payload pt)]
                           {:id         (:doc_id p)
                            :title      (:title p)
                            :tags       (:tags p)
                            :chunks     (:chunk_count p)
                            :source_url (:source_url p)
                            :created_at (:created_at p)
                            :updated_at (:updated_at p)}))
                       (:points result))]
    {:documents docs
     :next_offset (:next_page_offset result)}))

(defn update-document!
  "Update a document. Re-chunks if content changed, otherwise metadata-only update."
  [config {:keys [id title content tags source_url]}]
  (let [doc-filter {:must [{:key "doc_id" :match {:value id}}]}
        now        (util/now-iso)]
    (if content
      ;; Content changed — delete old chunks and re-store
      (p/let [_       (points/delete-by-filter! config doc-filter)
              ;; Read existing metadata for merge
              raw-chunks (chunking/chunk-text config content)
              prefixed   (chunking/prefix-document-chunks raw-chunks)
              vectors    (embed/embed-texts config prefixed)
              points     (mapv (fn [i raw-chunk vector]
                                 {:id      (util/uuid-v5 id i)
                                  :vector  vector
                                  :payload {:doc_id      id
                                            :chunk_index i
                                            :chunk_count (count raw-chunks)
                                            :content     raw-chunk
                                            :title       (or title "Untitled")
                                            :tags        (or tags [])
                                            :source_url  source_url
                                            :created_at  now
                                            :updated_at  now}})
                               (range) raw-chunks vectors)
              _          (points/upsert! config points)]
        {:id     id
         :title  (or title "Untitled")
         :chunks (count raw-chunks)
         :tags   (or tags [])})
      ;; Metadata-only update
      (p/let [payload (cond-> {:updated_at now}
                        title      (assoc :title title)
                        tags       (assoc :tags tags)
                        source_url (assoc :source_url source_url))
              _       (points/set-payload! config doc-filter payload)]
        {:id      id
         :updated (keys payload)}))))

(defn delete-document!
  "Delete all chunks for a document."
  [config id]
  (p/let [_ (points/delete-by-filter! config {:must [{:key   "doc_id"
                                                      :match {:value id}}]})]
    {:id id :deleted true}))
