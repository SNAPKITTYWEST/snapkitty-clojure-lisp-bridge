(ns snapkitty.lisp.bridge-test
  (:require [cljs.test :refer-macros [deftest is testing]]
            [snapkitty.lisp.bridge.reader :as reader]
            [snapkitty.lisp.bridge.compiler :as compiler]))

(deftest test-lisp-reader-atom
  (testing "Reader should parse simple atoms"
    (let [result (reader/read-lisp "42")]
      (is (number? (first result))))))

(deftest test-lisp-reader-list
  (testing "Reader should parse S-expressions"
    (let [result (reader/read-lisp "(+ 1 2)")]
      (is (list? (first result))))))

(deftest test-lisp-compiler-symbol
  (testing "Compiler should convert symbols"
    (let [compiled (compiler/compile-form 'x)]
      (is (= (:type compiled) :symbol)))))

(deftest test-lisp-compiler-number
  (testing "Compiler should convert numbers"
    (let [compiled (compiler/compile-form 42)]
      (is (= (:type compiled) :number)))))

(deftest test-lisp-compiler-sexpr
  (testing "Compiler should convert S-expressions"
    (let [compiled (compiler/compile-form '(+ 1 2))]
      (is (= (:type compiled) :sexpr)))))
