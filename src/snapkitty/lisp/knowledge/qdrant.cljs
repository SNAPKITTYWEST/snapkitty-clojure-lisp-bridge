(ns snapkitty.lisp.knowledge.qdrant
  "Qdrant vector database client — ENFORCED auth"
  (:require [promesa.core :as p]
            ["node-fetch" :default fetch]))

(defn http-request [cfg method path {:keys [body]}]
  "Make authenticated HTTP request to Qdrant — REQUIRES API KEY"
  (let [url (str (:qdrant-url cfg) path)
        headers {"Content-Type" "application/json"
                 "api-key" (:qdrant-key cfg)}
        options (clj->js (merge {:method method
                                 :headers headers}
                                (when body {:body (js/JSON.stringify body)})))]
    (p/let [response (fetch url options)]
      (if (.-ok response)
        (.json response)
        (throw (ex-info (str "Qdrant error: " (.-status response))
                        {:status (.-status response)}))))))

(defn ensure-collection! [cfg]
  "Create collection if it doesn't exist"
  (let [coll-name (:collection-name cfg)]
    (p/let [_ (http-request cfg "PUT"
                            (str "/collections/" coll-name)
                            {:body {:vectors {:size 384
                                             :distance "Cosine"}}})]
      (js/console.log (str "[qdrant] Collection '" coll-name "' ready")))))

(defn upsert-point! [cfg coll-id vector {:keys [id payload]}]
  "Insert/update vector with metadata"
  (http-request cfg "PUT"
                (str "/collections/" coll-id "/points")
                {:body {:points [{:id id
                                 :vector vector
                                 :payload payload}]}}))

(defn search-points [cfg coll-id query-vector {:keys [limit filter]}]
  "Search for nearest vectors"
  (http-request cfg "POST"
                (str "/collections/" coll-id "/points/search")
                {:body {:vector query-vector
                       :limit (or limit 10)
                       :filter filter}}))

(defn delete-point! [cfg coll-id point-id]
  "Delete a point by ID"
  (http-request cfg "POST"
                (str "/collections/" coll-id "/points/delete")
                {:body {:points_selector {:ids [point-id]}}}))
