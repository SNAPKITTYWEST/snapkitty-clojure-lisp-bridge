(ns snapkitty.lisp.bridge.reader
  "Read LISP code and convert to Clojure forms"
  (:require [cljs.reader :as reader]))

(defn read-lisp [code]
  "Parse LISP code string into Clojure forms"
  (try
    (reader/read-string (str "(" code ")"))
    (catch :default e
      (throw (ex-info "LISP parse error" {:code code :error (str e)})))))

(defn read-lisp-file [path]
  "Read LISP file and parse"
  (let [fs (js/require "fs")]
    (read-lisp (.readFileSync fs path "utf-8"))))

(defn lisp->knowledge [lisp-form]
  "Convert LISP form to knowledge graph representation"
  {:form lisp-form
   :type (if (list? lisp-form) :list :atom)
   :value (if (list? lisp-form) (first lisp-form) lisp-form)})
