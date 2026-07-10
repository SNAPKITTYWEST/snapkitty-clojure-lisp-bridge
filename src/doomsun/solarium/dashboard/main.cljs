(ns doomsun.solarium.dashboard.main
  (:require [doomsun.solarium.config :as config]
            [doomsun.solarium.qdrant.collections :as collections]
            [doomsun.solarium.dashboard.server :as server]
            [doomsun.solarium.dashboard.sse :as sse]
            [doomsun.solarium.util :as util]
            [promesa.core :as p]))

(defn main! []
  (let [cfg (config/load-config)]
    (p/let [_ (collections/ensure-collection! cfg)
            _ (server/start! cfg)]
      (sse/start-polling! cfg)
      (util/log "Dashboard ready."))))
