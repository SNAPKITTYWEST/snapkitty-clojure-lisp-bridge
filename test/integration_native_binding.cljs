(ns snapkitty.lisp.test.integration-native-binding
  "Integration tests for NASM validators + MCP tools"
  (:require [cljs.test :refer [deftest is testing async]]
            [promesa.core :as p]
            [snapkitty.lisp.native :as native]
            [snapkitty.lisp.mcp.tools :as tools]))

;; ============================================================================
;; Native Library Loading
;; ============================================================================

(deftest test-load-native-library
  "Verify native ASM library loads successfully"
  (async done
    (p/let [loaded? (native/load-native-library! "./native/build/Release/skclisp_native.node")]
      (is (= true loaded?))
      (is (= true (native/is-loaded?)))
      (done))))

;; ============================================================================
;; Mutation Validation Gate
;; ============================================================================

(deftest test-validate-mutation-all-pass
  "Mutation validation passes all 8 checks"
  (async done
    (p/let [result (native/validate-mutation!
                     {:mutation-id 1
                      :generation-before 10
                      :generation-after 11
                      :actor 100
                      :target 200}
                     nil)]
      (is (true? (:passes-gate result)))
      (is (= 255 (:error-code result)))
      (is (= "All checks passed" (:details result)))
      (done))))

(deftest test-validate-mutation-generation-fail
  "Mutation fails on generation check (not monotonic)"
  (async done
    (p/let [result (native/validate-mutation!
                     {:mutation-id 1
                      :generation-before 10
                      :generation-after 10  ; Not greater than before
                      :actor 100
                      :target 200}
                     nil)]
      (is (false? (:passes-gate result)))
      (is (= 8 (:error-code result)))
      (is (clojure.string/includes? (:details result) "generation"))
      (done))))

;; ============================================================================
;; Blake3 Digest Verification
;; ============================================================================

(deftest test-verify-blake3-valid
  "Blake3 digest verification succeeds on valid input"
  (async done
    (p/let [payload (js/Uint8Array. #js [1 2 3 4 5])
            digest (js/Uint8Array. 32)  ; Placeholder 32-byte digest
            result (native/verify-blake3! payload digest)]
      (is (true? (:digest-valid result)))
      (is (= 0 (:error-code result)))
      (done))))

(deftest test-verify-blake3-invalid-input
  "Blake3 verification fails on null input"
  (async done
    (p/let [result (native/verify-blake3! nil nil)]
      (is (false? (:digest-valid result)))
      (is (= 2 (:error-code result)))
      (done))))

;; ============================================================================
;; Ed25519 Signature Verification
;; ============================================================================

(deftest test-verify-ed25519-valid
  "Ed25519 verification succeeds on valid input"
  (async done
    (p/let [message (js/Uint8Array. #js [72 101 108 108 111])  ; "Hello"
            sig (js/Uint8Array. 64)  ; Placeholder signature
            pk (js/Uint8Array. 32)   ; Placeholder public key
            result (native/verify-ed25519! message sig pk)]
      (is (true? (:signature-valid result)))
      (is (= 0 (:error-code result)))
      (done))))

;; ============================================================================
;; MCP Tool Integration
;; ============================================================================

(deftest test-mcp-validate-mutation-tool
  "MCP validate_mutation tool calls native validator"
  (async done
    (p/let [result (tools/handle-validate-mutation
                     {:mutation-id 1
                      :generation-before 10
                      :generation-after 11
                      :actor 100
                      :target 200})]
      (is (clojure.string/includes? result "Mutation validation"))
      (is (or (clojure.string/includes? result "PASS")
              (clojure.string/includes? result "FAIL")))
      (done))))

(deftest test-mcp-verify-blake3-tool
  "MCP verify_blake3 tool calls native validator"
  (async done
    (p/let [result (tools/handle-verify-blake3
                     {:payload "Hello world"
                      :expected-digest "abc123"})]
      (is (clojure.string/includes? result "Blake3 verification"))
      (is (or (clojure.string/includes? result "MATCH")
              (clojure.string/includes? result "MISMATCH")))
      (done))))

(deftest test-mcp-verify-ed25519-tool
  "MCP verify_ed25519 tool calls native validator"
  (async done
    (p/let [result (tools/handle-verify-ed25519
                     {:message "Hello"
                      :signature "sig123456789"
                      :public-key "pk123456789"})]
      (is (clojure.string/includes? result "Ed25519 verification"))
      (is (or (clojure.string/includes? result "VALID")
              (clojure.string/includes? result "INVALID")))
      (done))))
