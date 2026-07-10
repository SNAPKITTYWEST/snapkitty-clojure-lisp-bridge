(ns doomsun.solarium.dashboard.views.analytics
  (:require [doomsun.solarium.dashboard.views.layout :as layout]
            [doomsun.solarium.dashboard.html :as html]
            [doomsun.solarium.qdrant.collections :as collections]
            [doomsun.solarium.qdrant.points :as points]
            [doomsun.solarium.tools.tags :as tags]
            [promesa.core :as p]))

(defn- vega-lite-embed [element-id spec]
  (html/raw
   (str "vegaEmbed('#" element-id "', "
        (js/JSON.stringify (clj->js spec))
        ", {actions: false, theme: window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : undefined})"
        ".catch(function(e) { console.warn(e); });")))

(defn page [config]
  (p/let [coll-info  (collections/collection-info config)
          doc-result (points/scroll config
                                    :filter {:must [{:key "chunk_index" :match {:value 0}}]}
                                    :limit 1000)
          all-result (points/scroll config :limit 1000)
          tag-data   (tags/list-tags config)]
    (let [docs       (:points doc-result)
          all-points (:points all-result)
          docs-by-date (->> docs
                            (map #(when-let [d (get-in % [:payload :created_at])]
                                    (subs d 0 10)))
                            (remove nil?)
                            frequencies
                            (sort)
                            (mapv (fn [[date cnt]] {:date date :count cnt})))
          chunk-sizes (->> all-points
                           (map #(count (get-in % [:payload :content] "")))
                           (mapv (fn [s] {:size s})))]
      (layout/page {:title "Analytics" :active-nav :analytics :config config}
                   [:script {:src "https://cdn.jsdelivr.net/npm/vega@5"}]
                   [:script {:src "https://cdn.jsdelivr.net/npm/vega-lite@5"}]
                   [:script {:src "https://cdn.jsdelivr.net/npm/vega-embed@6"}]

                   [:h2 "Analytics"]

                   [:div {:class "chart-section"}
                    [:h3 "Documents Over Time"]
                    (if (seq docs-by-date)
                      [:div
                       [:div {:id "chart-docs-time"}]
                       [:script
                        (vega-lite-embed "chart-docs-time"
                                         {:$schema "https://vega.github.io/schema/vega-lite/v5.json"
                                          :width "container"
                                          :height 250
                                          :data {:values docs-by-date}
                                          :mark {:type "bar" :color "#e67e22"}
                                          :encoding {:x {:field "date" :type "temporal" :title "Date"}
                                                     :y {:field "count" :type "quantitative" :title "Documents"}}})]]
                      [:p {:class "empty-state"} "No data yet."])]

                   [:div {:class "chart-section"}
                    [:h3 "Tag Distribution"]
                    (if (seq (:tags tag-data))
                      [:div
                       [:div {:id "chart-tags"}]
                       [:script
                        (vega-lite-embed "chart-tags"
                                         {:$schema "https://vega.github.io/schema/vega-lite/v5.json"
                                          :width "container"
                                          :height 250
                                          :data {:values (:tags tag-data)}
                                          :mark {:type "bar" :color "#3498db"}
                                          :encoding {:x {:field "tag" :type "nominal" :sort "-y" :title "Tag"}
                                                     :y {:field "count" :type "quantitative" :title "Count"}}})]]
                      [:p {:class "empty-state"} "No tags yet."])]

                   [:div {:class "chart-section"}
                    [:h3 "Chunk Size Distribution"]
                    (if (seq chunk-sizes)
                      [:div
                       [:div {:id "chart-chunks"}]
                       [:script
                        (vega-lite-embed "chart-chunks"
                                         {:$schema "https://vega.github.io/schema/vega-lite/v5.json"
                                          :width "container"
                                          :height 250
                                          :data {:values chunk-sizes}
                                          :mark {:type "bar" :color "#27ae60"}
                                          :encoding {:x {:field "size" :type "quantitative" :bin true :title "Characters"}
                                                     :y {:aggregate "count" :type "quantitative" :title "Chunks"}}})]]
                      [:p {:class "empty-state"} "No chunks yet."])]

                   [:div {:class "card"}
                    [:h3 {:style "margin-bottom: 12px;"} "Collection Stats"]
                    [:table
                     [:tbody
                      [:tr [:td "Points"] [:td (str (get coll-info :points_count 0))]]
                      [:tr [:td "Vectors"] [:td (str (get coll-info :vectors_count 0))]]
                      [:tr [:td "Indexed Vectors"] [:td (str (get coll-info :indexed_vectors_count 0))]]
                      [:tr [:td "Segments"] [:td (str (get coll-info :segments_count 0))]]]]]))))
