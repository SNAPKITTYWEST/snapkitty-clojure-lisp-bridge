(ns doomsun.solarium.dashboard.server
  (:require ["node:http" :as node-http]
            ["node:url" :as url]
            [doomsun.solarium.dashboard.http :as http]
            [doomsun.solarium.dashboard.routes :as routes]
            [doomsun.solarium.util :as util]
            [promesa.core :as p]))

(defn- parse-request [^js req]
  (let [parsed (url/URL. (.-url req) (str "http://" (or (aget (.-headers req) "host") "localhost")))]
    {:method   (.-method req)
     :pathname (.-pathname parsed)
     :search-params (.-searchParams parsed)}))

(defn start! [config]
  (p/create
   (fn [resolve _reject]
     (let [server (node-http/createServer
                   (fn [req res]
                     (let [parsed (parse-request req)]
                       (-> (p/let [_ (routes/handle config parsed res)])
                           (p/catch
                            (fn [err]
                              (util/log "Dashboard error:" (.-message err))
                              (http/respond-html res 500 "<h1>Internal Server Error</h1>")))))))]
       (.listen server (:dashboard-port config)
                (fn []
                  (util/log (str "Solarium dashboard: http://localhost:" (:dashboard-port config)))
                  (resolve server)))))))
