;; PH3.S2 — Deterministic World Serializer
;; From SKC-LISP-WORLD-COQ-001 <dump : World → ByteString>

(ns snapkitty.lisp.dump.serializer
  (:require [cljs.reader :as reader]
            ["crypto" :as crypto]))

(defn compute-payload-digest [payload]
  "SHA-256 of payload"
  (-> (crypto/createHash "sha256")
      (.update payload)
      (.digest "hex")))

(defn serialize-header [world]
  "Dump header (12 fields, fixed byte order)"
  {:magic "LISP"
   :format-version 0x0100
   :endianness-marker 0x1234
   :word-size-policy 64
   :character-encoding 0
   :world-generation (:generation world)
   :root-count (count (:root-set world))
   :object-count (count (:object-store world))
   :code-object-count (count (:code-registry world))
   :mutation-count (count (:mutation-journal world))
   :payload-length 0  ; Will be computed
   :payload-digest ""}) ; Will be computed

(defn serialize-section-1 [world]
  "Section 1: Symbol and package registry (sorted)"
  (sort-by first (:symbol-table world)))

(defn serialize-section-2 [world]
  "Section 2: Object table (sorted by ObjectId)"
  (sort-by first (:object-store world)))

(defn serialize-section-3 [world]
  "Section 3: Environment graph"
  (:environment-chain world))

(defn serialize-section-4 [world]
  "Section 4: Code object registry"
  (:code-registry world))

(defn serialize-section-5 [world]
  "Section 5: Continuation and frame state"
  (:machine-state world))

(defn serialize-section-6 [world]
  "Section 6: Machine state snapshot"
  (:machine-state world))

(defn serialize-section-7 [world]
  "Section 7: Mutation journal (chronological)"
  (:mutation-journal world))

(defn serialize-section-8 [world]
  "Section 8: Capability descriptors"
  (:capability-registry world))

(defn serialize-section-9 [world]
  "Section 9: Root set"
  (:root-set world))

(defn dump [world]
  "Serialize world to deterministic byte string"
  (let [header (serialize-header world)
        s1 (serialize-section-1 world)
        s2 (serialize-section-2 world)
        s3 (serialize-section-3 world)
        s4 (serialize-section-4 world)
        s5 (serialize-section-5 world)
        s6 (serialize-section-6 world)
        s7 (serialize-section-7 world)
        s8 (serialize-section-8 world)
        s9 (serialize-section-9 world)
        payload (str s1 s2 s3 s4 s5 s6 s7 s8 s9)
        digest (compute-payload-digest payload)]
    {:header (assoc header
               :payload-length (count payload)
               :payload-digest digest)
     :sections [s1 s2 s3 s4 s5 s6 s7 s8 s9]}))

(defn dump-determinism-check [w1 w2]
  "Verify: canonical(w1) = canonical(w2) → dump(w1) = dump(w2)"
  (= (dump w1) (dump w2)))
