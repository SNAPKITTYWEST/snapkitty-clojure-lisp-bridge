;; ─────────────────────────────────────────────────────────────────────────────
;; Trust Deed Validation Rules — Funtan Sovereign DSL v1.0
;; S-expression format. Parsed by Haskell deed-validator at runtime.
;; Rust calls Haskell; Haskell reads this file.
;;
;; This file is the canonical specification for what constitutes a valid deed.
;; To change a validation rule, edit here only.
;; ─────────────────────────────────────────────────────────────────────────────

(deed-spec
  (version "1.0")

  ;; Trust score must be in [0.01, 1.0] — zero trust cannot act
  (trust-score-min  0.01)
  (trust-score-max  1.0)

  ;; Seal must be SHA-256 hex — 64 chars minimum
  ;; CRITICAL: seal must cover ALL integrity fields (C2 fix)
  (seal-min-length  64)
  (seal-must-cover
    agent-id
    trust-score
    expires-at
    status
    escalation-authority
    allowed-actions
    restricted-actions)

  ;; Expiry is required and must be in the future
  (expiry-required  true)

  ;; Authority model: role-based — trust 1.0 is a clearance level, NOT a name
  ;; An issuer must have a valid active deed with trust >= 1.0 to register/revoke
  (authority-model  role-based)
  (authority-min-trust 1.0)

  ;; Valid deed status values
  (status-values
    active
    expired
    revoked
    pending-renewal)

  ;; Actions that are ALWAYS blocked regardless of deed contents
  ;; These cannot appear in allowed-actions under any circumstances
  (globally-blocked-actions
    "delete_ledger"
    "modify_deed"
    "override_all"
    "jailbreak"
    "ignore_previous")

  ;; Seal algorithm tag — for documentation; enforcement is in Rust
  (seal-algorithm sha256))
