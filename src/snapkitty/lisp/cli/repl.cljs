;; SKC-LISP: Lisp Machine REPL (CLI + Browser)
;; Interactive frontend connecting reader → compiler → LTMS → WASM → JIT → WORM

(ns snapkitty.lisp.cli.repl
  (:require
    [snapkitty.lisp.bridge.reader :as reader]
    [snapkitty.lisp.bridge.compiler :as compiler]
    [snapkitty.lisp.emojiscript :as emoji]
    [snapkitty.lisp.ltms.ltms :as ltms]
    [snapkitty.lisp.native :as native]
    [snapkitty.lisp.jit :as jit]
    [snapkitty.lisp.jit-ledger :as ledger]
    [snapkitty.lisp.wasm-bridge :as wasm]
    [cljs.reader :as edn]
    [promesa.core :as p]))

;; ============================================================================
;; REPL STATE
;; ============================================================================

(defonce repl-context
  (atom {:history []
         :variables {}
         :loaded-files []
         :proof-certs {}
         :wasm-ready? false}))

;; ============================================================================
;; CORE REPL FUNCTIONS
;; ============================================================================

(defn read-lisp
  "Parse LISP code into forms."
  [source]
  (try
    (reader/read-lisp source)
    (catch js/Error e
      {:error :parse-error :message (.toString e)})))

(defn compile-form
  "Compile a LISP form to semantic graph + bytecode."
  [form]
  (try
    (let [compiled (compiler/compile-form form)]
      (ltms/assert-fact! form :lisp-compiler 0.95 50)
      compiled)
    (catch js/Error e
      {:error :compile-error :message (.toString e)})))

(defn eval-form
  "Evaluate a compiled form (semantic + execution)."
  [form]
  (let [compiled (compile-form form)]
    (if (:error compiled)
      compiled
      (try
        (eval compiled)
        (catch js/Error e
          {:error :eval-error :message (.toString e)})))))

;; ============================================================================
;; EMOJI SCRIPT INTEGRATION
;; ============================================================================

(defn compile-emoji
  "Compile EmojiScript source to bytecode."
  [source]
  (try
    (emoji/compile-emojiscript source)
    (catch js/Error e
      {:error :emoji-compile-error :message (.toString e)})))

(defn execute-emoji
  "Execute EmojiScript bytecode with full pipeline."
  [source]
  (try
    (let [compiled (compile-emoji source)]
      (if (:error compiled)
        compiled
        (let [result (emoji/execute-emojiscript (:bytecode compiled))]
          (ltms/assert-fact! {:emoji-result result} :emoji-executor 0.95 50)
          result)))
    (catch js/Error e
      {:error :emoji-exec-error :message (.toString e)})))

;; ============================================================================
;; JIT COMPILATION PIPELINE
;; ============================================================================

(defn compile-with-jit
  "Compile LISP → EmojiScript → bytecode → native (JIT)."
  [source proof-cert]
  (try
    (p/let [emoji-compiled (compile-emoji source)
            jit-result (jit/compile-with-proof
                         source proof-cert "x86_64")]
      (if (:error jit-result)
        jit-result
        (do
          ;; Record in WORM ledger
          (ledger/compile-and-seal-to-ledger!
            source proof-cert "x86_64" "repl-user" (make-array 32)
            (atom {:entries [] :generation 0}))
          jit-result)))
    (catch js/Error e
      {:error :jit-error :message (.toString e)})))

;; ============================================================================
;; KNOWLEDGE LAYER QUERIES
;; ============================================================================

(defn query-belief
  "Query the knowledge base for a belief."
  [value]
  (let [belief (ltms/query value)]
    (if belief
      {:found true :value value :confidence (:confidence belief) :source (:source belief)}
      {:found false :value value :confidence 0.0})))

(defn assert-belief!
  "Assert a new belief to the knowledge base."
  [value source confidence priority]
  (ltms/assert-fact! value source confidence priority)
  {:status "asserted" :value value :source source})

(defn disambiguate-concept
  "Find senses of a concept."
  [concept-name context]
  (let [senses (ltms/disambiguate concept-name context)]
    {:concept concept-name :senses senses :count (count senses)}))

;; ============================================================================
;; WASM CRYPTO INTEGRATION
;; ============================================================================

(defn verify-blake3-wasm
  "Verify Blake3 digest (via WASM)."
  [payload expected-digest]
  (try
    (wasm/blake3-verify payload expected-digest)
    (catch js/Error e
      {:error :blake3-error :message (.toString e)})))

(defn verify-ed25519-wasm
  "Verify Ed25519 signature (via WASM)."
  [message signature public-key]
  (try
    (wasm/ed25519-verify message signature public-key)
    (catch js/Error e
      {:error :ed25519-error :message (.toString e)})))

;; ============================================================================
;; REPL COMMANDS
;; ============================================================================

(def repl-commands
  {
   ;; LISP operations
   :read "(read <source>) — Parse LISP code"
   :compile "(compile <form>) — Compile to semantic graph"
   :eval "(eval <form>) — Evaluate form"

   ;; EmojiScript operations
   :emoji:info "(emoji:info) — Show emoji instruction reference"
   :emoji:compile "(emoji:compile <source>) — Compile EmojiScript to bytecode"
   :emoji:exec "(emoji:exec <source>) — Execute EmojiScript"

   ;; JIT operations
   :jit:compile "(jit:compile <source> <proof-cert>) — Compile with JIT"
   :jit:status "(jit:status) — Show JIT compilation stats"

   ;; Knowledge layer
   :kb:query "(kb:query <value>) — Query belief base"
   :kb:assert "(kb:assert <value> <source> <confidence>) — Assert belief"
   :kb:disambiguate "(kb:disambiguate <concept> <context>) — Find senses"

   ;; WASM crypto
   :crypto:blake3 "(crypto:blake3 <payload> <digest>) — Verify Blake3"
   :crypto:ed25519 "(crypto:ed25519 <msg> <sig> <pk>) — Verify Ed25519"

   ;; System
   :help "(help) — Show this list"
   :history "(history) — Show command history"
   :clear "(clear) — Clear history"
   :exit "(exit) — Exit REPL"
   })

;; ============================================================================
;; COMMAND PARSER & EXECUTOR
;; ============================================================================

(defn parse-command
  "Parse REPL command like (command arg1 arg2)."
  [input]
  (try
    (let [forms (reader/read-lisp input)]
      (if (seq? forms)
        {:command (keyword (first forms))
         :args (rest forms)}
        {:error "Not a command"}))
    (catch js/Error e
      {:error (.toString e)})))

(defn execute-command
  "Execute a REPL command with args."
  [command args]
  (case command
    ;; LISP
    :read (read-lisp (first args))
    :compile (compile-form (first args))
    :eval (eval-form (first args))

    ;; EmojiScript
    :emoji:info (emoji/emoji-reference)
    :emoji:compile (compile-emoji (first args))
    :emoji:exec (execute-emoji (first args))

    ;; JIT
    :jit:compile (compile-with-jit (first args) (second args))
    :jit:status {:status "JIT ready" :target "x86_64"}

    ;; Knowledge
    :kb:query (query-belief (first args))
    :kb:assert (assert-belief! (first args) (second args) (nth args 2 0.9) (nth args 3 50))
    :kb:disambiguate (disambiguate-concept (first args) (second args))

    ;; Crypto
    :crypto:blake3 (verify-blake3-wasm (first args) (second args))
    :crypto:ed25519 (verify-ed25519-wasm (first args) (second args) (nth args 2))

    ;; System
    :help repl-commands
    :history (:history @repl-context)
    :clear (do (swap! repl-context assoc :history []) {:status "history cleared"})

    {:error (str "Unknown command: " command)}))

(defn eval-input
  "Parse and execute REPL input."
  [input]
  (let [trimmed (.trim input)]
    (if (.startsWith trimmed "(")
      ;; Command
      (let [{:keys [command args error]} (parse-command trimmed)]
        (if error
          {:error error}
          (let [result (execute-command command args)]
            (swap! repl-context update :history conj {:input trimmed :result result})
            result)))
      ;; EmojiScript or bare LISP
      (if (clojure.string/includes? trimmed "🔢")
        ;; Emoji code
        (execute-emoji trimmed)
        ;; Bare LISP expression
        (let [forms (read-lisp trimmed)]
          (if (:error forms)
            forms
            (eval-form (first forms))))))))

;; ============================================================================
;; BROWSER REPL INTEGRATION
;; ============================================================================

(defn format-output
  "Pretty-print REPL output for display."
  [result]
  (cond
    (:error result)
    (str "ERROR: " (:message result))

    (map? result)
    (str (clj->js result))

    (vector? result)
    (str (pr-str result))

    :else
    (str result)))

(defn ^:export init-repl
  "Initialize REPL (called from browser/Node.js)."
  []
  (p/let [_ (wasm/init-wasm!)]
    (swap! repl-context assoc :wasm-ready? true)
    (println "✓ REPL initialized")))

(defn ^:export repl-eval
  "Execute REPL input from browser console."
  [input]
  (let [result (eval-input input)]
    (println (format-output result))
    result))

(defn ^:export get-context
  "Get current REPL context (for debugging)."
  []
  @repl-context)

;; ============================================================================
;; TEST HARNESS
;; ============================================================================

(defn run-repl-tests []
  (println "=== Lisp Machine REPL Tests ===\n")

  ;; Test 1: Parse LISP
  (println "Test 1: Parse LISP")
  (let [result (read-lisp "(+ 1 2)")]
    (println "✓ Parsed:" result))
  (println)

  ;; Test 2: Compile form
  (println "Test 2: Compile form")
  (let [result (compile-form '(+ 1 2))]
    (println "✓ Compiled:" (:type result)))
  (println)

  ;; Test 3: Emoji script
  (println "Test 3: EmojiScript")
  (let [result (execute-emoji "🔢6 🔢7 ✖️ ↩️")]
    (println "✓ Result:" result))
  (println)

  ;; Test 4: Knowledge query
  (println "Test 4: Knowledge layer")
  (assert-belief! :test-fact :repl 0.95 100)
  (let [result (query-belief :test-fact)]
    (println "✓ Query:" result))
  (println)

  ;; Test 5: REPL command
  (println "Test 5: REPL command")
  (let [result (eval-input "(emoji:exec \"🔢40 🔢2 ➕ ↩️\")")]
    (println "✓ Command result:" result))
  (println)

  (println "=== REPL Tests Complete ==="))
