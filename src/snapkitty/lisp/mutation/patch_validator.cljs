;; PH2.S3 — Code Patch Validator
;; From SKC-LISP-WORLD-COQ-001 <instruction name="PATCH_CODE">

(ns snapkitty.lisp.mutation.patch-validator
  "Validate code patches before application")

(def valid-opcodes
  #{:const :lookup :bind :push :pop :cons :car :cdr
    :set-car :set-cdr :make-closure :call :tail-call
    :return :jump :jump-if-false :push-frame :pop-frame
    :capture-continuation :restore-continuation :raise
    :install-handler :remove-handler :request-effect
    :patch-code :define-code :replace-function
    :rewrite-dispatch :commit-generation :rollback-generation :halt})

(defn validate-patch [instructions]
  "All patch validation rules from XML"
  (let [errors []]
    (if-not (coll? instructions)
      (conj errors "Instructions not a collection"))
    (doseq [inst instructions]
      (when-not (contains? valid-opcodes (first inst))
        (conj errors (str "Invalid opcode: " (first inst)))))
    errors))

(defn patch-gate-passed? [old-code new-code]
  "All preconditions from mutation-gate"
  (and (not= old-code new-code)
       (empty? (validate-patch new-code))))

(defn apply-patch [code-object old-range new-instructions]
  "Replace instruction range with validated new sequence"
  (if (patch-gate-passed? old-range new-instructions)
    (assoc code-object :instructions new-instructions)
    (throw (ex-info "Patch validation failed"
                    {:old-range old-range :new new-instructions}))))
