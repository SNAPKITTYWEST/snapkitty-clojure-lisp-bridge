;; SKC-LISP: WASM Bridge for Browser Integration (Phase 3D-3)
;; Loads + exposes Rust WASM crypto functions to ClojureScript

(ns snapkitty.lisp.wasm-bridge
  (:require-macros [cljs.core :refer [js-inline]])
  (:require ["../../../native/pkg/skclisp_crypto_wasm.js" :as wasm-module]))

;; ============================================================================
;; WASM Module Lifecycle
;; ============================================================================

(def wasm-ready? (atom false))

(defn init-wasm!
  "Initialize WASM module (must be called before using crypto functions)."
  []
  (let [promise (wasm-module/default)]
    (-> promise
        (.then (fn [_]
                 (reset! wasm-ready? true)
                 (println "✓ WASM module initialized")))
        (.catch (fn [err]
                  (println "✗ WASM initialization failed:" err)
                  (throw err))))))

(defn ensure-wasm-ready []
  "Throw error if WASM not initialized."
  (if-not @wasm-ready?
    (throw (ex-info "WASM module not initialized. Call init-wasm! first." {}))))

;; ============================================================================
;; Blake3 WASM Functions
;; ============================================================================

(defn blake3-hash
  "Compute Blake3 digest of input bytes (returns 32-byte array)."
  [input]
  (ensure-wasm-ready)
  (let [input-array (if (array? input)
                      input
                      (js/Buffer.from (apply str input)))
        hash-vec (wasm-module/blake3_hash input-array)]
    (js/Buffer.from (js/Array.from hash-vec))))

(defn blake3-verify
  "Verify Blake3 digest matches expected value.

   Returns:
     {:valid? bool
      :error-code u8
      :message string}"
  [payload expected-digest]
  (ensure-wasm-ready)
  (let [payload-buf (if (array? payload) payload (js/Buffer.from payload))
        digest-buf (if (array? expected-digest) expected-digest (js/Buffer.from expected-digest))
        result (wasm-module/blake3_verify_wasm payload-buf digest-buf)]

    {:valid? (.-valid result)
     :error-code (.-error_code result)
     :message (case (.-error_code result)
                0 "Digest match"
                1 "Digest mismatch"
                2 "Invalid input"
                "Unknown error")}))

;; ============================================================================
;; Ed25519 WASM Functions
;; ============================================================================

(defn ed25519-verify
  "Verify Ed25519 signature.

   Returns:
     {:valid? bool
      :error-code u8
      :message string}"
  [message signature public-key]
  (ensure-wasm-ready)
  (let [msg-buf (if (array? message) message (js/Buffer.from message))
        sig-buf (if (array? signature) signature (js/Buffer.from signature))
        pk-buf (if (array? public-key) public-key (js/Buffer.from public-key))
        result (wasm-module/ed25519_verify_wasm msg-buf sig-buf pk-buf)]

    {:valid? (.-valid result)
     :error-code (.-error_code result)
     :message (case (.-error_code result)
                0 "Signature valid"
                1 "Signature invalid"
                2 "Invalid input"
                "Unknown error")}))

;; ============================================================================
;; Mutation Validation WASM Function
;; ============================================================================

(defn validate-mutation
  "8-point mutation validation gate (WASM version).

   Returns:
     {:valid? bool
      :error-code u8
      :message string}"
  [event-id generation source-hash bytecode-hash native-code-hash actor-signature]
  (ensure-wasm-ready)
  (let [result (wasm-module/validate_mutation_wasm
                 event-id
                 generation
                 source-hash
                 bytecode-hash
                 native-code-hash
                 actor-signature)]

    {:valid? (.-valid result)
     :error-code (.-error_code result)
     :message (.-message result)}))

;; ============================================================================
;; Proof Certificate Validation WASM Function
;; ============================================================================

(defn validate-proof-certificate
  "Validate 157-byte proof certificate structure + signature.

   Returns:
     {:valid? bool
      :theorem-id u32
      :theorems-covered u32
      :error-code u8
      :message string}"
  [cert-bytes]
  (ensure-wasm-ready)
  (let [cert-buf (if (array? cert-bytes) cert-bytes (js/Buffer.from cert-bytes))
        result (wasm-module/validate_proof_certificate_wasm cert-buf)]

    {:valid? (.-valid result)
     :theorem-id (.-theorem_id result)
     :theorems-covered (.-theorems_covered result)
     :error-code (.-error_code result)
     :message (.-message result)}))

;; ============================================================================
;; High-Level Browser API
;; ============================================================================

(defn compile-with-proof-browser
  "Browser-native JIT compilation pipeline: source → bytecode → native WASM → execute.

   Returns:
     {:status 'compiled'|'error'
      :bytecode <bytes>
      :proof-id <u32>
      :blake3-verified? <bool>
      :ed25519-verified? <bool>}"
  [source proof-cert]
  (ensure-wasm-ready)

  (try
    ;; Step 1: Parse source → bytecode
    (let [bytecode (emoji/compile-emojiscript source)

          ;; Step 2: Verify proof certificate structure
          cert-validation (validate-proof-certificate
                            (js/Buffer.from proof-cert))

          ;; Step 3: Blake3 verify source against expected digest
          blake3-check (blake3-verify
                         (js/Buffer.from (.getBytes source))
                         (:expected-source-digest proof-cert))

          ;; Step 4: Ed25519 verify proof signature
          ed25519-check (ed25519-verify
                          (js/Buffer.from (.getBytes source))
                          (:signature proof-cert)
                          (:public-key proof-cert))]

      {:status (if (and (:valid? cert-validation)
                       (:valid? blake3-check)
                       (:valid? ed25519-check))
                 "compiled"
                 "error")
       :bytecode (:bytecode bytecode)
       :proof-id (:theorem-id cert-validation)
       :blake3-verified? (:valid? blake3-check)
       :ed25519-verified? (:valid? ed25519-check)
       :errors (concat
                 (if-not (:valid? cert-validation)
                   [(:message cert-validation)]
                   [])
                 (if-not (:valid? blake3-check)
                   [(:message blake3-check)]
                   [])
                 (if-not (:valid? ed25519-check)
                   [(:message ed25519-check)]
                   []))})

    (catch js/Error e
      {:status "error"
       :error (.toString e)})))

;; ============================================================================
;; Dashboard Integration (GitHub Pages)
;; ============================================================================

(defn update-dashboard-metrics!
  "Update browser dashboard with live crypto metrics."
  [compile-result container-id]
  (let [container (js/document.getElementById container-id)]
    (when container
      (set! (.-innerHTML container)
            (str "<div style='font-family: monospace; color: #0d0;'>"
                 "<div>Status: " (:status compile-result) "</div>"
                 "<div>Proof ID: 0x" (.toString (:proof-id compile-result) 16) "</div>"
                 "<div>Blake3 Verified: " (if (:blake3-verified? compile-result) "✓" "✗") "</div>"
                 "<div>Ed25519 Verified: " (if (:ed25519-verified? compile-result) "✓" "✗") "</div>"
                 (if (:errors compile-result)
                   (str "<div style='color: #f44;'>Errors:<ul>"
                        (reduce (fn [html err]
                                  (str html "<li>" err "</li>"))
                                ""
                                (:errors compile-result))
                        "</ul></div>")
                   "")
                 "</div>")))))

;; ============================================================================
;; Export Functions for Global Access
;; ============================================================================

(defn ^:export init-crypto []
  "Initialize WASM crypto module (call from browser console)."
  (init-wasm!))

(defn ^:export verify-blake3-browser [payload digest]
  "Browser console: verify Blake3 digest."
  (blake3-verify payload digest))

(defn ^:export verify-ed25519-browser [message signature public-key]
  "Browser console: verify Ed25519 signature."
  (ed25519-verify message signature public-key))

(defn ^:export compile-and-verify [source proof-cert]
  "Browser console: full compile + verify pipeline."
  (compile-with-proof-browser source proof-cert))

;; ============================================================================
;; Diagnostic Functions
;; ============================================================================

(defn diagnostic-report []
  "Generate diagnostic report for troubleshooting."
  {:wasm-ready? @wasm-ready?
   :browser-type (.-userAgent js/navigator)
   :local-storage-available? (try
                               (let [key "__test__"]
                                 (js/localStorage.setItem key "1")
                                 (js/localStorage.removeItem key)
                                 true)
                               (catch js/Error _ false))
   :worker-support? (boolean (.-Worker js/window))
   :crypto-subtle-available? (boolean (-> js/window .-crypto .-subtle))})

(defn ^:export print-diagnostics []
  "Print browser diagnostic info to console."
  (println (clj->js (diagnostic-report))))
