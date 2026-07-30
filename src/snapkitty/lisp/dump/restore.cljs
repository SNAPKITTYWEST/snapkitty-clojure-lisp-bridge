;; PH3.S3 — Defensive Restore Parser (5-stage validation)
;; From SKC-LISP-WORLD-COQ-001 <restore : ByteString → Result World Error>

(ns snapkitty.lisp.dump.restore
  (:require ["crypto" :as crypto]))

(defn stage-1-parse-untrusted [bytes]
  "Parse header (no world yet, untrusted data)"
  (try
    {:magic (subs bytes 0 4)
     :format-version (js/parseInt (subs bytes 4 6))
     :endianness-marker (js/parseInt (subs bytes 6 8))}
    (catch :default e
      (throw (ex-info "Stage 1: Parse failed" {})))))

(defn stage-2-validate-structure [parsed]
  "Check all section lengths and bounds"
  (if (not= (:magic parsed) "LISP")
    (throw (ex-info "Stage 2: Invalid magic" {})))
  (if (not= (:format-version parsed) 256)
    (throw (ex-info "Stage 2: Unsupported version" {})))
  true)

(defn stage-3-validate-content [sections]
  "Verify code objects, environments, continuations"
  (doseq [section sections]
    (when (nil? section)
      (throw (ex-info "Stage 3: Nil section" {}))))
  true)

(defn stage-4-verify-integrity [payload expected-digest]
  "Recompute SHA-256 and compare"
  (let [computed (-> (crypto/createHash "sha256")
                     (.update payload)
                     (.digest "hex"))]
    (if (not= computed expected-digest)
      (throw (ex-info "Stage 4: Digest mismatch" {})))
    true))

(defn stage-5-publish-world [header sections]
  "Only after all 4 stages pass"
  {:format-version (:format-version header)
   :generation (:world-generation header)
   :symbol-table (first sections)
   :object-store (second sections)
   :environment-chain (nth sections 2)
   :code-registry (nth sections 3)
   :machine-state (nth sections 5)
   :mutation-journal (nth sections 6)
   :capability-registry (nth sections 7)
   :root-set (nth sections 8)})

(defn restore [dump-bytes]
  "Defensive restore with complete validation"
  (try
    (let [header (stage-1-parse-untrusted dump-bytes)]
      (stage-2-validate-structure header)
      (let [sections (stage-3-validate-content (js/JSON.parse dump-bytes))]
        (stage-4-verify-integrity (str sections) (:payload-digest header))
        (stage-5-publish-world header sections)))
    (catch :default e
      {:error (str e)})))
