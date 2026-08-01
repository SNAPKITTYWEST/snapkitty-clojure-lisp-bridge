<!DOCTYPE style-sheet PUBLIC "-//James Clark//DTD DSSSL Style Sheet//EN">

(style-sheet

 ;; ========================================================================
 ;; 1. RELATIONAL SYNTHESIS & UNIFICATION KERNEL
 ;; ========================================================================
 ;; This style-sheet IS the inference engine.
 ;; Construction rules (element ...) transform the SGML grove.
 ;; The synthesis kernel (below) fills holes before grove traversal begins.

 ;; Simple association list substitution lookup
 (define (lookup var env)
   (let ((cell (assoc var env)))
     (if cell (cdr cell) #f)))

 ;; Unifies a HOLE variable with a candidate replacement value
 ;; Returns extended env on success, #f on contradiction
 (define (unify-hole var candidate env)
   (let ((bound (lookup var env)))
     (cond
      ((not bound) (cons (cons var candidate) env))
      ((equal? bound candidate) env)
      (else #f))))

 ;; Evaluates numerical results across the synthesized AST grove
 ;; Mirrors eval-grove-node in dsssl-synthesis.mjs exactly
 (define (eval-grove-node node env)
   (let ((tag (gi node)))
     (cond
      ((string=? tag "INT")
       (string->number (data node)))
      ((string=? tag "HOLE")
       (let* ((var (attribute-string "var" node))
              (val (lookup var env)))
         (if val val (quote UNBOUND))))
      ((string=? tag "EXPR")
       (let* ((op-node    (node-list-first (select-elements (children node) "OP")))
              (left-node  (node-list-first (select-elements (children node) "LEFT")))
              (right-node (node-list-first (select-elements (children node) "RIGHT")))
              (op    (data op-node))
              (l-val (eval-grove-node (node-list-first (children left-node))  env))
              (r-val (eval-grove-node (node-list-first (children right-node)) env)))
         (cond
          ((or (equal? l-val (quote UNBOUND))
               (equal? r-val (quote UNBOUND))) (quote UNBOUND))
          ((string=? op "+") (+ l-val r-val))
          ((string=? op "*") (* l-val r-val))
          (else 0))))
      (else 0))))

 ;; Target invariant: the root EXPR must evaluate to exactly 20
 (define *target-invariant-value* 20)

 ;; Synthesizes bindings by exhaustive candidate search against the invariant.
 ;; Returns the first satisfying environment, or #f if none found.
 ;; This is the SMT oracle: unify-hole + eval = lightweight SMT over integers.
 (define (synthesize-bindings hole-var candidates env)
   (let loop ((rest-candidates candidates))
     (if (null? rest-candidates)
         #f
         (let* ((cand     (car rest-candidates))
                (test-env (unify-hole hole-var cand env)))
           (if test-env
               (if (= (eval-grove-node (root-element) test-env)
                      *target-invariant-value*)
                   test-env
                   (loop (cdr rest-candidates)))
               (loop (cdr rest-candidates)))))))

 ;; ========================================================================
 ;; 2. DSSSL CONSTRUCTION RULES (SGML GROVE TRANSFORMATIONS)
 ;; ========================================================================
 ;; Each (element TAG ...) rule fires when the grove walker visits that tag.
 ;; The synthesis kernel runs first (in GROVE rule), then rules transform.

 ;; Root Rule: Initializes synthesis context, processes the grove
 (element GROVE
   (let* ((candidate-pool (quote (1 2 3 4 5)))
          (solved-env     (synthesize-bindings "?x" candidate-pool (quote ()))))
     (make scroll
       font-family-name: "Monospace"
       font-size:        10pt
       (make element
         gi:         "SYNTHESIZED-GROVE"
         attributes: (list (list "STATUS" (if solved-env "VERIFIED" "FAILED")))
         ;; Pass solved environment into child transformations
         (process-children-with-env solved-env)))))

 ;; Expression Rule: Recursive traversal down operator branches
 (element EXPR
   (make element
     gi:         "EXPR"
     attributes: (list (list "ID" (attribute-string "id")))
     (process-children)))

 (element OP
   (make element gi: "OP"
     (make literal (data (current-node)))))

 (element LEFT
   (make element gi: "LEFT"
     (process-children)))

 (element RIGHT
   (make element gi: "RIGHT"
     (process-children)))

 (element INT
   (make element gi: "INT"
     (make literal (data (current-node)))))

 ;; Hole Rule: Replaces <HOLE> with synthesized <INT> node
 ;; If synthesis found a binding for this variable, emit SYNTHESIZED-INT.
 ;; If not, emit UNRESOLVED-HOLE (signals failed synthesis).
 (element HOLE
   (let* ((var        (attribute-string "var"))
          (solved-val (lookup var (current-synthesis-env))))
     (if solved-val
         (make element
           gi:         "SYNTHESIZED-INT"
           attributes: (list (list "RESOLVED-FROM" var))
           (make literal (number->string solved-val)))
         (make element
           gi:         "UNRESOLVED-HOLE"
           attributes: (list (list "VAR" var))))))

)
