(ns snapkitty.lisp.emojiscript
  "Ahmad's EmojiScript bytecode dialect — emoji tokens → SigilOp bytecode"
  (:require [goog.string :as gstring]
            [goog.string.format]))

;; ============================================================================
;; SigilOp Bytecode Instructions
;; ============================================================================

(def bin-ops
  {"++" :Add
   "--" :Sub
   "**" :Mul
   "//" :Div
   "&&" :And
   "||" :Or
   "^^" :Xor})

(defn- bin-op? [k]
  (contains? bin-ops k))

;; ============================================================================
;; EmojiToken AST
;; ============================================================================

(def emoji-tokens
  {
   ; Numeric literal
   "🔢" :Number

   ; Binary ops
   "➕" :Add
   "➖" :Sub
   "✖" :Mul
   "✖️" :Mul
   "❌" :Mul
   "➗" :Div
   "🤝" :And
   "🔗" :And
   "👐" :Or
   "🔀" :Or
   "🌀" :Xor
   "⊕" :Xor

   ; Control & memory
   "🔑" :CapGate    ; (pops 2 args: slot, rights)
   "⚡" :Call       ; (pops 1 arg: func_idx)
   "🏗️" :Alloc      ; (pops 2 args: size, type_tag)
   "📤" :Load       ; (pops 1 arg: offset)
   "📦" :Store      ; (pops 1 arg: offset)
   "➡️" :Jump       ; (pops 1 arg: target)
   "❓" :JumpIf     ; (pops 1 arg: target)
   "↩" :Ret
   "↩️" :Ret
   "🔙" :Ret
   "🔚" :Ret

   ; Future semantic passes (Sprint 2)
   "🌊" :Stream     ; routes to telemetry-bus
   "🧠" :PolicyCheck ; routes to policy-immune
   "🔒" :Seal       ; Bifrost seal hint
   "🔓" :ReadOnly   ; Downgrade rights hint
  })

;; ============================================================================
;; Lexer — parse grapheme by grapheme (Unicode-aware)
;; ============================================================================

(defn- split-graphemes [s]
  "Split string into grapheme clusters (emoji-safe)"
  (let [arr (js-array)]
    (doseq [char (seq s)]
      (.push arr (str char)))
    (vec arr)))

(defn- pop-number [graphemes idx]
  "Parse 🔢<digits> starting at idx, return [number, next_idx]"
  (let [start (inc idx)
        digits (atom "")]
    (loop [i start]
      (if (< i (count graphemes))
        (let [g (get graphemes i)]
          (if (re-matches #"^\d$" g)
            (do
              (swap! digits str g)
              (recur (inc i)))
            [i @digits]))
        [i @digits]))))

(defn- lex [input]
  "Tokenize EmojiScript input → vector of SigilOp maps"
  (let [graphemes (split-graphemes input)
        ops (atom [])]
    (loop [idx 0]
      (if (>= idx (count graphemes))
        @ops
        (let [g (get graphemes idx)]
          (cond
            ; Skip whitespace
            (re-matches #"^\s$" g)
            (recur (inc idx))

            ; Number: 🔢<digits>
            (= g "🔢")
            (let [[next-idx digits-str] (pop-number graphemes idx)]
              (if (empty? digits-str)
                (throw (ex-info "Invalid number after 🔢" {:input input :at idx}))
                (let [n (js/parseInt digits-str 10)]
                  (swap! ops conj {:op :Push :value n})
                  (recur next-idx))))

            ; Zero-arg ops (return immediately)
            (contains? #{:Add :Sub :Mul :Div :And :Or :Xor :Ret :Stream :PolicyCheck :Seal :ReadOnly} (get emoji-tokens g))
            (do
              (swap! ops conj {:op (get emoji-tokens g)})
              (recur (inc idx)))

            ; Arg-consuming ops (need to pop preceding 🔢 pushes)
            (= g "🔑")
            (let [;; Pop two preceding Push ops (rights, then slot)
                  rights-op (when (> (count @ops) 0) (peek @ops))
                  _ (when-not (= (:op rights-op) :Push)
                      (throw (ex-info "🔑 requires 🔢 arguments before it" {:at idx})))
                  rights (-> @ops pop (.-value))
                  _ (swap! ops pop)

                  slot-op (when (> (count @ops) 0) (peek @ops))
                  _ (when-not (= (:op slot-op) :Push)
                      (throw (ex-info "🔑 requires 2 🔢 arguments before it" {:at idx})))
                  slot (-> @ops peek (.-value))
                  _ (swap! ops pop)]
              (swap! ops conj {:op :CapGate :slot slot :rights rights})
              (recur (inc idx)))

            (= g "⚡")
            (let [func-op (peek @ops)]
              (when-not (= (:op func-op) :Push)
                (throw (ex-info "⚡ requires 🔢 argument before it" {:at idx})))
              (let [func (-> @ops peek (.-value))]
                (swap! ops pop)
                (swap! ops conj {:op :Call :func func})
                (recur (inc idx))))

            (= g "🏗️")
            (let [type-op (peek @ops)]
              (when-not (= (:op type-op) :Push)
                (throw (ex-info "🏗️ requires 2 🔢 arguments before it" {:at idx})))
              (let [type-tag (-> @ops peek (.-value))
                    _ (swap! ops pop)
                    size-op (peek @ops)
                    _ (when-not (= (:op size-op) :Push)
                        (throw (ex-info "🏗️ requires 2 🔢 arguments before it" {:at idx})))
                    size (-> @ops peek (.-value))
                    _ (swap! ops pop)]
                (swap! ops conj {:op :Alloc :size size :type-tag type-tag})
                (recur (inc idx))))

            (= g "📤")
            (let [offset-op (peek @ops)]
              (when-not (= (:op offset-op) :Push)
                (throw (ex-info "📤 requires 🔢 argument before it" {:at idx})))
              (let [offset (-> @ops peek (.-value))]
                (swap! ops pop)
                (swap! ops conj {:op :Load :offset offset})
                (recur (inc idx))))

            (= g "📦")
            (let [offset-op (peek @ops)]
              (when-not (= (:op offset-op) :Push)
                (throw (ex-info "📦 requires 🔢 argument before it" {:at idx})))
              (let [offset (-> @ops peek (.-value))]
                (swap! ops pop)
                (swap! ops conj {:op :Store :offset offset})
                (recur (inc idx))))

            (= g "➡️")
            (let [target-op (peek @ops)]
              (when-not (= (:op target-op) :Push)
                (throw (ex-info "➡️ requires 🔢 argument before it" {:at idx})))
              (let [target (-> @ops peek (.-value))]
                (swap! ops pop)
                (swap! ops conj {:op :Jump :target target})
                (recur (inc idx))))

            (= g "❓")
            (let [target-op (peek @ops)]
              (when-not (= (:op target-op) :Push)
                (throw (ex-info "❓ requires 🔢 argument before it" {:at idx})))
              (let [target (-> @ops peek (.-value))]
                (swap! ops pop)
                (swap! ops conj {:op :JumpIf :target target})
                (recur (inc idx))))

            ; Unknown emoji
            :else
            (throw (ex-info (str "Unknown emoji: " g) {:input input :at idx})))))))

  (if (empty? @ops)
    (throw (ex-info "Empty program" {:input input}))
    @ops))

;; ============================================================================
;; Compilation to bytecode
;; ============================================================================

(defn compile-emojiscript [input]
  "Compile EmojiScript source → bytecode object

   Returns:
   {:source string
    :bytecode [ops]
    :hash string
    :instructions-count number}"
  (let [bytecode (lex input)
        source-hash (js/btoa (js/encodeURIComponent input))
        instructions-count (count bytecode)]
    {:source input
     :bytecode bytecode
     :hash source-hash
     :instructions-count instructions-count
     :valid? true}))

;; ============================================================================
;; Execution (stack-based interpreter)
;; ============================================================================

;; ============================================================================
;; Semantic Passes (Phase 2)
;; ============================================================================

(defn- telemetry-log [event data]
  "Log event to telemetry-bus (stub for now)"
  {:type :telemetry :event event :data data :timestamp (js/Date.now)})

(defn- policy-route [value policy-id]
  "Route to policy-immune handler (stub for now)"
  {:type :policy-route :value value :policy-id policy-id})

(defn- bifrost-seal [value seal-id worm-ledger]
  "Seal value to WORM ledger (stub for now)"
  {:type :bifrost-seal :value value :seal-id seal-id :ledger worm-ledger})

(defn- capability-downgrade [rights from-level to-level]
  "Downgrade capability rights (stub for now)"
  (max 0 (bit-shift-right rights (- from-level to-level))))

;; ============================================================================
;; Execution (stack-based interpreter)
;; ============================================================================

(defn execute-emojiscript [bytecode & {:keys [max-steps telemetry-bus policy-registry worm-ledger]
                                       :or {max-steps 10000}}]
  "Execute compiled EmojiScript bytecode with semantic passes

   Returns:
   {:result (top of stack)
    :stack [values]
    :steps number
    :halted? boolean
    :events [telemetry events]}"
  (let [stack (atom [])
        pc (atom 0)
        steps (atom 0)
        halted? (atom false)
        events (atom [])]

    (try
      (while (and (< @pc (count bytecode))
                  (< @steps max-steps)
                  (not @halted?))
        (let [instr (get bytecode @pc)]
          (swap! steps inc)

          (case (:op instr)
            :Push
            (do
              (swap! stack conj (:value instr))
              (swap! pc inc))

            :Add
            (let [b (peek @stack) _ (swap! stack pop)
                  a (peek @stack) _ (swap! stack pop)]
              (swap! stack conj (+ a b))
              (swap! pc inc))

            :Sub
            (let [b (peek @stack) _ (swap! stack pop)
                  a (peek @stack) _ (swap! stack pop)]
              (swap! stack conj (- a b))
              (swap! pc inc))

            :Mul
            (let [b (peek @stack) _ (swap! stack pop)
                  a (peek @stack) _ (swap! stack pop)]
              (swap! stack conj (* a b))
              (swap! pc inc))

            :Div
            (let [b (peek @stack) _ (swap! stack pop)
                  a (peek @stack) _ (swap! stack pop)]
              (if (= b 0)
                (throw (ex-info "Division by zero" {}))
                (do
                  (swap! stack conj (quot a b))
                  (swap! pc inc))))

            :And
            (let [b (peek @stack) _ (swap! stack pop)
                  a (peek @stack) _ (swap! stack pop)]
              (swap! stack conj (bit-and a b))
              (swap! pc inc))

            :Or
            (let [b (peek @stack) _ (swap! stack pop)
                  a (peek @stack) _ (swap! stack pop)]
              (swap! stack conj (bit-or a b))
              (swap! pc inc))

            :Xor
            (let [b (peek @stack) _ (swap! stack pop)
                  a (peek @stack) _ (swap! stack pop)]
              (swap! stack conj (bit-xor a b))
              (swap! pc inc))

            :CapGate
            (do
              ; For now, just verify both args are present (no-op gate)
              (swap! pc inc))

            :Call
            (throw (ex-info "Function calls not yet supported" {}))

            :Alloc
            (throw (ex-info "Memory allocation not yet supported" {}))

            :Load
            (throw (ex-info "Memory load not yet supported" {}))

            :Store
            (throw (ex-info "Memory store not yet supported" {}))

            :Jump
            (swap! pc (:target instr))

            :JumpIf
            (let [cond (peek @stack) _ (swap! stack pop)]
              (if (zero? cond)
                (swap! pc inc)
                (swap! pc (:target instr))))

            :Ret
            (reset! halted? true)

            :Stream
            (let [value (peek @stack)]
              (if value
                (do
                  (swap! events conj (telemetry-log :stream-push {:value value}))
                  (swap! pc inc))
                (throw (ex-info "🌊 Stream requires value on stack" {}))))

            :PolicyCheck
            (let [value (peek @stack)
                  policy-id (:policy-id instr 0)]
              (if value
                (do
                  (swap! events conj (policy-route value policy-id))
                  (swap! pc inc))
                (throw (ex-info "🧠 PolicyCheck requires value on stack" {}))))

            :Seal
            (let [value (peek @stack)
                  seal-id (:seal-id instr (str "seal-" @steps))]
              (if value
                (do
                  (swap! events conj (bifrost-seal value seal-id worm-ledger))
                  (swap! pc inc))
                (throw (ex-info "🔒 Seal requires value on stack" {}))))

            :ReadOnly
            (let [rights (peek @stack)
                  from-level (:from-level instr 7)
                  to-level (:to-level instr 3)]
              (if rights
                (do
                  (let [downgraded (capability-downgrade rights from-level to-level)]
                    (swap! stack pop)
                    (swap! stack conj downgraded))
                  (swap! events conj {:type :capability-downgrade :from from-level :to to-level})
                  (swap! pc inc))
                (throw (ex-info "🔓 ReadOnly requires capability on stack" {}))))

            (throw (ex-info "Unknown instruction" {:instr instr})))))

      {:result (peek @stack)
       :stack (vec @stack)
       :steps @steps
       :halted? @halted?
       :events @events}

      (catch js/Error e
        {:error (.-message e)
         :stack (vec @stack)
         :steps @steps
         :halted? false
         :events @events}))))

;; ============================================================================
;; Public API
;; ============================================================================

(def emojiscript-api
  {:compile compile-emojiscript
   :execute execute-emojiscript})
