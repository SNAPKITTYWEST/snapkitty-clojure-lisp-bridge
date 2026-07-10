(ns doomsun.solarium.dashboard.views.overview
  (:require [doomsun.solarium.dashboard.views.layout :as layout]
            [doomsun.solarium.qdrant.collections :as collections]
            [doomsun.solarium.tools.documents :as docs]
            [doomsun.solarium.tools.tags :as tags]
            [promesa.core :as p]))

(defn- stat-card [label value]
  [:div {:class "stat-card"}
   [:div {:class "label"} label]
   [:div {:class "value"} (str value)]])

(defn- doc-row [{:keys [id title tags chunks created_at]}]
  [:tr
   [:td [:a {:href (str "/documents/" id)} title]]
   [:td (str chunks)]
   [:td (for [t tags] [:a {:href (str "/documents?tag=" t) :class "tag"} t])]
   [:td (when created_at (subs created_at 0 10))]])

(defn- tag-pill [{:keys [tag count]}]
  [:a {:href (str "/documents?tag=" tag) :class "tag"}
   tag [:span {:class "tag-count"} (str " " count)]])

(defn stats-fragment
  "Render just the stats grid (for SSE updates)."
  [config]
  (p/let [info     (collections/collection-info config)
          tag-data (tags/list-tags config)]
    (let [points-count (get info :points_count 0)
          vectors-count (get info :vectors_count 0)]
      [:div {:class "stats-grid"}
       (stat-card "Documents" (count (:tags tag-data)))
       (stat-card "Vectors" vectors-count)
       (stat-card "Points" points-count)
       (stat-card "Tags" (count (:tags tag-data)))])))

(defn page [config]
  (p/let [info     (collections/collection-info config)
          doc-list (docs/list-documents config {:limit 20})
          tag-data (tags/list-tags config)]
    (let [points-count (get info :points_count 0)
          vectors-count (get info :vectors_count 0)
          doc-count    (count (:documents doc-list))]
      (layout/page {:title "Overview" :active-nav :overview :config config}
                   [:div {:id "stats" :data-sse-url "/sse/overview"}
                    [:div {:class "stats-grid"}
                     (stat-card "Documents" doc-count)
                     (stat-card "Vectors" vectors-count)
                     (stat-card "Points" points-count)
                     (stat-card "Tags" (count (:tags tag-data)))]]
                   [:div {:class "two-column"}
                    [:section
                     [:h2 "Recent Documents"]
                     (if (seq (:documents doc-list))
                       [:table
                        [:thead
                         [:tr [:th "Title"] [:th "Chunks"] [:th "Tags"] [:th "Created"]]]
                        [:tbody
                         (for [doc (:documents doc-list)]
                           (doc-row doc))]]
                       [:div {:class "empty-state"}
                        [:h3 "No documents yet"]
                        [:p "Store documents via the MCP tools to get started."]])]
                    [:section
                     [:h2 "Tags"]
                     (if (seq (:tags tag-data))
                       [:div {:class "tag-cloud"}
                        (for [t (:tags tag-data)]
                          (tag-pill t))]
                       [:p {:class "empty-state"} "No tags yet."])]]))))
