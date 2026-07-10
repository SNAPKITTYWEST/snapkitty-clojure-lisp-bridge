(ns doomsun.solarium.dashboard.views.search
  (:require [doomsun.solarium.dashboard.views.layout :as layout]
            [doomsun.solarium.tools.search :as search-tools]
            [doomsun.solarium.tools.tags :as tags]
            [promesa.core :as p]))

(defn- result-card [{:keys [doc_id title tags score snippet]}]
  [:div {:class "result-card"}
   [:div {:class "result-header"}
    [:a {:href (str "/documents/" doc_id)} [:h3 (or title "Untitled")]]
    [:span {:class "score"} (.toFixed score 3)]]
   [:div {:class "score-bar"}
    [:div {:class "score-fill" :style (str "width:" (* score 100) "%")}]]
   [:p {:class "snippet"} snippet]
   [:div
    (for [t tags]
      [:a {:href (str "/documents?tag=" t) :class "tag"} t])]])

(defn page [config {:keys [query tag]}]
  (p/let [results  (when (and query (seq query))
                     (search-tools/search-documents config {:query query :limit 20}))
          tag-data (tags/list-tags config)]
    (layout/page {:title "Search" :active-nav :search :config config}
                 [:h2 "Search"]
                 [:form {:class "search-box" :method "GET" :action "/search"}
                  [:input {:type "text" :name "query" :value (or query "")
                           :placeholder "Semantic search..." :autofocus true}]
                  [:select {:name "tag"}
                   [:option {:value ""} "All tags"]
                   (for [{:keys [tag count]} (:tags tag-data)]
                     [:option {:value tag :selected (when (= tag query) true)}
                      (str tag " (" count ")")])]
                  [:button {:type "submit"} "Search"]]
                 (cond
                   (and query results (seq (:results results)))
                   [:div
                    [:p {:style "color: var(--text-secondary); margin-bottom: 16px;"}
                     (str (count (:results results)) " results for \"" query "\"")]
                    (for [r (:results results)]
                      (result-card r))]

                   (and query results)
                   [:div {:class "empty-state"}
                    [:h3 "No results"]
                    [:p (str "No documents matched \"" query "\"")]]

                   :else nil))))
