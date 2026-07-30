(ns snapkitty.lisp.emojiscript-adapter
  "Bridge between EmojiScript and Lisp Machine CLI
   Exposes Ahmad's EmojiScript bytecode dialect to the machine REPL"
  (:require [snapkitty.lisp.emojiscript :as emoji]))

;; ============================================================================
;; Command Handlers for Lisp Machine REPL
;; ============================================================================

(defn handle-emoji-command [repl-state input]
  "Parse and execute EmojiScript commands from the REPL

   Commands:
   - (emoji:compile \"🔢6 🔢7 ✖️ ↩️\") → bytecode
   - (emoji:exec \"🔢40 🔢2 ➕ ↩️\") → result
   - (emoji:disasm <bytecode>) → instruction list"
  (try
    (cond
      ; Compile command: (emoji:compile <source>)
      (string/starts-with? input "(emoji:compile")
      (let [source-match (re-find #"\(emoji:compile\s+\"([^\"]+)\"\)" input)
            source (second source-match)]
        (if source
          (let [result (emoji/compile-emojiscript source)]
            {:type :success
             :command :emoji:compile
             :source source
             :bytecode (:bytecode result)
             :hash (:hash result)
             :instructions (:instructions-count result)
             :output (str "✅ EmojiScript compiled\n"
                         "  Instructions: " (:instructions-count result) "\n"
                         "  Hash: " (subs (:hash result) 0 16) "...")})
          {:type :error :message "Invalid emoji:compile syntax"}))

      ; Execute command: (emoji:exec <source>)
      (string/starts-with? input "(emoji:exec")
      (let [source-match (re-find #"\(emoji:exec\s+\"([^\"]+)\"\)" input)
            source (second source-match)]
        (if source
          (try
            (let [compiled (emoji/compile-emojiscript source)
                  result (emoji/execute-emojiscript (:bytecode compiled))]
              {:type :success
               :command :emoji:exec
               :source source
               :result (:result result)
               :stack (:stack result)
               :steps (:steps result)
               :halted? (:halted? result)
               :output (str "✅ EmojiScript executed\n"
                           "  Result: " (:result result) "\n"
                           "  Stack: " (pr-str (:stack result)) "\n"
                           "  Steps: " (:steps result))})
            (catch js/Error e
              {:type :error :message (str "Execution failed: " (.-message e))}))
          {:type :error :message "Invalid emoji:exec syntax"}))

      ; Disassemble: (emoji:disasm <bytecode>)
      (string/starts-with? input "(emoji:disasm")
      (let [bytecode-str (re-find #"\(emoji:disasm\s+'([^)]+)\)" input)]
        (if bytecode-str
          {:type :success
           :command :emoji:disasm
           :output "🔍 EmojiScript disassembly not yet implemented"}
          {:type :error :message "Invalid emoji:disasm syntax"}))

      ; Info command: (emoji:info)
      (= input "(emoji:info)")
      {:type :success
       :command :emoji:info
       :output (str "╔══════════════════════════════════════════════════════════╗\n"
                    "║        Ahmad's EmojiScript — Bytecode Language         ║\n"
                    "╠══════════════════════════════════════════════════════════╣\n"
                    "║                                                          ║\n"
                    "║  15 Instructions:                                        ║\n"
                    "║    🔢<digits>      Push number literal                  ║\n"
                    "║    ➕ ➖ ✖️ ➗  Binary ops: add, sub, mul, div           ║\n"
                    "║    🤝 👐 🌀       Bitwise: and, or, xor                 ║\n"
                    "║    ➡️ ❓ ↩️        Control: jump, jumpif, return        ║\n"
                    "║    🔑 ⚡ 🏗️       Gate, call, alloc                     ║\n"
                    "║    📤 📦          Load, store                            ║\n"
                    "║                                                          ║\n"
                    "║  Usage:                                                  ║\n"
                    "║    (emoji:compile \"🔢6 🔢7 ✖️ ↩️\")  → bytecode        ║\n"
                    "║    (emoji:exec \"🔢40 🔢2 ➕ ↩️\")    → result           ║\n"
                    "║                                                          ║\n"
                    "║  Example: 🔢40 🔢2 ➕ ↩️ = 42                            ║\n"
                    "║                                                          ║\n"
                    "╚══════════════════════════════════════════════════════════╝")}

      :else
      {:type :unknown :input input})

    (catch js/Error e
      {:type :error :message (str "EmojiScript error: " (.-message e))})))

;; ============================================================================
;; Lisp Machine Integration
;; ============================================================================

(defn register-emoji-dialect! [repl-context]
  "Register EmojiScript dialect handlers with the Lisp Machine REPL

   Adds to the machine's command registry:
   - emoji:compile
   - emoji:exec
   - emoji:info"

  (update repl-context
          :command-handlers
          (fn [handlers]
            (merge handlers
                   {:emoji:compile (partial handle-emoji-command repl-context)
                    :emoji:exec (partial handle-emoji-command repl-context)
                    :emoji:disasm (partial handle-emoji-command repl-context)
                    :emoji:info (partial handle-emoji-command repl-context)}))))

;; ============================================================================
;; REPL Integration — Pretty Print Results
;; ============================================================================

(defn format-emoji-result [result]
  "Format EmojiScript results for REPL display"
  (case (:type result)
    :success
    (str "\n" (:output result) "\n\n")

    :error
    (str "❌ " (:message result) "\n\n")

    :unknown
    (str "❓ Unknown command: " (:input result) "\n\n")

    (str result)))

;; ============================================================================
;; Benchmarking & Profiling
;; ============================================================================

(defn bench-emoji-program [source iterations]
  "Benchmark EmojiScript execution

   Returns:
   {:source string
    :iterations number
    :total-time-ms number
    :avg-time-ms number
    :result any}"
  (let [compiled (emoji/compile-emojiscript source)
        bytecode (:bytecode compiled)
        start (js/Date.now)]
    (doseq [_ (range iterations)]
      (emoji/execute-emojiscript bytecode))
    (let [end (js/Date.now)
          total (- end start)]
      {:source source
       :iterations iterations
       :total-time-ms total
       :avg-time-ms (/ total iterations)
       :ops-per-sec (* 1000 (/ iterations total))})))

;; ============================================================================
;; Export API for Lisp Machine
;; ============================================================================

(def emojiscript-cli-api
  {:compile emoji/compile-emojiscript
   :execute emoji/execute-emojiscript
   :handle-command handle-emoji-command
   :register register-emoji-dialect!
   :format-result format-emoji-result
   :bench bench-emoji-program})
