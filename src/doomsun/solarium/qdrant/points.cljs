(ns doomsun.solarium.qdrant.points
  (:require [doomsun.solarium.qdrant.client :as client]
            [promesa.core :as p]))

(defn upsert!
  "Upsert points into the collection."
  [config points]
  (client/request config "PUT"
                  (str "/collections/" (:collection-name config) "/points")
                  {:points points}))

(defn search
  "Vector similarity search with optional filter."
  [config vector limit & [filter-clause]]
  (p/let [body (cond-> {:vector vector
                        :limit  limit
                        :with_payload true}
                 filter-clause (assoc :filter filter-clause))
          resp (client/request config "POST"
                               (str "/collections/" (:collection-name config) "/points/search")
                               body)]
    (:result resp)))

(defn scroll
  "Scroll through points with optional filter."
  [config & {:keys [filter limit offset with-payload]
             :or   {limit 100 with-payload true}}]
  (p/let [body (cond-> {:limit        limit
                        :with_payload with-payload}
                 filter (assoc :filter filter)
                 offset (assoc :offset offset))
          resp (client/request config "POST"
                               (str "/collections/" (:collection-name config) "/points/scroll")
                               body)]
    (:result resp)))

(defn delete-by-filter!
  "Delete points matching a filter."
  [config filter-clause]
  (client/request config "POST"
                  (str "/collections/" (:collection-name config) "/points/delete")
                  {:filter filter-clause}))

(defn set-payload!
  "Set payload fields on points matching a filter."
  [config filter-clause payload]
  (client/request config "POST"
                  (str "/collections/" (:collection-name config) "/points/payload")
                  {:payload payload
                   :filter  filter-clause}))
