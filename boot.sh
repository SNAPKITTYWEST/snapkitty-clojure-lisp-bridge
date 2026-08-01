#!/usr/bin/env bash
# SnapKitty Sovereign Lisp Stack — Cold Boot
# One command. Every layer. Full sovereign Lisp runtime.
#
# curl -fsSL https://raw.githubusercontent.com/SNAPKITTYWEST/snapkitty-clojure-lisp-bridge/main/boot.sh | bash
#
# Or clone and run:
#   git clone https://github.com/SNAPKITTYWEST/snapkitty-clojure-lisp-bridge
#   cd snapkitty-clojure-lisp-bridge && bash boot.sh

set -euo pipefail

GRN='\033[0;32m'; YLW='\033[0;33m'; CYN='\033[0;36m'
MAG='\033[0;35m'; RED='\033[0;31m'; DIM='\033[2m'
BLD='\033[1m'; NC='\033[0m'

clear

echo -e "${MAG}"
cat << 'BANNER'
  ██████  ███    ██  █████  ██████  ██   ██ ██ ████████ ████████ ██    ██
  ██      ████   ██ ██   ██ ██   ██ ██  ██  ██    ██       ██     ██  ██
  ███████ ██ ██  ██ ███████ ██████  █████   ██    ██       ██      ████
       ██ ██  ██ ██ ██   ██ ██      ██  ██  ██    ██       ██       ██
  ███████ ██   ████ ██   ██ ██      ██   ██ ██    ██       ██       ██

  ██████  ██████  ██    ██ ███████ ██████  ███████ ██  ██████  ███    ██
  ██      ██    ██ ██    ██ ██      ██   ██ ██      ██ ██       ████   ██
  ███████ ██    ██ ██    ██ █████   ██████  █████   ██ ██   ███ ██ ██  ██
       ██ ██    ██  ██  ██  ██      ██   ██ ██      ██ ██    ██ ██  ██ ██
  ███████ ██████    ████   ███████ ██   ██ ███████ ██  ██████  ██   ████

  ██      ██ ███████ ██████      ███████ ████████  █████   ██████ ██   ██
  ██      ██ ██      ██   ██     ██         ██     ██   ██ ██      ██  ██
  ██      ██ ███████ ██████      ███████    ██     ███████ ██      █████
  ██      ██      ██ ██               ██    ██     ██   ██ ██      ██  ██
  ███████ ██ ███████ ██          ███████    ██     ██   ██  ██████ ██   ██
BANNER
echo -e "${NC}"
echo -e "${CYN}  McCarthy LISP · Rust REPL · miniKanren · EmojiScript · Funtan DSL${NC}"
echo -e "${CYN}  DSSSL · Coq · ClojureScript · WORM · Bifrost · Sovereign${NC}"
echo ""
echo -e "${DIM}  Bel Esprit D'Accord Irrevocable Trust · EIN 42-697643${NC}"
echo -e "${DIM}  Ahmad Ali Parr · BOW-Ω-φ-∂-2026${NC}"
echo ""

OS="unknown"
[[ "$OSTYPE" == "darwin"* ]] && OS="mac"
[[ "$OSTYPE" == "linux-gnu"* ]] && OS="linux"
[[ "$OSTYPE" == msys* || "$OSTYPE" == cygwin* ]] && OS="windows"
echo -e "${DIM}  Platform: $OS${NC}"
echo ""

check() { command -v "$1" &>/dev/null; }
ok()    { echo -e "  ${GRN}✓${NC} $1"; }
warn()  { echo -e "  ${YLW}?${NC} $1 ${DIM}(optional)${NC}"; }
miss()  { echo -e "  ${RED}✗${NC} $1"; }

# ── [1/6] Node.js ────────────────────────────────────────────────────────────
echo -e "${YLW}  [1/6] Node.js${NC}"
if check node; then
    ok "Node.js $(node --version)"
    npm install --silent 2>/dev/null && ok "npm packages installed"
else
    miss "Node.js not found"
    [[ "$OS" == "mac" ]]   && echo -e "  ${DIM}  brew install node${NC}"
    [[ "$OS" == "linux" ]] && echo -e "  ${DIM}  sudo apt install nodejs npm${NC}"
fi

# ── [2/6] Rust ────────────────────────────────────────────────────────────────
echo -e "${YLW}  [2/6] Rust (Lisp REPL + Bifrost)${NC}"
if check cargo; then
    ok "Rust $(rustc --version | cut -d' ' -f2)"
    echo -e "  ${DIM}  Building lisp-rs...${NC}"
    cargo build --manifest-path backend/lisp-rs/Cargo.toml --quiet 2>/dev/null \
        && ok "lisp-repl built" || warn "lisp-rs build failed (check Cargo.toml deps)"
else
    miss "Rust not found"
    echo -e "  ${DIM}  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
fi

# ── [3/6] SWI-Prolog ─────────────────────────────────────────────────────────
echo -e "${YLW}  [3/6] SWI-Prolog (LTMS + Funtan + Governance)${NC}"
if check swipl; then
    ok "SWI-Prolog $(swipl --version 2>&1 | head -1 | grep -oP '\d+\.\d+\.\d+')"
else
    warn "swipl not found"
    [[ "$OS" == "mac" ]]   && echo -e "  ${DIM}  brew install swi-prolog${NC}"
    [[ "$OS" == "linux" ]] && echo -e "  ${DIM}  sudo apt install swi-prolog${NC}"
fi

# ── [4/6] Lean 4 ─────────────────────────────────────────────────────────────
echo -e "${YLW}  [4/6] Lean 4 (QEC proofs)${NC}"
if check lean || check lake; then
    ok "Lean 4 found"
    echo -e "  ${DIM}  Run: cd lean-formalization/skclisp && lake build${NC}"
else
    warn "Lean 4 not found"
    echo -e "  ${DIM}  curl https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh -sSf | sh${NC}"
fi

# ── [5/6] Coq ────────────────────────────────────────────────────────────────
echo -e "${YLW}  [5/6] Coq (SKC-LISP-WORLD proofs)${NC}"
if check coqc; then
    ok "Coq $(coqc --version | head -1 | grep -oP '\d+\.\d+\.\d+')"
    echo -e "  ${DIM}  Run: cd coq && coq_makefile -f _CoqProject -o Makefile && make${NC}"
else
    warn "Coq not found"
    echo -e "  ${DIM}  opam install coq coq-mathcomp-ssreflect${NC}"
fi

# ── [6/6] GHC ────────────────────────────────────────────────────────────────
echo -e "${YLW}  [6/6] GHC (Funtan validator + deed_validator.hs)${NC}"
if check ghc; then
    ok "GHC $(ghc --version | grep -oP '\d+\.\d+\.\d+')"
else
    warn "GHC not found"
    echo -e "  ${DIM}  ghcup install ghc (https://www.haskell.org/ghcup/)${NC}"
fi

echo ""
echo -e "${GRN}  ═══════════════════════════════════════════════════════════${NC}"
echo -e "${GRN}  SOVEREIGN LISP STACK — BOOT COMPLETE${NC}"
echo -e "${GRN}  ═══════════════════════════════════════════════════════════${NC}"
echo ""

# ── What you can run right now ───────────────────────────────────────────────
echo -e "${BLD}  WHAT YOU CAN RUN RIGHT NOW:${NC}"
echo ""

if check node; then
cat << 'CMDS'
  ── ClojureScript Lisp Machine ──────────────────────────────────────
  npm run watch              open http://localhost:9000 (dev, hot reload)
  npm run build              production build
  npm run compile:clojure    Lisp -> EmojiScript bytecode
  npm test                   30 tests (Lisp, EmojiScript, JIT, LTMS)

  ── Relational Engine ────────────────────────────────────────────────
  npm run pipeline           miniKanren tag validation pipeline
  node backend/relational-engine/examples/tree-invert.mjs
                             tree inversion synthesis (2-pass Z3 oracle)

  ── DSSSL Synthesis ──────────────────────────────────────────────────
  node dsssl-synthesis/dsssl-kernel.mjs dsssl-synthesis/dsssl-input.sgml
                             (?x + 4) * 4 = 20  =>  ?x = 1
  node dsssl-synthesis/test-dsssl.mjs

  ── Funtan DSL ───────────────────────────────────────────────────────
  node funtan/deed_validator_bridge.mjs
                             parse deed-rules.lisp + validate trust deed

  ── Quantum QEC Pipeline ─────────────────────────────────────────────
  node bob-reasoning-engine/wire-quantum.mjs
                             Lean -> APL -> Rust -> WORM (7 stages)

  ── Servers ──────────────────────────────────────────────────────────
  npm run serve:bob          BOB sovereign agent (EVIDENCE / SILENCE)
  npm run serve:snap-os      SNAP OS JIT bridge (port 8001)
CMDS
fi

if check cargo; then
cat << 'CMDS'
  ── Rust Lisp REPL ───────────────────────────────────────────────────
  cargo run --bin lisp-repl --manifest-path backend/lisp-rs/Cargo.toml
  # lambda> (+ 1618 618)   =>  2236
  # lambda> (seal!)         =>  WORM checkpoint
CMDS
fi

if check swipl; then
cat << 'CMDS'
  ── Prolog ───────────────────────────────────────────────────────────
  swipl -g run_tests -t halt src/snapkitty/ltms/ltms.pl
  swipl -g run_tests -t halt logic/bio_ops.pl
CMDS
fi

echo ""
echo -e "${CYN}  Live at: https://snapkittywest.github.io/snapkitty-clojure-lisp-bridge/${NC}"
echo ""
echo -e "${MAG}  φ = 1.6180339887...   Ω = TRUST ∧ CODE${NC}"
echo ""
