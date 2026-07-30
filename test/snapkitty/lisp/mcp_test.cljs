(ns snapkitty.lisp.mcp-test
  (:require [cljs.test :refer-macros [deftest is testing]]
            [snapkitty.lisp.mcp.util :as util]
            [snapkitty.lisp.mcp.config :as config]))

(deftest test-config-requires-qdrant-key
  (testing "Config should fail if QDRANT_API_KEY not set"
    (is (thrown? :default (config/load-config)))))

(deftest test-tool-result-format
  (testing "Tool results should have correct format"
    (let [result (util/tool-result "test output")]
      (is (= (:type result) "text"))
      (is (not (:is-error result))))))

(deftest test-tool-error-format
  (testing "Tool errors should be marked is-error true"
    (let [result (util/tool-error "something failed")]
      (is (= (:type result) "text"))
      (is (:is-error result)))))
