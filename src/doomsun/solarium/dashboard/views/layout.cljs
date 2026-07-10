(ns doomsun.solarium.dashboard.views.layout
  (:require [clojure.string :as str]))

(def ^:private nav-items
  [{:path "/" :label "Overview" :key :overview}
   {:path "/documents" :label "Documents" :key :documents}
   {:path "/search" :label "Search" :key :search}
   {:path "/analytics" :label "Analytics" :key :analytics}
   {:path "/random" :label "Random" :key :random}])

(defn- qdrant-label
  "Extract a short display label from the Qdrant URL."
  [url]
  (cond
    (str/includes? url "cloud.qdrant.io")
    (let [host (second (re-find #"//([^:]+)" url))
          cluster-id (first (str/split (or host "") #"\."))]
      (str (subs cluster-id 0 (min 8 (count cluster-id))) "… (cloud)"))

    (str/includes? url "localhost")
    "localhost"

    :else
    (second (re-find #"//([^:/]+)" url))))

(defn page [{:keys [title active-nav config]} & body]
  [:html {:lang "en"}
   [:head
    [:meta {:charset "utf-8"}]
    [:meta {:name "viewport" :content "width=device-width, initial-scale=1"}]
    [:title (str title " \u2014 Solarium")]
    [:link {:rel "preconnect" :href "https://fonts.googleapis.com"}]
    [:link {:rel "preconnect" :href "https://fonts.gstatic.com" :crossorigin true}]
    [:link {:rel "stylesheet"
            :href "https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&family=JetBrains+Mono:wght@400;500;600&family=Playfair+Display:wght@600;700&display=swap"}]
    [:link {:rel "stylesheet" :href "/static/app.css"}]
    [:script {:src "https://cdn.jsdelivr.net/npm/idiomorph@0.3.0/dist/idiomorph.min.js"
              :defer true}]]
   [:body
    [:nav {:class "sidebar"}
     [:div {:class "sidebar-header"}
      [:a {:href "/"}
       [:img {:src "/static/logo.svg" :alt "Solarium" :class "sidebar-logo"}]]
      [:h1 "SOLARIUM"]
      [:div {:class "subtitle"} "Knowledge Base"]
      (when config
        [:div {:class "sidebar-meta"}
         [:div {:class "sidebar-meta-item"}
          [:span {:class "sidebar-meta-label"} "collection"]
          [:span {:class "sidebar-meta-value"} (:collection-name config)]]
         [:div {:class "sidebar-meta-item"}
          [:span {:class "sidebar-meta-label"} "qdrant"]
          [:span {:class "sidebar-meta-value"} (qdrant-label (:qdrant-url config))]]])]
     (for [{:keys [path label key]} nav-items]
       [:a {:href path :class (when (= key active-nav) "active")}
        [:span label]])]
    [:main {:id "content"}
     body]
    [:script {:src "/static/app.js"}]]])
