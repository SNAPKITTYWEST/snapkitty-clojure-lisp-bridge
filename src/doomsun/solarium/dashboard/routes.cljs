(ns doomsun.solarium.dashboard.routes
  (:require [clojure.string :as str]
            [doomsun.solarium.dashboard.http :as http]
            [doomsun.solarium.dashboard.html :as html]
            [doomsun.solarium.dashboard.static :as static]
            [doomsun.solarium.dashboard.views.overview :as overview]
            [doomsun.solarium.dashboard.views.documents :as documents]
            [doomsun.solarium.dashboard.views.search :as search]
            [doomsun.solarium.dashboard.views.analytics :as analytics]
            [doomsun.solarium.dashboard.views.random :as random]
            [doomsun.solarium.dashboard.sse :as sse]
            [promesa.core :as p]))

(defn- get-param [search-params k]
  (.get search-params k))

(defn- parse-path-segments [pathname]
  (filterv (complement str/blank?) (str/split pathname #"/")))

(defn handle [config {:keys [method pathname search-params]} res]
  (let [segments (parse-path-segments pathname)]
    (cond
      ;; Static assets
      (= pathname "/static/app.js")
      (http/respond-static res "application/javascript" static/app-js)

      (= pathname "/static/app.css")
      (http/respond-static res "text/css" static/app-css)

      (= pathname "/static/logo.svg")
      (http/respond-static res "image/svg+xml" static/logo-svg)

      ;; SSE endpoints
      (= pathname "/sse/overview")
      (sse/handle-overview config res)

      (= pathname "/sse/documents")
      (sse/handle-documents config res)

      (= pathname "/sse/random")
      (sse/handle-random config res)

      ;; Overview
      (or (= pathname "/") (= pathname ""))
      (p/let [page-html (overview/page config)]
        (http/respond-html res 200 (html/render page-html)))

      ;; Documents list
      (= pathname "/documents")
      (let [tag    (get-param search-params "tag")
            offset (get-param search-params "offset")]
        (p/let [page-html (documents/list-page config
                                               {:tag    tag
                                                :offset offset})]
          (http/respond-html res 200 (html/render page-html))))

      ;; Document detail
      (and (= (first segments) "documents")
           (second segments)
           (nil? (get segments 2)))
      (let [doc-id (second segments)]
        (p/let [page-html (documents/detail-page config doc-id)]
          (if page-html
            (http/respond-html res 200 (html/render page-html))
            (http/respond-html res 404 "<h1>Document not found</h1>"))))

      ;; Tag action
      (and (= (first segments) "documents")
           (second segments)
           (#{"tag" "untag"} (get segments 2)))
      (let [doc-id (second segments)
            action (get segments 2)
            tag    (get-param search-params "tag")]
        (p/let [_ (documents/handle-tag-action config doc-id action tag)]
          (http/respond-redirect res (str "/documents/" doc-id))))

      ;; Search
      (= pathname "/search")
      (let [query (get-param search-params "query")
            tag   (get-param search-params "tag")]
        (p/let [page-html (search/page config {:query query :tag tag})]
          (http/respond-html res 200 (html/render page-html))))

      ;; Analytics
      (= pathname "/analytics")
      (p/let [page-html (analytics/page config)]
        (http/respond-html res 200 (html/render page-html)))

      ;; Random
      (= pathname "/random")
      (p/let [page-html (random/page config)]
        (http/respond-html res 200 (html/render page-html)))

      ;; 404
      :else
      (http/respond-html res 404 "<h1>Not Found</h1>"))))
