;; ============================================================================
;; FUNTAN SOVEREIGN DSL — Language Specification v1.0
;; ============================================================================
;;
;; Funtan is a Lisp dialect created by Ahmad Ali Parr for trust deed validation
;; in the SnapKitty sovereign stack.
;;
;; The name comes from the Arabic root for "art" / "creative act" (funun).
;; A Funtan program IS the specification. The Haskell validator enforces it.
;; The Rust runtime calls the validator. Prolog loads it as a layer.
;;
;; Design principles:
;;   1. The S-expression file is the SINGLE source of truth for validation rules.
;;      To change a rule, edit here only. Everything else derives.
;;   2. No mutation. Funtan specs are immutable once sealed.
;;   3. Human-readable. A non-programmer can read a Funtan deed spec.
;;   4. Formally enforceable. Haskell + LiquidHaskell types enforce at runtime.
;;
;; Syntax:
;;   (keyword value)               -- single field declaration
;;   (keyword value1 value2 ...)   -- multi-value field
;;   ;; comment                    -- line comment
;;
;; Built-in field types:
;;   number     -- floating point (0.01, 1.0)
;;   boolean    -- true | false
;;   string     -- "quoted-string"
;;   symbol     -- unquoted-identifier
;;   list       -- (item1 item2 ...)
;;
;; Top-level forms:
;;   (deed-spec ...)           -- defines a deed validation specification
;;   (version "N.M")           -- spec version (required)
;;   (trust-score-min N)       -- minimum valid trust score
;;   (trust-score-max N)       -- maximum valid trust score
;;   (seal-min-length N)       -- minimum seal hex length
;;   (seal-must-cover s1 s2 ..)-- fields the seal must cover
;;   (expiry-required bool)    -- whether expiry timestamp is required
;;   (authority-model symbol)  -- role-based | identity-based
;;   (authority-min-trust N)   -- min trust to issue/revoke deeds
;;   (status-values s1 s2 ...) -- valid deed status strings
;;   (globally-blocked-actions -- these actions are ALWAYS blocked
;;     "action1" "action2" ..)    regardless of deed contents
;;   (seal-algorithm symbol)   -- sha256 | blake3
;;
;; Example:
;;   (deed-spec
;;     (version "1.0")
;;     (trust-score-min 0.01)
;;     (globally-blocked-actions "delete_ledger" "jailbreak"))
;;
;; Runtime:
;;   Haskell: deed_validator.hs parses this file via parseSExprs
;;   Rust:    calls deed_validator as subprocess, reads key=value stdout
;;   Prolog:  shrew_observer.pl loads funtan_rules as a layer
;;
;; Prior art: Ahmad Ali Parr, SnapKitty Collective, 2026
;;            Bel Esprit D'Accord Irrevocable Trust, EIN 42-697643
;; ============================================================================

;; ── Grammar extension for future Funtan versions ─────────────────────────────
;;
;; v1.1 (planned): conditional rules
;;   (when (trust-score >= 0.9) (allow "high-value-transfer"))
;;
;; v1.2 (planned): rule composition
;;   (import "base-deed-spec.lisp")
;;   (extend (trust-score-min 0.05))
;;
;; v2.0 (planned): proof-carrying deeds
;;   (proof-obligation "deed-valid-implies-agent-sovereign")
;;   (lean4-theorem "DeedValidator.deed_valid")
;;
;; ── Connection to EmojiScript ─────────────────────────────────────────────────
;;
;; EmojiScript opcodes can reference Funtan-validated deed capabilities:
;;   🔑 (CapGate) -- pops slot + rights, checked against deed allowed-actions
;;   🧠 (PolicyCheck) -- routes to Funtan rule engine for policy evaluation
;;   🔒 (Seal) -- triggers Bifrost seal, deed seal-algorithm applied
;;
;; The pipeline:
;;   EmojiScript bytecode
;;         -> 🧠 PolicyCheck
;;         -> Funtan deed-rules.lisp evaluation
;;         -> Haskell deed_validator enforcement
;;         -> Rust runtime authorization
;;         -> WORM seal
