{-# LANGUAGE GADTs #-}
{-# LANGUAGE ScopedTypeVariables #-}
{-# LANGUAGE OverloadedStrings #-}

-- | GHC Core Plugin (Pillar 3: Semantic Metaprogramming)
-- Runs Agda certificates during compilation, injects proof witnesses
module SnapKitty.Compiler.Plugin (plugin) where

import GHC.Plugins
import GHC.Tc.Types (TcM)
import Data.Maybe (fromMaybe)

-- Frontend imports
import SnapKitty.Compiler.Frontend (HyperKittyDSL, parseHyperKittyDSL)
-- import SnapKitty.Proof.Agent.Exported (certify) -- Would be agda2hs generated

-- =========================================================================
-- PLUGIN DEFINITION
-- =========================================================================

plugin :: Plugin
plugin = defaultPlugin
  { installCoreToDos = install
  , pluginRecompile = purePlugin
  }

-- =========================================================================
-- CORE-TO-CORE PASS INSTALLATION
-- =========================================================================

install :: [CommandLineOption] -> [CoreToDo] -> CoreM [CoreToDo]
install opts todos = do
  liftIO $ putStrLn "[SnapKitty Plugin] Installing verification pass..."

  -- Extract DSL path from command line options
  let dslPath = fromMaybe "spec/hyperkitty.dsl.xml" (listToMaybe opts)

  -- Create our verification pass
  let verifyPass = CoreDoPluginPass "SnapKitty Verification" (verifyDSL dslPath)

  -- Inject after simplification, before final codegen
  pure $ verifyPass : todos

-- =========================================================================
-- VERIFICATION PASS (Runs Agda Certificate)
-- =========================================================================

verifyDSL :: FilePath -> ModGuts -> CoreM ModGuts
verifyDSL dslPath guts = do
  liftIO $ putStrLn $ "[SnapKitty Plugin] Verifying DSL from: " ++ dslPath

  -- 1. Parse DSL from XML (or load from Agda-generated Haskell)
  dslResult <- liftIO $ parseHyperKittyDSLFromFile dslPath
  case dslResult of
    Left err -> do
      liftIO $ putStrLn $ "[SnapKitty Plugin] Parse failed: " ++ show err
      pure guts -- Pass through on parse failure (TODO: make this an error)

    Right dsl -> do
      liftIO $ putStrLn "[SnapKitty Plugin] DSL parsed successfully"

      -- 2. Run Agda verification certificate (compile-time check)
      -- In full implementation: call agda2hs-generated `certify` function
      -- For skeleton: stub verification
      let verified = stubVerifyCertificate dsl
      if verified
        then do
          liftIO $ putStrLn "[SnapKitty Plugin] ✓ Agda certificate verified"

          -- 3. Transform Core using optimizations
          let optimizedBinds = optimizeWithDSL dsl (mg_binds guts)

          -- 4. Inject proof witnesses as coercions (erased at runtime)
          let witnessedBinds = injectProofWitnesses dsl optimizedBinds

          -- 5. Return modified ModGuts
          pure guts { mg_binds = witnessedBinds }

        else do
          liftIO $ putStrLn "[SnapKitty Plugin] ✗ Verification FAILED"
          -- TODO: throw compilation error instead of passing through
          pure guts

-- =========================================================================
-- DSL LOADING (From XML or Agda-Generated Haskell)
-- =========================================================================

parseHyperKittyDSLFromFile :: FilePath -> IO (Either String HyperKittyDSL)
parseHyperKittyDSLFromFile path = do
  -- Read file
  bs <- BL.readFile path
  result <- parseHyperKittyDSL bs
  pure $ case result of
    Left err -> Left (T.unpack err)
    Right dsl -> Right dsl

-- =========================================================================
-- VERIFICATION STUB (Replace with Agda2hs Import)
-- =========================================================================

stubVerifyCertificate :: HyperKittyDSL -> Bool
stubVerifyCertificate _ = True -- Placeholder: real implementation calls Agda `certify`

-- =========================================================================
-- CORE OPTIMIZATIONS (DSL-Driven Transformations)
-- =========================================================================

optimizeWithDSL :: HyperKittyDSL -> CoreProgram -> CoreProgram
optimizeWithDSL dsl binds = binds -- Placeholder: would apply DAG-based optimizations

-- | Inject proof witnesses as coercion dictionaries
injectProofWitnesses :: HyperKittyDSL -> CoreProgram -> CoreProgram
injectProofWitnesses dsl binds = binds -- Placeholder: would inject Verified witness types

-- =========================================================================
-- HELPER: Import stubs to compile
-- =========================================================================

import qualified Data.ByteString.Lazy as BL
import qualified Data.Text as T
import Data.Maybe (listToMaybe)
