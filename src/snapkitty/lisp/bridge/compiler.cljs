(ns snapkitty.lisp.bridge.compiler
  "Compile LISP → knowledge graph"
  (:require [snapkitty.lisp.bridge.reader :as reader]))

(defn compile-form [form]
  "Compile a LISP form into structured knowledge"
  (cond
    (symbol? form) {:type :symbol :name (str form)}
    (number? form) {:type :number :value form}
    (string? form) {:type :string :value form}
    (list? form) {:type :sexpr
                  :head (first form)
                  :args (mapv compile-form (rest form))}
    :else {:type :unknown :value form}))

(defn compile-lisp [code]
  "Compile LISP code to knowledge structure"
  (let [forms (reader/read-lisp code)]
    (mapv compile-form forms)))
