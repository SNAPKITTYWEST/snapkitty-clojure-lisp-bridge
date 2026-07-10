(ns doomsun.solarium.util
  (:require ["crypto" :as crypto]))

(defn gen-uuid []
  (str (random-uuid)))

(defn uuid-v5
  "Deterministic UUID from doc-id + chunk-index using SHA-1 hash.
   Not strict RFC 4122 v5, but deterministic and collision-free for our use."
  [doc-id chunk-index]
  (let [input (str doc-id ":" chunk-index)
        hash  (.digest (.update (.createHash crypto "sha1") input) "hex")]
    (str (subs hash 0 8) "-"
         (subs hash 8 12) "-"
         "5" (subs hash 13 16) "-"
         (subs hash 16 20) "-"
         (subs hash 20 32))))

(defn now-iso []
  (.toISOString (js/Date.)))

(defn log [& args]
  (apply js/console.error args))

(defn tool-result [data]
  (clj->js {:content [{:type "text"
                       :text (js/JSON.stringify (clj->js data) nil 2)}]}))

(defn tool-error [message]
  (clj->js {:content [{:type "text" :text message}]
            :isError true}))
