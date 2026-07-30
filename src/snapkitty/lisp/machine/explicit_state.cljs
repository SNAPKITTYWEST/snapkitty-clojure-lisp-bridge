;; PH2.S1 — Explicit Machine State (Runtime)
;; From SKC-LISP-WORLD-COQ-001 <machine-state>

(ns snapkitty.lisp.machine.explicit-state
  "Non-recursive explicit machine state")

(defrecord MachineState
  [world-generation
   control
   value-stack
   frame-stack
   environment
   heap
   code-store
   program-counter
   dynamic-context
   handlers
   pending-effects
   mutation-log
   status])

(defn initial-state []
  (->MachineState
    1                    ; world-generation
    :fetch               ; control
    []                   ; value-stack
    []                   ; frame-stack
    0                    ; environment
    {}                   ; heap
    {}                   ; code-store
    [0 0]                ; program-counter [code-id offset]
    {}                   ; dynamic-context
    []                   ; handlers
    []                   ; pending-effects
    []                   ; mutation-log
    :running))           ; status

(defn push-value [state val]
  (update state :value-stack conj val))

(defn pop-value [state]
  (if (empty? (:value-stack state))
    (throw (ex-info "Value stack underflow" {}))
    [(peek (:value-stack state))
     (update state :value-stack pop)]))

(defn push-frame [state frame]
  (update state :frame-stack conj frame))

(defn pop-frame [state]
  (if (empty? (:frame-stack state))
    (throw (ex-info "Frame stack underflow" {}))
    [(peek (:frame-stack state))
     (update state :frame-stack pop)]))

(defn well-formed? [state]
  (and (> (:world-generation state) 0)
       (or (not= (:program-counter state) [0 0])
           (= (:status state) :halted))))
