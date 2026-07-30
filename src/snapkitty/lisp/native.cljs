(ns snapkitty.lisp.native
  "Runtime binding to NASM validators (mutation gate + digest verification)"
  (:require [promesa.core :as p]))

;; ============================================================================
;; Load native module at startup
;; ============================================================================

(def native-lib (atom nil))
(def lib-loaded? (atom false))

(defn load-native-library! [lib-path]
  "Load the compiled NASM + Node.js binding library
   lib-path: path to .node file (e.g., './native/build/Release/skclisp_native.node')"
  (p/let [lib (require lib-path)]
    (reset! native-lib lib)
    (reset! lib-loaded? true)
    (js/console.log "[SKC-LISP] Native ASM library loaded:" lib-path)
    true))

;; ============================================================================
;; Mutation Validation Gate (8-point check)
;; ============================================================================

(defn validate-mutation! [mutation-event object-store]
  "Fast-path mutation validation using NASM

   Returns promise with validation result:
   {:passes-gate boolean    ; true if all 8 checks pass
    :error-code number      ; 0-8 for failed check, 255 for all pass
    :details string}        ; human-readable error message"

  (if-not @lib-loaded?
    (p/rejected (js/Error. "Native library not loaded"))

    (p/let [lib @native-lib

            ; Prepare mutation_event buffer (64 bytes)
            ; Layout matches NASM struct:
            ; [0-8]    mutation_id
            ; [8-16]   generation_before
            ; [16-24]  generation_after
            ; [24-32]  actor
            ; [32-40]  target
            ; [40-44]  operation
            ; [44-48]  reserved
            ; [48-56]  old_digest (pointer)
            ; [56-64]  new_digest (pointer)

            mutation-buf (js/Uint8Array. 64)
            result-buf (js/Uint8Array. 2)

            ; Copy fields into buffer (simplified for now)
            _ (doseq [[i val] (mapcat
                               (fn [[offset v]] [(offset 0) v])
                               {0 (:mutation-id mutation-event)
                                8 (:generation-before mutation-event)
                                16 (:generation-after mutation-event)
                                24 (:actor mutation-event)
                                32 (:target mutation-event)})]
                (aset mutation-buf i val))

            ; Call NASM function
            ret (.validateMutation lib mutation-buf
                                   (or object-store 0)
                                   result-buf)

            passes? (= (aget result-buf 0) 1)
            error-code (aget result-buf 1)

            error-msg (case error-code
                        0 "Check 1 failed: target does not exist"
                        1 "Check 2 failed: old_digest mismatch"
                        2 "Check 3 failed: new_digest mismatch"
                        3 "Check 4 failed: replacement not well-formed"
                        4 "Check 5 failed: invalid references"
                        5 "Check 6 failed: invalid code"
                        6 "Check 7 failed: invariants not preserved"
                        7 "Check 8 failed: generation not monotonic"
                        255 "All checks passed"
                        "Unknown error")]

      {:passes-gate passes?
       :error-code error-code
       :details error-msg
       :native-result ret})))

;; ============================================================================
;; Blake3 Digest Verification
;; ============================================================================

(defn verify-blake3! [payload expected-digest]
  "Fast-path Blake3 verification using NASM

   Returns promise with verification result:
   {:digest-valid boolean   ; true if payload matches expected digest
    :error-code number      ; 0=match, 1=mismatch, 2=invalid_input
    :details string}"

  (if-not @lib-loaded?
    (p/rejected (js/Error. "Native library not loaded"))

    (p/let [lib @native-lib

            ; Convert payload to Uint8Array if needed
            payload-buf (if (instance? js/Uint8Array payload)
                          payload
                          (js/Uint8Array. (.from js/Array payload)))

            ; Convert expected digest
            digest-buf (if (instance? js/Uint8Array expected-digest)
                         expected-digest
                         (js/Uint8Array. (.from js/Array expected-digest)))

            result-buf (js/Uint8Array. 2)

            ; Call NASM function
            ret (.verifyBlake3 lib payload-buf digest-buf result-buf)

            valid? (= (aget result-buf 0) 1)
            error-code (aget result-buf 1)

            error-msg (case error-code
                        0 "Digest matches"
                        1 "Digest mismatch"
                        2 "Invalid input (null or malformed)"
                        "Unknown error")]

      {:digest-valid valid?
       :error-code error-code
       :details error-msg
       :native-result ret})))

;; ============================================================================
;; Ed25519 Signature Verification
;; ============================================================================

(defn verify-ed25519! [message signature public-key]
  "Fast-path Ed25519 verification using NASM

   Returns promise with verification result:
   {:signature-valid boolean ; true if signature is valid
    :error-code number       ; 0=valid, 1=invalid, 2=invalid_input
    :details string}"

  (if-not @lib-loaded?
    (p/rejected (js/Error. "Native library not loaded"))

    (p/let [lib @native-lib

            msg-buf (if (instance? js/Uint8Array message)
                      message
                      (js/Uint8Array. (.from js/Array message)))

            sig-buf (if (instance? js/Uint8Array signature)
                      signature
                      (js/Uint8Array. (.from js/Array signature)))

            key-buf (if (instance? js/Uint8Array public-key)
                      public-key
                      (js/Uint8Array. (.from js/Array public-key)))

            result-buf (js/Uint8Array. 2)

            ; Call NASM function
            ret (.verifyEd25519 lib msg-buf sig-buf key-buf result-buf)

            valid? (= (aget result-buf 0) 1)
            error-code (aget result-buf 1)

            error-msg (case error-code
                        0 "Signature is valid"
                        1 "Signature is invalid"
                        2 "Invalid input"
                        "Unknown error")]

      {:signature-valid valid?
       :error-code error-code
       :details error-msg
       :native-result ret})))

;; ============================================================================
;; Export for MCP tools
;; ============================================================================

(def native-api
  {:load-library load-native-library!
   :validate-mutation validate-mutation!
   :verify-blake3 verify-blake3!
   :verify-ed25519 verify-ed25519!
   :is-loaded? (fn [] @lib-loaded?)})
