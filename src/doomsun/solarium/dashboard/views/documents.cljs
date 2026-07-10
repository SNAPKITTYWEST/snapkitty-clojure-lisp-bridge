(ns doomsun.solarium.dashboard.views.documents
  (:require [clojure.string :as str]
            ["marked" :refer [marked]]
            [doomsun.solarium.dashboard.views.layout :as layout]
            [doomsun.solarium.dashboard.html :as html]
            [doomsun.solarium.qdrant.points :as points]
            [doomsun.solarium.tools.documents :as docs]
            [doomsun.solarium.tools.tags :as tags]
            [promesa.core :as p]))

(defn- find-overlap
  "Find the length of the longest suffix of `a` that is a prefix of `b`."
  [a b]
  (let [max-check (min (count a) (count b))]
    (loop [len max-check]
      (if (zero? len)
        0
        (if (= (subs a (- (count a) len)) (subs b 0 len))
          len
          (recur (dec len)))))))

(defn- deduplicate-chunks
  "Join chunks, removing overlapping content between adjacent chunks."
  [chunks]
  (if (<= (count chunks) 1)
    (first chunks)
    (loop [result (first chunks)
           remaining (rest chunks)]
      (if (empty? remaining)
        result
        (let [next-chunk (first remaining)
              overlap (find-overlap result next-chunk)]
          (recur (str result (subs next-chunk overlap))
                 (rest remaining)))))))

(defn- doc-row [{:keys [id title tags chunks created_at updated_at]}]
  [:tr
   [:td [:a {:href (str "/documents/" id)} (or title "Untitled")]]
   [:td (str chunks)]
   [:td (for [t tags] [:a {:href (str "/documents?tag=" t) :class "tag"} t])]
   [:td (when created_at (subs created_at 0 10))]])

(defn list-page [config {:keys [tag offset]}]
  (p/let [result (if tag
                   (p/let [r (points/scroll config
                                            :filter {:must [{:key "chunk_index" :match {:value 0}}
                                                            {:key "tags" :match {:value tag}}]}
                                            :limit 50
                                            :offset offset)]
                     {:documents (mapv (fn [pt]
                                         (let [p (:payload pt)]
                                           {:id (:doc_id p) :title (:title p)
                                            :tags (:tags p) :chunks (:chunk_count p)
                                            :created_at (:created_at p)
                                            :updated_at (:updated_at p)}))
                                       (:points r))
                      :next_offset (:next_page_offset r)})
                   (docs/list-documents config {:limit 50 :offset offset}))
          tag-data (tags/list-tags config)]
    (layout/page {:title "Documents" :active-nav :documents :config config}
                 [:h2 (if tag (str "Documents tagged: " tag) "All Documents")]
                 (when tag
                   [:p [:a {:href "/documents"} "Clear filter"]])
                 [:div {:style "margin-bottom: 20px;"}
                  (for [t (:tags tag-data)]
                    [:a {:href (str "/documents?tag=" (:tag t))
                         :class (str "tag" (when (= tag (:tag t)) " active"))}
                     (:tag t) [:span {:class "tag-count"} (str " " (:count t))]])]
                 (if (seq (:documents result))
                   [:div
                    [:table
                     [:thead
                      [:tr [:th "Title"] [:th "Chunks"] [:th "Tags"] [:th "Created"]]]
                     [:tbody
                      (for [doc (:documents result)]
                        (doc-row doc))]]
                    (when (:next_offset result)
                      [:div {:class "pagination"}
                       [:a {:href (str "/documents?"
                                       (when tag (str "tag=" tag "&"))
                                       "offset=" (:next_offset result))}
                        "Next page"]])]
                   [:div {:class "empty-state"}
                    [:h3 "No documents found"]
                    [:p (if tag
                          "No documents with this tag."
                          "Store documents via the MCP tools to get started.")]]))))

(defn detail-page [config doc-id]
  (p/let [result (points/scroll config
                                :filter {:must [{:key "doc_id" :match {:value doc-id}}]}
                                :limit 1000)
          pts    (sort-by #(get-in % [:payload :chunk_index]) (:points result))]
    (when (seq pts)
      (let [meta       (:payload (first pts))
            chunks     (mapv (fn [pt]
                               {:content     (get-in pt [:payload :content])
                                :chunk_index (get-in pt [:payload :chunk_index])})
                             pts)
            full-content (deduplicate-chunks (map :content chunks))]
        (layout/page {:title (:title meta) :active-nav :documents :config config}
                     [:div {:style "margin-bottom: 16px;"}
                      [:a {:href "/documents"} "Back to Documents"]]
                     [:h2 (or (:title meta) "Untitled")]
                     [:div {:class "meta"}
                      (when (:source_url meta)
                        [:a {:href (:source_url meta) :target "_blank"} "Source"])
                      [:span (str "Created: " (when (:created_at meta) (subs (:created_at meta) 0 10)))]
                      (when (and (:updated_at meta) (not= (:created_at meta) (:updated_at meta)))
                        [:span (str "Updated: " (subs (:updated_at meta) 0 10))])
                      [:span (str (count chunks) " chunks")]]
                     [:div {:id "doc-tags"}
                      [:strong "Tags: "]
                      (if (seq (:tags meta))
                        (for [t (:tags meta)]
                          [:span {:class "tag"}
                           t
                           " " [:a {:href (str "/documents/" doc-id "/untag?tag=" t)
                                    :class "remove-tag"} "x"]])
                        [:span {:style "color: var(--text-secondary)"} "none"])
                      [:form {:class "tag-form" :method "GET"
                              :action (str "/documents/" doc-id "/tag")}
                       [:input {:type "text" :name "tag" :placeholder "Add tag..."}]
                       [:button {:type "submit"} "Add"]]]
                     [:div {:class "prose max-w-none mt-8"}
                      (html/raw (marked full-content))])))))

(defn handle-tag-action [config doc-id action tag]
  (case action
    "tag"   (tags/tag-document! config {:id doc-id :tags [tag]})
    "untag" (tags/untag-document! config {:id doc-id :tags [tag]})))
