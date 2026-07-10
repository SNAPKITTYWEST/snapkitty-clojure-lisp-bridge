(ns doomsun.solarium.dashboard.sse
  (:require [doomsun.solarium.dashboard.http :as http]
            [doomsun.solarium.dashboard.html :as html]
            [doomsun.solarium.dashboard.views.overview :as overview]
            [doomsun.solarium.dashboard.views.random :as random]
            [doomsun.solarium.util :as util]
            [promesa.core :as p]))

(defonce ^:private clients (atom {}))

(defn- add-client! [channel ^js res]
  (http/respond-sse res)
  (swap! clients update channel (fnil conj #{}) res)
  (.on res "close"
       (fn []
         (swap! clients update channel disj res))))

(defn- broadcast! [channel event-name html-str]
  (let [data (-> html-str
                 (.replace (js/RegExp. "\n" "g") "\ndata: "))]
    (doseq [res (get @clients channel)]
      (try
        (.write res (str "event: " event-name "\ndata: " data "\n\n"))
        (catch :default _
          (swap! clients update channel disj res))))))

(defn handle-overview [config res]
  (add-client! :overview res))

(defn handle-documents [config res]
  (add-client! :documents res))

(defn handle-random [config res]
  (add-client! :random res))

(defn start-polling!
  "Start all SSE polling intervals."
  [config]
  ;; Overview stats — every 60s
  (js/setInterval
   (fn []
     (when (seq (get @clients :overview))
       (-> (p/let [fragment (overview/stats-fragment config)]
             (broadcast! :overview "morph" (html/render fragment)))
           (p/catch (fn [err]
                      (util/log "SSE poll error:" (.-message err)))))))
   60000)
  ;; Random document — every 10s
  (js/setInterval
   (fn []
     (when (seq (get @clients :random))
       (-> (p/let [fragment (random/random-doc-fragment config)]
             (when fragment
               (broadcast! :random "morph" (html/render fragment))))
           (p/catch (fn [err]
                      (util/log "SSE random error:" (.-message err)))))))
   10000))
