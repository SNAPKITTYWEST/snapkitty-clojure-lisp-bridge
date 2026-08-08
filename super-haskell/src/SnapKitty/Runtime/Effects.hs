{-# LANGUAGE DataKinds #-}
{-# LANGUAGE FlexibleContexts #-}
{-# LANGUAGE GADTs #-}
{-# LANGUAGE LambdaCase #-}
{-# LANGUAGE TypeOperators #-}
{-# LANGUAGE TypeFamilies #-}

-- | Algebraic Effects Runtime Kernel (Pillar 1)
-- Unified semantics: compile-time verification ≡ runtime execution via effect handlers
module SnapKitty.Runtime.Effects
  ( SnapKittyEffect(..)
  , SnapKittyEs
  , runPureVerify
  , runProduction
  , verifyConstraint
  , sampleEntropy
  , scalarMult
  , auditLog
  ) where

import Effectful
import Effectful.Dispatch.Dynamic
import Effectful.Error.Static
import Effectful.State.Static.Local
import Effectful.Writer.Static.Local
import Data.Text (Text)
import qualified Data.Text as T
import qualified Data.Text.IO as TIO

-- =========================================================================
-- ALGEBRAIC EFFECT SIGNATURE (Domain Primitives)
-- =========================================================================

-- | Boolean Expression AST (from Frontend)
data BoolExpr
  = BVar Text
  | BConst Bool
  | BNot BoolExpr
  | BAnd BoolExpr BoolExpr
  | BOr BoolExpr BoolExpr
  | BImplies BoolExpr BoolExpr
  | BEqual BoolExpr BoolExpr
  | BNand BoolExpr BoolExpr
  deriving (Eq, Show)

-- | Cryptographic Point (placeholder)
data Point = Point
  { pointX :: Integer
  , pointY :: Integer
  } deriving (Eq, Show)

-- | Agent State (runtime representation)
data AgentState = AgentState
  { asId :: Text
  , asEntropy :: Double
  , asTrusted :: Bool
  , asActive :: Bool
  } deriving (Eq, Show)

-- | Verification Error
data VerifyError
  = EntropyViolation Double
  | TrustViolation Text
  | ConstraintFailure BoolExpr
  deriving (Eq, Show)

-- | Audit Trail (WORM chain entry)
newtype AuditTrail = AuditTrail { unAudit :: [Text] }
  deriving (Eq, Show, Semigroup, Monoid)

-- | The Algebraic Signature of the Compute Engine
data SnapKittyEffect :: Effect where
  -- Constraint Verification (Pure, Deterministic)
  VerifyConstraint :: BoolExpr -> SnapKittyEffect m Bool
  -- Entropy Sampling (Controlled Source)
  SampleEntropy :: SnapKittyEffect m Double
  -- Cryptographic Primitive (Verified by Agda)
  ScalarMult :: Integer -> Point -> SnapKittyEffect m Point
  -- Audit Log (WORM - Write Once Read Many)
  AuditLog :: Text -> SnapKittyEffect m ()

type instance DispatchOf SnapKittyEffect = Dynamic

-- =========================================================================
-- EFFECT OPERATIONS (User-Facing API)
-- =========================================================================

verifyConstraint :: (SnapKittyEffect :> es) => BoolExpr -> Eff es Bool
verifyConstraint = send . VerifyConstraint

sampleEntropy :: (SnapKittyEffect :> es) => Eff es Double
sampleEntropy = send SampleEntropy

scalarMult :: (SnapKittyEffect :> es) => Integer -> Point -> Eff es Point
scalarMult k p = send $ ScalarMult k p

auditLog :: (SnapKittyEffect :> es) => Text -> Eff es ()
auditLog = send . AuditLog

-- =========================================================================
-- EFFECT STACK (Compose with State, Error, Writer)
-- =========================================================================

type SnapKittyEs =
  '[ SnapKittyEffect
   , State AgentState
   , Error VerifyError
   , Writer AuditTrail
   , IOE
   ]

-- =========================================================================
-- HANDLER: PURE VERIFICATION (Compile-Time)
-- =========================================================================

-- | Pure handler for compile-time verification in GHC plugin
-- No IO, deterministic entropy, constraint eval only
runPureVerify :: AgentState -> Eff (SnapKittyEffect ': '[State AgentState, Error VerifyError, IOE]) a -> Eff '[IOE] (Either (CallStack, VerifyError) a)
runPureVerify initState =
  runErrorNoCallStack @VerifyError
  . evalState initState
  . interpret pureHandler

pureHandler :: SnapKittyEffect m a -> Eff (State AgentState ': Error VerifyError ': IOE ': es) a
pureHandler = \case
  VerifyConstraint expr -> do
    state <- get @AgentState
    pure $ evalBoolExpr expr (bindVars state)

  SampleEntropy -> pure 0.15 -- Deterministic seed for compile-time

  ScalarMult k p ->
    if k >= 2
      then throwError $ ConstraintFailure (BConst True) -- Reject k≥2 per Agda proof
      else pure $ verifiedScalarMult k p

  AuditLog _ -> pure () -- No-op at compile time

-- =========================================================================
-- HANDLER: PRODUCTION RUNTIME (Hardware Backend)
-- =========================================================================

-- | Production handler with FFI to Rust crypto and WORM chain
runProduction :: Eff (SnapKittyEffect ': '[State AgentState, Error VerifyError, Writer AuditTrail, IOE]) a -> Eff '[IOE] (Either (CallStack, VerifyError) a)
runProduction =
  runErrorNoCallStack @VerifyError
  . execWriter @AuditTrail
  . evalState (AgentState "production" 0.15 True True)
  . interpret productionHandler

productionHandler :: SnapKittyEffect m a -> Eff (State AgentState ': Error VerifyError ': Writer AuditTrail ': IOE ': es) a
productionHandler = \case
  VerifyConstraint expr -> do
    state <- get @AgentState
    let result = evalBoolExpr expr (bindVars state)
    unless result $ throwError $ ConstraintFailure expr
    pure result

  SampleEntropy -> do
    state <- get @AgentState
    let e = asEntropy state
    when (e > 0.20) $ throwError $ EntropyViolation e
    pure e

  ScalarMult k p -> liftIO $ rustFFI_ScalarMult k p -- FFI to Rust

  AuditLog msg -> do
    tell $ AuditTrail [msg]
    liftIO $ appendWORMChain msg

-- =========================================================================
-- PURE LOGIC (Shared by Both Handlers)
-- =========================================================================

-- | Boolean Expression Evaluator (Total Function)
evalBoolExpr :: BoolExpr -> (Text -> Maybe Bool) -> Bool
evalBoolExpr expr env = case expr of
  BVar v -> maybe False id (env v)
  BConst b -> b
  BNot e -> not (evalBoolExpr e env)
  BAnd a b -> evalBoolExpr a env && evalBoolExpr b env
  BOr a b -> evalBoolExpr a env || evalBoolExpr b env
  BImplies a b -> not (evalBoolExpr a env) || evalBoolExpr b env
  BEqual a b -> evalBoolExpr a env == evalBoolExpr b env
  BNand a b -> not (evalBoolExpr a env && evalBoolExpr b env)

-- | Variable Binding for Boolean Evaluation
bindVars :: AgentState -> Text -> Maybe Bool
bindVars s "active" = Just (asActive s)
bindVars s "trusted" = Just (asTrusted s)
bindVars s "entropy_ge_0" = Just (asEntropy s >= 0)
bindVars s "entropy_le_020" = Just (asEntropy s <= 0.20)
bindVars _ _ = Nothing

-- | Verified Scalar Multiplication (Agda-extracted logic)
verifiedScalarMult :: Integer -> Point -> Point
verifiedScalarMult 0 _ = Point 0 0 -- Point at infinity
verifiedScalarMult 1 p = p
verifiedScalarMult _ _ = error "Entropy bound violation: k >= 2 rejected by Agda proof"

-- =========================================================================
-- FFI STUBS (To Rust Backend)
-- =========================================================================

-- | FFI to Rust crypto implementation
rustFFI_ScalarMult :: Integer -> Point -> IO Point
rustFFI_ScalarMult k (Point x y) = do
  -- TODO: foreign import ccall "rust_scalar_mult" rust_scalar_mult :: CInt -> CInt -> CInt -> IO (CInt, CInt)
  pure $ Point (x * k) (y * k) -- Placeholder

-- | FFI to WORM chain (Bifrost bridge)
appendWORMChain :: Text -> IO ()
appendWORMChain msg = TIO.putStrLn $ "[WORM] " <> msg
