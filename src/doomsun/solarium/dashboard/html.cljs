(ns doomsun.solarium.dashboard.html
  (:require [clojure.string :as str]))

(def ^:private void-elements
  #{"area" "base" "br" "col" "embed" "hr" "img" "input"
    "link" "meta" "param" "source" "track" "wbr"})

(defn escape-html [s]
  (-> (str s)
      (str/replace "&" "&amp;")
      (str/replace "<" "&lt;")
      (str/replace ">" "&gt;")
      (str/replace "\"" "&quot;")
      (str/replace "'" "&#39;")))

(defn- render-attr-value [k v]
  (cond
    (true? v)  (str " " (name k))
    (false? v) ""
    (nil? v)   ""
    :else      (str " " (name k) "=\"" (escape-html v) "\"")))

(defn- render-attrs [attrs]
  (apply str (map (fn [[k v]] (render-attr-value k v)) attrs)))

(defrecord RawHTML [html])

(declare render)

(defn- render-children [children]
  (apply str (map render children)))

(defn render [form]
  (cond
    (nil? form)    ""
    (instance? RawHTML form) (:html form)
    (string? form) (escape-html form)
    (number? form) (str form)
    (keyword? form) (str (name form))
    (seq? form)    (render-children form)
    (vector? form)
    (let [tag      (name (first form))
          has-attrs? (map? (second form))
          attrs    (if has-attrs? (second form) nil)
          children (if has-attrs? (drop 2 form) (rest form))]
      (if (contains? void-elements tag)
        (str "<" tag (render-attrs attrs) ">")
        (str "<" tag (render-attrs attrs) ">"
             (render-children children)
             "</" tag ">")))
    :else (escape-html (str form))))

(defn raw
  "Insert raw HTML without escaping."
  [html-str]
  (->RawHTML html-str))
