(ns doomsun.solarium.qdrant.client
  (:require [promesa.core :as p]))

(defn request
  "Send an HTTP request to the Qdrant REST API."
  [{:keys [qdrant-url qdrant-api-key]} method path & [body]]
  (p/let [opts    (cond-> {:method  method
                           :headers {"Content-Type" "application/json"}}
                    qdrant-api-key (assoc-in [:headers "api-key"] qdrant-api-key)
                    body           (assoc :body (js/JSON.stringify (clj->js body))))
          resp    (js/fetch (str qdrant-url path) (clj->js opts))
          ok?     (.-ok resp)
          json    (.json resp)]
    (let [result (js->clj json :keywordize-keys true)]
      (if ok?
        result
        (throw (ex-info (str "Qdrant error: " (:status result))
                        {:status (.-status resp)
                         :body   result}))))))
