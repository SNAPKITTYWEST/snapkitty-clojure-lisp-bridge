(ns doomsun.solarium.qdrant.collections
  (:require [doomsun.solarium.qdrant.client :as client]
            [doomsun.solarium.util :as util]
            [promesa.core :as p]))

(defn collection-exists? [config collection-name]
  (p/let [resp (client/request config "GET" (str "/collections/" collection-name))]
    (= "ok" (:status resp))))

(defn create-collection! [config collection-name]
  (client/request config "PUT" (str "/collections/" collection-name)
                  {:vectors {:size     768
                             :distance "Cosine"}}))

(defn create-payload-indexes! [config collection-name]
  (p/do
    (client/request config "PUT"
                    (str "/collections/" collection-name "/index")
                    {:field_name "doc_id"
                     :field_schema "keyword"})
    (client/request config "PUT"
                    (str "/collections/" collection-name "/index")
                    {:field_name "chunk_index"
                     :field_schema "integer"})
    (client/request config "PUT"
                    (str "/collections/" collection-name "/index")
                    {:field_name "tags"
                     :field_schema "keyword"})))

(defn collection-info
  "Get detailed collection information including point counts and stats."
  [config]
  (p/let [resp (client/request config "GET"
                               (str "/collections/" (:collection-name config)))]
    (:result resp)))

(defn ensure-collection!
  "Create the collection and payload indexes if they don't exist."
  [config]
  (let [coll (:collection-name config)]
    (p/catch
     (p/do
       (collection-exists? config coll)
       (util/log (str "Collection '" coll "' already exists.")))
     (fn [_]
       (p/do
         (util/log (str "Creating collection '" coll "'..."))
         (create-collection! config coll)
         (create-payload-indexes! config coll)
         (util/log (str "Collection '" coll "' created with indexes.")))))))
