(ns snapkitty.lisp.test.emojiscript-tests
  "EmojiScript compiler + executor tests"
  (:require [cljs.test :refer [deftest is testing]]
            [snapkitty.lisp.emojiscript :as emoji]))

;; ============================================================================
;; Compilation Tests
;; ============================================================================

(deftest test-compile-simple-arithmetic
  "Compile: 🔢40 🔢2 ➕ ↩️"
  (let [source "🔢40 🔢2 ➕ ↩️"
        result (emoji/compile-emojiscript source)]
    (is (= true (:valid? result)))
    (is (= 4 (:instructions-count result)))
    (is (contains? #{:Push :Add :Ret}
                   (map :op (:bytecode result))))))

(deftest test-compile-cap-gate
  "Compile: 🔢3 🔢5 🔑 ↩️ (capability check)"
  (let [source "🔢3 🔢5 🔑 ↩️"
        result (emoji/compile-emojiscript source)]
    (is (= true (:valid? result)))
    (is (some #(= :CapGate (:op %)) (:bytecode result)))))

(deftest test-compile-memory-ops
  "Compile: 🔢64 🔢9 🏗️ 🔢0 📤 ↩️ (alloc + load)"
  (let [source "🔢64 🔢9 🏗️ 🔢0 📤 ↩️"
        result (emoji/compile-emojiscript source)]
    (is (= true (:valid? result)))
    (is (some #(= :Alloc (:op %)) (:bytecode result)))
    (is (some #(= :Load (:op %)) (:bytecode result)))))

(deftest test-compile-jump
  "Compile: 🔢10 ➡️ (unconditional jump)"
  (let [source "🔢10 ➡️ ↩️"
        result (emoji/compile-emojiscript source)]
    (is (= true (:valid? result)))
    (is (some #(= :Jump (:op %)) (:bytecode result)))))

(deftest test-compile-conditional-jump
  "Compile: 🔢1 ❓ (conditional jump)"
  (let [source "🔢1 ❓ ↩️"
        result (emoji/compile-emojiscript source)]
    (is (= true (:valid? result)))
    (is (some #(= :JumpIf (:op %)) (:bytecode result)))))

(deftest test-compile-bitwise-ops
  "Compile: 🔢7 🔢3 🤝 ↩️ (bitwise AND)"
  (let [source "🔢7 🔢3 🤝 ↩️"
        result (emoji/compile-emojiscript source)]
    (is (= true (:valid? result)))
    (is (some #(= :And (:op %)) (:bytecode result)))))

(deftest test-compile-rejects-empty
  "Reject empty program"
  (is (thrown-with-msg?
        js/Error
        #"Empty program"
        (emoji/compile-emojiscript ""))))

(deftest test-compile-rejects-whitespace-only
  "Reject whitespace-only program"
  (is (thrown-with-msg?
        js/Error
        #"Empty program"
        (emoji/compile-emojiscript "   \n\t  "))))

(deftest test-compile-rejects-unknown-emoji
  "Reject unknown emoji"
  (is (thrown-with-msg?
        js/Error
        #"Unknown emoji"
        (emoji/compile-emojiscript "🥷"))))

;; ============================================================================
;; Execution Tests
;; ============================================================================

(deftest test-execute-simple-addition
  "Execute: 🔢40 🔢2 ➕ ↩️ = 42"
  (let [source "🔢40 🔢2 ➕ ↩️"
        compiled (emoji/compile-emojiscript source)
        result (emoji/execute-emojiscript (:bytecode compiled))]
    (is (= true (:halted? result)))
    (is (= 42 (:result result)))
    (is (= [42] (:stack result)))))

(deftest test-execute-subtraction
  "Execute: 🔢100 🔢40 ➖ ↩️ = 60"
  (let [source "🔢100 🔢40 ➖ ↩️"
        compiled (emoji/compile-emojiscript source)
        result (emoji/execute-emojiscript (:bytecode compiled))]
    (is (= 60 (:result result)))))

(deftest test-execute-multiplication
  "Execute: 🔢6 🔢7 ✖️ ↩️ = 42"
  (let [source "🔢6 🔢7 ✖️ ↩️"
        compiled (emoji/compile-emojiscript source)
        result (emoji/execute-emojiscript (:bytecode compiled))]
    (is (= 42 (:result result)))))

(deftest test-execute-division
  "Execute: 🔢84 🔢2 ➗ ↩️ = 42"
  (let [source "🔢84 🔢2 ➗ ↩️"
        compiled (emoji/compile-emojiscript source)
        result (emoji/execute-emojiscript (:bytecode compiled))]
    (is (= 42 (:result result)))))

(deftest test-execute-bitwise-and
  "Execute: 🔢15 🔢7 🤝 ↩️ = 7 (bitwise AND)"
  (let [source "🔢15 🔢7 🤝 ↩️"
        compiled (emoji/compile-emojiscript source)
        result (emoji/execute-emojiscript (:bytecode compiled))]
    (is (= 7 (:result result)))))

(deftest test-execute-bitwise-or
  "Execute: 🔢12 🔢5 👐 ↩️ = 13 (bitwise OR)"
  (let [source "🔢12 🔢5 👐 ↩️"
        compiled (emoji/compile-emojiscript source)
        result (emoji/execute-emojiscript (:bytecode compiled))]
    (is (= 13 (:result result)))))

(deftest test-execute-bitwise-xor
  "Execute: 🔢12 🔢5 🌀 ↩️ = 9 (bitwise XOR)"
  (let [source "🔢12 🔢5 🌀 ↩️"
        compiled (emoji/compile-emojiscript source)
        result (emoji/execute-emojiscript (:bytecode compiled))]
    (is (= 9 (:result result)))))

(deftest test-execute-division-by-zero
  "Reject division by zero"
  (let [source "🔢10 🔢0 ➗ ↩️"
        compiled (emoji/compile-emojiscript source)
        result (emoji/execute-emojiscript (:bytecode compiled))]
    (is (contains? result :error))
    (is (clojure.string/includes? (:error result) "Division by zero"))))

(deftest test-execute-max-steps
  "Enforce step limit (prevent infinite loops)"
  (let [source "🔢0 ➡️"  ; unconditional jump to 0 = infinite loop
        compiled (emoji/compile-emojiscript source)
        result (emoji/execute-emojiscript (:bytecode compiled) :max-steps 100)]
    (is (= 100 (:steps result)))
    (is (= false (:halted? result)))))

;; ============================================================================
;; Integration Tests (MCP tools)
;; ============================================================================

(deftest test-mcp-compile-tool
  "MCP compile_emojiscript tool"
  (let [source "🔢40 🔢2 ➕ ↩️"]
    ; Would call: (tools/handle-compile-emojiscript {:source source})
    ; For now, just verify compile works
    (is (truthy? (emoji/compile-emojiscript source)))))

(deftest test-mcp-execute-tool
  "MCP execute_emojiscript tool"
  (let [source "🔢40 🔢2 ➕ ↩️"
        compiled (emoji/compile-emojiscript source)]
    ; Would call: (tools/handle-execute-emojiscript {:source source :max-steps 10000})
    (is (= 42 (:result (emoji/execute-emojiscript (:bytecode compiled)))))))
