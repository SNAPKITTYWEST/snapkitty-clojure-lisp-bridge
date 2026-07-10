(ns doomsun.solarium.dashboard.views.random
  (:require [clojure.string :as str]
            ["marked" :refer [marked]]
            [doomsun.solarium.dashboard.views.layout :as layout]
            [doomsun.solarium.dashboard.html :as html]
            [doomsun.solarium.qdrant.points :as points]
            [promesa.core :as p]))

(defn- deduplicate-chunks [chunks]
  (if (<= (count chunks) 1)
    (first chunks)
    (loop [result (first chunks)
           remaining (rest chunks)]
      (if (empty? remaining)
        result
        (let [next-chunk (first remaining)
              max-check (min (count result) (count next-chunk))
              overlap (loop [len max-check]
                        (if (zero? len) 0
                            (if (= (subs result (- (count result) len)) (subs next-chunk 0 len))
                              len (recur (dec len)))))]
          (recur (str result (subs next-chunk overlap))
                 (rest remaining)))))))

(defn random-doc-fragment
  "Fetch a random document and render it as an HTML fragment."
  [config]
  (p/let [;; Get all doc-level points
          result (points/scroll config
                                :filter {:must [{:key "chunk_index" :match {:value 0}}]}
                                :limit 1000)
          docs (:points result)]
    (when (seq docs)
      (let [pick (rand-nth docs)
            doc-id (get-in pick [:payload :doc_id])
            meta (:payload pick)]
        (p/let [all-chunks (points/scroll config
                                          :filter {:must [{:key "doc_id" :match {:value doc-id}}]}
                                          :limit 1000)
                pts (sort-by #(get-in % [:payload :chunk_index]) (:points all-chunks))
                content (deduplicate-chunks (map #(get-in % [:payload :content]) pts))]
          [:div
           [:div {:class "flex items-baseline justify-between mb-6"}
            [:h2 {:class "text-2xl"}
             [:a {:href (str "/documents/" doc-id)} (or (:title meta) "Untitled")]]
            [:span {:style "font-family: var(--font-mono); font-size: 10px; text-transform: uppercase; letter-spacing: 0.12em; color: var(--color-ds-text-muted);"}
             (str (count pts) " chunks")]]
           [:div {:class "flex gap-2 mb-6"}
            (for [t (or (:tags meta) [])]
              [:a {:href (str "/documents?tag=" t) :class "tag"} t])]
           [:div {:class "prose max-w-none"}
            (html/raw (marked (or content "")))]])))))

(defn page [config]
  (p/let [fragment (random-doc-fragment config)]
    (layout/page {:title "Random" :active-nav :random :config config}
                 [:div {:style "display: flex; align-items: baseline; justify-content: space-between; margin-bottom: 24px;"}
                  [:h2 {:style "font-family: var(--font-display); font-size: clamp(1.8rem, 3vw, 2.5rem);"} "Random Document"]
                  [:span {:style "font-family: var(--font-mono); font-size: 10px; text-transform: uppercase; letter-spacing: 0.15em; color: var(--color-ds-text-muted);"} "Refreshes every 10s"]]
                 [:div {:id "random-doc" :data-sse-url "/sse/random"}
                  (or fragment
                      [:div {:class "empty-state"}
                       [:h3 "No documents yet"]
                       [:p "Store documents via the MCP tools to get started."]])])))
