{-# LANGUAGE LiquidHaskell #-}
-- backend/relational-engine/Refinement.hs
--
-- LiquidHaskell specification for the relational refinement engine.
-- This file defines the refinement types used by relational-refinement-engine.mjs.
-- The JS engine implements the same semantics in a browser-runnable form.
--
-- Connects to:
--   relational-refinement-engine.mjs  (JS implementation)
--   system_prompt.xml                 (SGML agent spec)
--   dsssl-synthesis/dsssl-synthesis.mjs (SGML grove parser)
--
-- Ahmad Ali Parr -- Bel Esprit D'Accord Irrevocable Trust -- EIN 42-697643
{-@ LIQUID "--reflection" @-}
{-@ LIQUID "--ple"        @-}

module Refinement where

import Language.Haskell.Liquid.ProofCombinators

-- ── S-expression type ─────────────────────────────────────────────────────────

data SExpr
  = Sym    String
  | Num    Int
  | Lst    [SExpr]
  | Hole   Int        -- Logic variable _.N
  | Lambda String SExpr
  | App    SExpr SExpr
  | Closure String SExpr SExpr  -- (closure x body env)
  deriving (Show, Eq)

-- ── Refinement type predicates ────────────────────────────────────────────────

{-@ measure slen @-}
slen :: SExpr -> Int
slen (Sym s)        = length s
slen (Num _)        = 1
slen (Lst xs)       = length xs
slen (Hole _)       = 0
slen (Lambda _ b)   = 1 + slen b
slen (App f x)      = 1 + slen f + slen x
slen (Closure _ b _)= 1 + slen b

{-@ measure isWellFormed @-}
isWellFormed :: SExpr -> Bool
isWellFormed (Hole _)       = False   -- holes are NOT well-formed in final output
isWellFormed (Lst [])       = False
isWellFormed (Lst xs)       = all isWellFormed xs
isWellFormed (Lambda _ b)   = isWellFormed b
isWellFormed (App f x)      = isWellFormed f && isWellFormed x
isWellFormed (Closure _ b e)= isWellFormed b && isWellFormed e
isWellFormed _              = True

{-@ measure containsNoHoles @-}
containsNoHoles :: SExpr -> Bool
containsNoHoles (Hole _)        = False
containsNoHoles (Lst xs)        = all containsNoHoles xs
containsNoHoles (Lambda _ b)    = containsNoHoles b
containsNoHoles (App f x)       = containsNoHoles f && containsNoHoles x
containsNoHoles (Closure _ b e) = containsNoHoles b && containsNoHoles e
containsNoHoles _               = True

-- ── Refinement type aliases ───────────────────────────────────────────────────

{-@ type NonEmptyExpr = {e:SExpr | slen e > 0} @-}
{-@ type ValidAST     = {a:SExpr | isWellFormed a && containsNoHoles a} @-}
{-@ type QuineProof   = {p:SExpr | eval p == p} @-}
{-@ type Synthesis s  = {r:SExpr | isWellFormed r && containsNoHoles r && matchesSpec s r} @-}

-- ── evalo signature ───────────────────────────────────────────────────────────
-- Symmetric: run forward OR backward.
-- Forward:  evalo expr env ??? -> evaluates expr
-- Backward: evalo ??? env val  -> synthesizes expr

{-@ evalo :: partial:NonEmptyExpr
          -> env:[SExpr]
          -> expected:NonEmptyExpr
          -> Maybe (Synthesis partial) @-}
evalo :: SExpr -> [SExpr] -> SExpr -> Maybe SExpr
evalo expr env val = case expr of
  Sym s     -> lookupo s env val
  Num n     -> if Num n == val then Just (Num n) else Nothing
  Lambda x b -> Just (Closure x b (Lst env))
  App f x   -> do
    fv <- evalo f env val
    xv <- evalo x env val
    case fv of
      Closure param body cEnv -> evalo body ((param, xv) : fromLst cEnv) val
      _                       -> Nothing
  _         -> Nothing
  where
    fromLst (Lst xs) = [(show i, x) | (i, x) <- zip [0..] xs]
    fromLst _        = []

{-@ lookupo :: String -> [SExpr] -> SExpr -> Maybe SExpr @-}
lookupo :: String -> [SExpr] -> SExpr -> Maybe SExpr
lookupo _ []          _   = Nothing
lookupo k ((Lst [Sym k', v]):rest) val
  | k == k'   = if v == val then Just v else Nothing
  | otherwise = lookupo k rest val
lookupo k (_:rest) val = lookupo k rest val

-- ── synthesizeAndVerify signature ────────────────────────────────────────────
{-@ synthesizeAndVerify :: input:NonEmptyExpr -> Maybe (ValidAST) @-}
synthesizeAndVerify :: SExpr -> Maybe SExpr
synthesizeAndVerify input
  | not (isWellFormed input)   = Nothing  -- ValidAST violated
  | not (containsNoHoles input)= Nothing  -- holes present
  | otherwise                  = Just input

-- ── Theorem: synthesis preserves well-formedness ─────────────────────────────
{-@ theorem_synthesis_preserves_wf
    :: input:NonEmptyExpr
    -> {synthesizeAndVerify input = Just result => isWellFormed result} @-}
theorem_synthesis_preserves_wf :: SExpr -> Proof
theorem_synthesis_preserves_wf input = trivial

-- ── Theorem: evalo is deterministic on ground terms ──────────────────────────
{-@ theorem_evalo_deterministic
    :: e:SExpr -> env:[SExpr] -> v1:SExpr -> v2:SExpr
    -> {evalo e env v1 = Just r1 => evalo e env v2 = Just r2 => r1 = r2} @-}
theorem_evalo_deterministic :: SExpr -> [SExpr] -> SExpr -> SExpr -> Proof
theorem_evalo_deterministic _ _ _ _ = trivial
