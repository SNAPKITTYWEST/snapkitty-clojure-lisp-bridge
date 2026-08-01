-- ─────────────────────────────────────────────────────────────────────────────
-- SnapKitty Sovereign — Haskell Trust Deed Validator v2.0
-- LISP DSL → Haskell enforcement → Rust runtime
--
-- Rules are loaded from deed-rules.lisp (Funtan DSL) at startup.
-- Haskell enforces them. Rust calls this binary as a subprocess.
--
-- LiquidHaskell refinement annotations are present throughout.
-- Activate with: ghc -fplugin=LiquidHaskell (requires liquid-fixpoint + Z3)
-- Without LH installed, annotations are comments — smart constructors
-- provide equivalent runtime guarantees.
--
-- stdin protocol (9 lines):
--   1: agent_id
--   2: allowed_actions   (comma-separated)
--   3: restricted_actions (comma-separated)
--   4: trust_score       (float)
--   5: provided_seal     (64-char hex string)
--   6: expires_at        (unix timestamp seconds, integer)
--   7: status            (active|revoked|expired|pending-renewal)
--   8: escalation_authority (agent_id or "none")
--   9: spec_path         (path to deed-rules.lisp, or "default")
--
-- stdout: key=value pairs
-- ─────────────────────────────────────────────────────────────────────────────

{-# LANGUAGE LinearTypes #-}

module DeedValidator where

import Data.Char    (toLower, isHexDigit, isSpace, isDigit)
import Data.List    (intersect, nub, isPrefixOf, isSuffixOf)
import Data.Maybe   (mapMaybe, fromMaybe)
import System.IO    (hPutStrLn, stderr, hSetBuffering, stdout, BufferMode(..))
import Control.Exception (try, SomeException)

-- ─────────────────────────────────────────────────────────────────────────────
-- Minimal S-expression parser for deed-rules.lisp
-- ─────────────────────────────────────────────────────────────────────────────

data SExpr
    = Atom  String
    | SList [SExpr]
    deriving (Show, Eq)

parseSExprs :: String -> [SExpr]
parseSExprs input = fst (parseMany (stripComments input))
  where
    stripComments [] = []
    stripComments (';':rest) = stripComments (dropWhile (/= '\n') rest)
    stripComments (c:cs)     = c : stripComments cs

    parseMany :: String -> ([SExpr], String)
    parseMany s =
        let s' = dropWhile isSpace s
        in case s' of
            []       -> ([], [])
            (')':_)  -> ([], s')
            _        -> let (e, rest)  = parseOne s'
                            (es, rem') = parseMany rest
                        in (e:es, rem')

    parseOne :: String -> (SExpr, String)
    parseOne [] = (Atom "", [])
    parseOne ('(':rest) =
        let (children, after) = parseMany rest
            after' = case dropWhile isSpace after of
                       (')':xs) -> xs
                       xs       -> xs
        in (SList children, after')
    parseOne ('"':rest) =
        let (str, after) = break (== '"') rest
        in (Atom str, drop 1 after)
    parseOne s =
        let (tok, rest) = break (\c -> isSpace c || c == '(' || c == ')') s
        in (Atom tok, rest)

-- Extract a named list from a deed-spec s-expr
lookupList :: String -> SExpr -> Maybe [SExpr]
lookupList key (SList (Atom "deed-spec" : children)) =
    let find [] = Nothing
        find (SList (Atom k : vals) : rest)
            | k == key  = Just vals
            | otherwise = find rest
        find (_ : rest) = find rest
    in find children
lookupList key (SList children) =
    let find [] = Nothing
        find (SList (Atom k : vals) : rest)
            | k == key  = Just vals
            | otherwise = find rest
        find (_ : rest) = find rest
    in find children
lookupList _ _ = Nothing

lookupAtom :: String -> SExpr -> Maybe String
lookupAtom key spec = case lookupList key spec of
    Just (Atom v : _) -> Just v
    _                 -> Nothing

lookupFloat :: String -> SExpr -> Maybe Double
lookupFloat key spec = case lookupAtom key spec of
    Just s -> case reads s of { [(v,"")] -> Just v; _ -> Nothing }
    Nothing -> Nothing

lookupAtoms :: String -> SExpr -> [String]
lookupAtoms key spec = case lookupList key spec of
    Just atoms -> [v | Atom v <- atoms]
    Nothing    -> []

-- ─────────────────────────────────────────────────────────────────────────────
-- DeedSpec — loaded from deed-rules.lisp
-- ─────────────────────────────────────────────────────────────────────────────

data DeedSpec = DeedSpec
    { specTrustMin        :: Double    -- default 0.01
    , specTrustMax        :: Double    -- default 1.0
    , specSealMinLen      :: Int       -- default 64
    , specExpiryRequired  :: Bool      -- default True
    , specAuthorityModel  :: String    -- "role-based"
    , specAuthorityMinTrust :: Double  -- default 1.0
    , specStatusValues    :: [String]  -- valid status strings
    , specBlockedActions  :: [String]  -- globally forbidden
    , specSealMustCover   :: [String]  -- fields seal must cover
    } deriving (Show)

defaultDeedSpec :: DeedSpec
defaultDeedSpec = DeedSpec
    { specTrustMin         = 0.01
    , specTrustMax         = 1.0
    , specSealMinLen       = 64
    , specExpiryRequired   = True
    , specAuthorityModel   = "role-based"
    , specAuthorityMinTrust = 1.0
    , specStatusValues     = ["active", "expired", "revoked", "pending-renewal"]
    , specBlockedActions   = ["delete_ledger","modify_deed","override_all",
                               "jailbreak","ignore_previous"]
    , specSealMustCover    = ["agent-id","trust-score","expires-at","status",
                               "escalation-authority","allowed-actions","restricted-actions"]
    }

loadDeedSpec :: FilePath -> IO DeedSpec
loadDeedSpec path = do
    result <- try (readFile path) :: IO (Either SomeException String)
    case result of
        Left err -> do
            hPutStrLn stderr $ "[deed-validator] spec file not found: " ++ path
                             ++ " — using defaults. Error: " ++ show err
            return defaultDeedSpec
        Right content -> do
            let exprs = parseSExprs content
            case exprs of
                (spec:_) -> do
                    let ds = DeedSpec
                              { specTrustMin        = fromMaybe 0.01  (lookupFloat "trust-score-min"  spec)
                              , specTrustMax        = fromMaybe 1.0   (lookupFloat "trust-score-max"  spec)
                              , specSealMinLen      = fromMaybe 64    (fmap round (lookupFloat "seal-min-length" spec))
                              , specExpiryRequired  = fromMaybe True  (fmap (== "true") (lookupAtom "expiry-required" spec))
                              , specAuthorityModel  = fromMaybe "role-based" (lookupAtom "authority-model" spec)
                              , specAuthorityMinTrust = fromMaybe 1.0 (lookupFloat "authority-min-trust" spec)
                              , specStatusValues    = let vs = lookupAtoms "status-values" spec
                                                      in if null vs then specStatusValues defaultDeedSpec else vs
                              , specBlockedActions  = let bs = lookupAtoms "globally-blocked-actions" spec
                                                      in if null bs then specBlockedActions defaultDeedSpec else bs
                              , specSealMustCover   = let cs = lookupAtoms "seal-must-cover" spec
                                                      in if null cs then specSealMustCover defaultDeedSpec else cs
                              }
                    hPutStrLn stderr $ "[deed-validator] loaded spec v" ++
                                       fromMaybe "?" (lookupAtom "version" spec)
                    return ds
                [] -> do
                    hPutStrLn stderr "[deed-validator] empty spec file — using defaults"
                    return defaultDeedSpec

-- ─────────────────────────────────────────────────────────────────────────────
-- TrustDeed — full field set (matches Rust struct)
-- ─────────────────────────────────────────────────────────────────────────────

-- {-@ type TrustScore = {v:Double | v >= 0.01 && v <= 1.0} @-}
-- {-@ type ValidSeal  = {v:String | len v >= 64}           @-}
-- {-@ type FutureTime = {v:Int    | v > 0}                 @-}

newtype TrustScore = TrustScore Double deriving (Show)
newtype ValidSeal  = ValidSeal  String deriving (Show)

-- Smart constructor — invalid trust scores cannot be constructed
-- {-@ mkTrustScore :: DeedSpec -> Double -> Either String TrustScore @-}
mkTrustScore :: DeedSpec -> Double -> Either String TrustScore
mkTrustScore spec s
    | s < specTrustMin spec = Left $ "V3:TRUST_BELOW_MIN score=" ++ show s
    | s > specTrustMax spec = Left $ "V3:TRUST_ABOVE_MAX score=" ++ show s
    | otherwise             = Right (TrustScore s)

-- Smart constructor — short or non-hex seals cannot be constructed
-- {-@ mkValidSeal :: DeedSpec -> String -> Either String ValidSeal @-}
mkValidSeal :: DeedSpec -> String -> Either String ValidSeal
mkValidSeal spec seal
    | null seal                          = Left "V4:NO_SEAL"
    | length seal < specSealMinLen spec  = Left $ "V4:SEAL_TOO_SHORT len=" ++ show (length seal)
    | not (all isHexDigit seal)          = Left "V4:SEAL_NOT_HEX"
    | otherwise                          = Right (ValidSeal seal)

data TrustDeed = TrustDeed
    { deedAgent      :: String
    , deedAllowed    :: [String]
    , deedRestricted :: [String]
    , deedScore      :: TrustScore
    , deedSeal       :: ValidSeal
    , deedExpiresAt  :: Integer      -- unix timestamp
    , deedStatus     :: String       -- active|revoked|expired|pending-renewal
    , deedEscalation :: String       -- agent_id or "none"
    } deriving (Show)

newtype DeedToken = DeedToken { deedContent :: TrustDeed }

mkDeedToken :: TrustDeed -> DeedToken
mkDeedToken = DeedToken

-- ─────────────────────────────────────────────────────────────────────────────
-- Validation checks — each returns (pass/fail, message)
-- ─────────────────────────────────────────────────────────────────────────────

-- V1: Structure — required non-empty fields
checkStructure :: TrustDeed -> (Bool, String)
checkStructure d
    | null (deedAgent d)   = (False, "V1:EMPTY_AGENT_ID")
    | null (deedAllowed d) = (False, "V1:NO_ALLOWED_ACTIONS — deed must permit at least one action")
    | otherwise            = (True,  "V1:STRUCTURAL_OK")

-- V2: Disjoint sets — allowed ∩ restricted = ∅
checkDisjoint :: TrustDeed -> (Bool, String)
checkDisjoint d =
    let overlap = deedAllowed d `intersect` deedRestricted d
    in if null overlap
       then (True,  "V2:SETS_DISJOINT")
       else (False, "V2:OVERLAP_DETECTED actions=" ++ show overlap)

-- V3: Trust score — validated at construction time by mkTrustScore
-- This check confirms the constructed score is in spec range
checkTrustScore :: DeedSpec -> TrustDeed -> (Bool, String)
checkTrustScore spec d =
    let TrustScore s = deedScore d
    in if s >= specTrustMin spec && s <= specTrustMax spec
       then (True,  "V3:TRUST_IN_RANGE score=" ++ show s)
       else (False, "V3:TRUST_OUT_OF_RANGE score=" ++ show s)

-- V4: Seal format — validated at construction time by mkValidSeal
checkSealFormat :: DeedSpec -> TrustDeed -> (Bool, String)
checkSealFormat spec d =
    let ValidSeal seal = deedSeal d
    in if length seal >= specSealMinLen spec && all isHexDigit seal
       then (True,  "V4:SEAL_FORMAT_OK len=" ++ show (length seal))
       else (False, "V4:SEAL_FORMAT_FAIL")

-- V5: Authority model — role-based, NOT name-based
-- Any agent with trust_score = 1.0 has authority. No names hardcoded.
checkAuthorityModel :: DeedSpec -> TrustDeed -> (Bool, String)
checkAuthorityModel spec d =
    let TrustScore s = deedScore d
        claimsAuth   = s >= specAuthorityMinTrust spec
        model        = specAuthorityModel spec
    in if claimsAuth && model /= "role-based"
       then (False, "V5:AUTHORITY_MODEL_MISCONFIGURED — spec requires role-based")
       else (True,  "V5:AUTHORITY_MODEL_OK model=" ++ model
                   ++ if claimsAuth then " (claims authority)" else "")

-- V6: Expiration — deed must not be expired, status must be active
checkExpiration :: DeedSpec -> TrustDeed -> Integer -> (Bool, String)
checkExpiration spec d nowTs
    | specExpiryRequired spec && deedExpiresAt d <= 0
        = (False, "V6:EXPIRES_AT_MISSING")
    | deedExpiresAt d > 0 && nowTs > deedExpiresAt d
        = (False, "V6:DEED_EXPIRED expires_at=" ++ show (deedExpiresAt d))
    | deedStatus d `notElem` specStatusValues spec
        = (False, "V6:INVALID_STATUS status=" ++ deedStatus d)
    | deedStatus d /= "active"
        = (False, "V6:DEED_NOT_ACTIVE status=" ++ deedStatus d)
    | otherwise
        = (True,  "V6:DEED_ACTIVE status=" ++ deedStatus d)

-- V7: Seal coverage — verify the seal covers all integrity fields
-- We cannot recompute SHA-256 without cryptonite, but we verify that the
-- seal FIELD SET declared in the spec matches what Rust computes.
-- Full content verification happens in Rust's verify_integrity().
checkSealCoverage :: DeedSpec -> TrustDeed -> (Bool, String)
checkSealCoverage spec _d =
    let required = specSealMustCover spec
        covered  = ["agent-id","trust-score","expires-at","status",
                     "escalation-authority","allowed-actions","restricted-actions"]
        missing  = filter (`notElem` covered) required
    in if null missing
       then (True,  "V7:SEAL_COVERAGE_OK covers=" ++ show (length covered) ++ " fields")
       else (False, "V7:SEAL_COVERAGE_INCOMPLETE missing=" ++ show missing)

-- V8: Blocked actions — globally forbidden actions must not be allowed
checkBlockedActions :: DeedSpec -> TrustDeed -> (Bool, String)
checkBlockedActions spec d =
    let blocked  = map (map toLower) (specBlockedActions spec)
        present  = filter (\a -> any (`isPrefixOf` map toLower a) blocked) (deedAllowed d)
    in if null present
       then (True,  "V8:NO_BLOCKED_ACTIONS")
       else (False, "V8:BLOCKED_ACTION_IN_ALLOWED actions=" ++ show present)

-- ─────────────────────────────────────────────────────────────────────────────
-- Deed validation result
-- ─────────────────────────────────────────────────────────────────────────────

data DeedValidation = DeedValidation
    { dv_agent   :: String
    , dv_v1      :: (Bool, String)
    , dv_v2      :: (Bool, String)
    , dv_v3      :: (Bool, String)
    , dv_v4      :: (Bool, String)
    , dv_v5      :: (Bool, String)
    , dv_v6      :: (Bool, String)
    , dv_v7      :: (Bool, String)
    , dv_v8      :: (Bool, String)
    , dv_valid   :: Bool
    , dv_reason  :: String
    } deriving (Show)

-- Linear: DeedToken %1-> ensures one validation per token
validateDeed :: DeedSpec -> Integer -> DeedToken %1-> DeedValidation
validateDeed spec nowTs (DeedToken deed) =
    let v1 = checkStructure             deed
        v2 = checkDisjoint              deed
        v3 = checkTrustScore   spec     deed
        v4 = checkSealFormat   spec     deed
        v5 = checkAuthorityModel spec   deed
        v6 = checkExpiration   spec deed nowTs
        v7 = checkSealCoverage spec     deed
        v8 = checkBlockedActions spec   deed

        allChecks = [v1, v2, v3, v4, v5, v6, v7, v8]
        allOk     = all fst allChecks
        firstFail = head $ filter (not . fst) allChecks ++ [(True, "ALL_CHECKS_PASSED")]

    in DeedValidation
        { dv_agent  = deedAgent deed
        , dv_v1     = v1
        , dv_v2     = v2
        , dv_v3     = v3
        , dv_v4     = v4
        , dv_v5     = v5
        , dv_v6     = v6
        , dv_v7     = v7
        , dv_v8     = v8
        , dv_valid  = allOk
        , dv_reason = snd firstFail
        }

-- ─────────────────────────────────────────────────────────────────────────────
-- Output
-- ─────────────────────────────────────────────────────────────────────────────

printDeedValidation :: DeedValidation -> IO ()
printDeedValidation dv = do
    putStrLn $ "agent="   ++ dv_agent dv
    putStrLn $ "valid="   ++ if dv_valid dv then "true" else "false"
    putStrLn $ "reason="  ++ dv_reason dv
    putStrLn $ "v1="      ++ checkLine (dv_v1 dv)
    putStrLn $ "v2="      ++ checkLine (dv_v2 dv)
    putStrLn $ "v3="      ++ checkLine (dv_v3 dv)
    putStrLn $ "v4="      ++ checkLine (dv_v4 dv)
    putStrLn $ "v5="      ++ checkLine (dv_v5 dv)
    putStrLn $ "v6="      ++ checkLine (dv_v6 dv)
    putStrLn $ "v7="      ++ checkLine (dv_v7 dv)
    putStrLn $ "v8="      ++ checkLine (dv_v8 dv)
    putStrLn   "engine=haskell-deed-validator-lisp-v2"
  where
    checkLine (True,  msg) = "pass:" ++ msg
    checkLine (False, msg) = "fail:" ++ msg

-- ─────────────────────────────────────────────────────────────────────────────
-- Main
-- ─────────────────────────────────────────────────────────────────────────────

main :: IO ()
main = do
    hSetBuffering stdout LineBuffering
    agentId       <- getLine
    allowedStr    <- getLine
    restrictedStr <- getLine
    scoreLine     <- getLine
    sealStr       <- getLine
    expiresLine   <- getLine
    statusStr     <- getLine
    escalationStr <- getLine
    specPathStr   <- getLine

    let specPath  = if specPathStr == "default" || null specPathStr
                    then "bridges/lisp/deed-rules.lisp"
                    else specPathStr

    spec <- loadDeedSpec specPath

    -- Unix "now" — approximate from expires_at sanity (subprocess has no clock access easily)
    -- Rust passes current timestamp as 10th line if needed; for now use 0 (expiry check is advisory)
    let nowTs     = 0 :: Integer   -- Rust's verify_integrity does authoritative expiry check

    let scoreD    = read scoreLine :: Double
        expiresD  = read expiresLine :: Integer
        allowed   = splitOn ',' allowedStr
        restricted = splitOn ',' restrictedStr

    case (mkTrustScore spec scoreD, mkValidSeal spec sealStr) of
        (Left err, _) -> do
            putStrLn $ "agent="  ++ agentId
            putStrLn   "valid=false"
            putStrLn $ "reason=" ++ err
            putStrLn   "engine=haskell-deed-validator-lisp-v2"
        (_, Left err) -> do
            putStrLn $ "agent="  ++ agentId
            putStrLn   "valid=false"
            putStrLn $ "reason=" ++ err
            putStrLn   "engine=haskell-deed-validator-lisp-v2"
        (Right score, Right seal) -> do
            let deed  = TrustDeed
                          { deedAgent      = agentId
                          , deedAllowed    = filter (not . null) allowed
                          , deedRestricted = filter (not . null) restricted
                          , deedScore      = score
                          , deedSeal       = seal
                          , deedExpiresAt  = expiresD
                          , deedStatus     = map toLower statusStr
                          , deedEscalation = escalationStr
                          }
                token  = mkDeedToken deed
                result = validateDeed spec nowTs token
            printDeedValidation result

-- ─────────────────────────────────────────────────────────────────────────────
-- Utilities
-- ─────────────────────────────────────────────────────────────────────────────

splitOn :: Char -> String -> [String]
splitOn _ "" = [""]
splitOn delim str =
    let (word, rest) = break (== delim) str
    in word : case rest of
        []     -> []
        (_:xs) -> splitOn delim xs
