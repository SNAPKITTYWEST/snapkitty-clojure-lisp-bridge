{-# LANGUAGE OverloadedStrings #-}
{-# LANGUAGE DeriveGeneric #-}

-- SKC-LISP: Layered Truth Maintenance System (LTMS) — Haskell Implementation
-- Knowledge layer: Type-safe symbolic reasoning
-- Author: Ahmad Parr (ahmedparr93@gmail.com)
-- Integration: Snapkitty LISP Bridge knowledge engine

module Snapkitty.LTMS.LTMS where

import Data.Time
import Data.List (sortBy, sortOn)
import Data.Ord (Down(..))
import Data.Map (Map)
import qualified Data.Map as Map
import Data.Set (Set)
import qualified Data.Set as Set
import Control.Monad (when)
import GHC.Generics
import Text.Printf

-- ============================================================================
-- DATA STRUCTURES
-- ============================================================================

-- A Fact: asserts a value from a source with confidence + priority
data Fact = Fact
  { value :: String
  , source :: String
  , timestamp :: UTCTime
  , confidence :: Double       -- 0.0 to 1.0
  , priority :: Int            -- higher = wins conflicts
  } deriving (Show, Eq, Generic)

-- A Sense: one interpretation of a concept
data Sense = Sense
  { gloss :: String                           -- human-readable meaning
  , contextPred :: Maybe (Context -> Bool)    -- predicate: "applies in this context?"
  , confidence :: Double                      -- how confident we are in this sense
  } deriving (Show, Eq)

-- A Concept: word/term with multiple possible meanings
data Concept = Concept
  { name :: String
  , senses :: [Sense]
  } deriving (Show, Eq)

-- Context: a key-value map for disambiguating senses
type Context = Map.Map String String

-- A Rule: head :- body within a module
data Rule = Rule
  { ruleHead :: String
  , ruleBody :: String
  , ruleModule :: String
  , rulePriority :: Int
  } deriving (Show, Eq)

-- Module statistics: for tracking complexity
data ModuleStats = ModuleStats
  { module :: String
  , ruleCount :: Int
  , maxRules :: Int        -- hard limit (80)
  , lastUpdated :: UTCTime
  } deriving (Show, Eq)

-- Result of hybrid query: symbolic proof vs embedding fallback
data HybridResult a
  = Symbolic a Double      -- proof + confidence (1.0 for logical proof)
  | Embedding a Double     -- embedding retrieval + confidence
  | Failed
  deriving (Show, Eq)

-- ============================================================================
-- 1. CONFLICT RESOLUTION
-- Higher priority wins, then higher confidence
-- ============================================================================

resolveConflict :: [Fact] -> Maybe Fact
resolveConflict [] = Nothing
resolveConflict fs = Just $ head $ sortBy cmp fs
  where
    cmp a b = compare (Down (priority a), Down (confidence a))
                      (Down (priority b), Down (confidence b))

-- Query the winning fact for a value
getBelief :: [Fact] -> String -> Maybe (Fact, Double)
getBelief facts val =
  case resolveConflict (filter (\f -> value f == val) facts) of
    Nothing -> Nothing
    Just f -> Just (f, confidence f)

-- ============================================================================
-- 2. OUTDATED DETECTION
-- Exponential decay: Conf(t) = Conf(0) * exp(-0.0001 * age_in_seconds)
-- ============================================================================

-- Half-life: time for confidence to drop to 50%
halfLifeSeconds :: Double
halfLifeSeconds = log 2.0 / 0.0001  -- ~6931 seconds

-- Compute decayed confidence at a given time
decayedConfidence :: Fact -> UTCTime -> Double
decayedConfidence f now =
  let age = realToFrac $ diffUTCTime now (timestamp f) :: Double
      decayed = confidence f * exp (-0.0001 * age)
  in decayed

-- Check if a fact is outdated (confidence < 15%)
isOutdated :: Fact -> UTCTime -> Bool
isOutdated f now = decayedConfidence f now < 0.15

-- Prune outdated facts
pruneOutdated :: [Fact] -> UTCTime -> [Fact]
pruneOutdated facts now = filter (not . flip isOutdated now) facts

-- ============================================================================
-- 3. AMBIGUOUS CONCEPTS
-- ============================================================================

-- Register a concept with multiple senses
createConcept :: String -> [String] -> Concept
createConcept nm glosses =
  Concept nm [Sense gloss Nothing 1.0 | gloss <- glosses]

-- Find senses that apply in a context
disambiguate :: Concept -> Context -> [Sense]
disambiguate c ctx =
  filter (\s -> case contextPred s of
                  Nothing -> True
                  Just pred -> pred ctx)
         (senses c)

-- Get the best (highest confidence) sense in context
bestSense :: Concept -> Context -> Maybe Sense
bestSense c ctx =
  case sortOn (Down . confidence) (disambiguate c ctx) of
    [] -> Nothing
    (s:_) -> Just s

-- ============================================================================
-- 4. MAINTAINABILITY GUARD
-- Hard limit: 80 rules per module (prevents knowledge explosion)
-- ============================================================================

-- Add a rule to a module (fails if limit exceeded)
addRule :: ModuleStats -> Rule -> Either String ModuleStats
addRule stats rule =
  if ruleCount stats >= maxRules stats
    then Left $ printf "Module %s has reached rule limit (%d rules)"
                       (LTMS.module stats)
                       (ruleCount stats)
    else Right $ stats { ruleCount = ruleCount stats + 1 }

-- Suggest refactoring when module > 70% full
needsRefactor :: ModuleStats -> Bool
needsRefactor stats =
  let percent = fromIntegral (ruleCount stats) / fromIntegral (maxRules stats) :: Double
  in percent > 0.7

-- Module complexity percentage (0-100)
complexityPercent :: ModuleStats -> Double
complexityPercent stats =
  (fromIntegral (ruleCount stats) / fromIntegral (maxRules stats)) * 100.0

-- ============================================================================
-- 5. HYBRID / NON-RULE KNOWLEDGE
-- Symbolic proof → Embedding fallback → Failure
-- ============================================================================

-- Pure symbolic prover (stub: just checks if fact exists)
proveSymbolic :: String -> [Fact] -> Maybe Fact
proveSymbolic goal facts =
  case filter (\f -> value f == goal) facts of
    [] -> Nothing
    (f:_) -> Just f

-- Embedding retrieval (stub: returns Nothing, can be extended to query vector DB)
embeddingRetrieve :: String -> Maybe (String, Double)
embeddingRetrieve _goal = Nothing  -- placeholder

-- Hybrid query: try symbolic first, then embedding
hybridQuery :: String -> [Fact] -> HybridResult Fact
hybridQuery goal facts =
  case proveSymbolic goal facts of
    Just f -> Symbolic f 1.0
    Nothing -> case embeddingRetrieve goal of
      Just (_res, conf) -> Embedding (Fact goal "embedding" (unsafePerformIO getCurrentTime) conf 0) conf
      Nothing -> Failed

-- ============================================================================
-- QUERY & ASSERTION API
-- ============================================================================

-- Get all facts for a value (before conflict resolution)
getAllFacts :: [Fact] -> String -> [Fact]
getAllFacts facts val = filter (\f -> value f == val) facts

-- Get facts from a specific source
factsBySource :: [Fact] -> String -> [Fact]
factsBySource facts src = filter (\f -> source f == src) facts

-- ============================================================================
-- SUBSUMPTION & TAXONOMIES
-- ============================================================================

-- Represent is-a relationships as facts
data Subsumption = Subsumption
  { subSpecific :: String
  , subGeneral :: String
  , subConfidence :: Double
  } deriving (Show, Eq)

-- Get all ancestors of a concept (transitive closure of is-a)
ancestors :: [Subsumption] -> String -> Set.Set String
ancestors subs concept = go concept Set.empty
  where
    go current visited
      | Set.member current visited = visited
      | otherwise =
          let visited' = Set.insert current visited
              parents = [subGeneral s | s <- subs, subSpecific s == current]
          in Set.unions $ visited' : [go p visited' | p <- parents]

-- ============================================================================
-- INSPECTION & DEBUGGING
-- ============================================================================

-- Pretty print a fact
factSummary :: Fact -> String
factSummary f =
  printf "Fact: %s\n  Source: %s\n  Confidence: %d%%\n  Priority: %d"
         (value f)
         (source f)
         (round $ confidence f * 100 :: Int)
         (priority f)

-- Pretty print a concept
conceptSummary :: Concept -> String
conceptSummary c =
  printf "Concept: %s\n  Senses: %d"
         (name c)
         (length $ senses c)

-- Module report
moduleReport :: [ModuleStats] -> String
moduleReport stats =
  unlines $
    [ printf "%s: %d/%d rules (%.1f%%)"
             (LTMS.module s) (ruleCount s) (maxRules s) (complexityPercent s)
    | s <- stats
    ] ++
    [ "" ] ++
    [ printf "Needs refactor (>70%%): %s"
             (unwords [LTMS.module s | s <- stats, needsRefactor s])
    ]

-- ============================================================================
-- TESTS
-- ============================================================================

runLTMSTests :: IO ()
runLTMSTests = do
  putStrLn "=== LTMS Tests (Haskell) ===\n"

  now <- getCurrentTime

  -- Test 1: Conflict resolution
  putStrLn "Test 1: Conflict resolution"
  let fact1 = Fact "color-car" "source-a" now 0.8 10
      fact2 = Fact "color-car" "source-b" now 0.9 5
      facts = [fact1, fact2]
  case resolveConflict facts of
    Just winner -> putStrLn $ "✓ Winner: " ++ source winner ++ " (confidence " ++ show (confidence winner) ++ ")"
    Nothing -> putStrLn "✗ No winner found"
  putStrLn ""

  -- Test 2: Outdated detection
  putStrLn "Test 2: Outdated detection"
  let oldTime = addUTCTime (-10 * 60) now  -- 10 minutes ago
      oldFact = Fact "old-value" "old-source" oldTime 0.5 1
  if isOutdated oldFact now
    then putStrLn "✓ Old fact marked as outdated"
    else putStrLn "✗ Old fact not detected"
  putStrLn ""

  -- Test 3: Concepts & disambiguation
  putStrLn "Test 3: Concepts & disambiguation"
  let bankConcept = createConcept "bank" ["financial institution", "river bank", "pile of snow"]
  putStrLn $ "✓ Bank concept has " ++ show (length $ senses bankConcept) ++ " senses"
  putStrLn ""

  -- Test 4: Module complexity
  putStrLn "Test 4: Module complexity guard"
  let stats = ModuleStats "main" 1 80 now
  putStrLn $ printf "✓ Main module: %.1f%% full" (complexityPercent stats)
  putStrLn ""

  -- Test 5: Hybrid query
  putStrLn "Test 5: Hybrid query with fallback"
  let skyFact = Fact "sky-color" "observation" now 0.95 100
      facts' = [skyFact]
  case hybridQuery "sky-color" facts' of
    Symbolic f _ -> putStrLn $ "✓ Query result: " ++ value f ++ " (symbolic)"
    Embedding f _ -> putStrLn $ "✓ Query result: " ++ value f ++ " (embedding)"
    Failed -> putStrLn "✗ Query failed"
  putStrLn ""

  putStrLn "=== LTMS Tests Complete ===\n"

-- Helper to get current time for tests
unsafePerformIO :: IO a -> a
unsafePerformIO = System.IO.Unsafe.unsafePerformIO
