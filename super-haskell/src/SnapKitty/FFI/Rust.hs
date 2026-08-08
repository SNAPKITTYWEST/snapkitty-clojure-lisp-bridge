{-# LANGUAGE ForeignFunctionInterface #-}
{-# LANGUAGE CApiFFI #-}

-- | Rust FFI Bindings (Crypto + WORM Chain)
module SnapKitty.FFI.Rust
  ( rustScalarMult
  , rustAppendWORM
  ) where

import Foreign.C.Types
import Foreign.Ptr

-- =========================================================================
-- FFI IMPORTS (To Rust Backend)
-- =========================================================================

-- | Scalar multiplication on elliptic curve (verified by Agda, implemented in Rust)
foreign import ccall unsafe "rust_scalar_mult"
  c_rust_scalar_mult :: CInt -> CInt -> CInt -> Ptr CInt -> Ptr CInt -> IO CInt

-- | Append to WORM chain (BLAKE3 attestation)
foreign import ccall unsafe "rust_append_worm"
  c_rust_append_worm :: Ptr CChar -> CInt -> IO CInt

-- =========================================================================
-- HASKELL WRAPPERS
-- =========================================================================

rustScalarMult :: Integer -> (Integer, Integer) -> IO (Integer, Integer)
rustScalarMult k (x, y) = do
  -- TODO: Proper marshalling with alloca
  pure (x * k, y * k) -- Placeholder

rustAppendWORM :: String -> IO ()
rustAppendWORM msg = do
  putStrLn $ "[Rust WORM] " ++ msg
  -- TODO: withCString msg $ \cstr -> c_rust_append_worm cstr (length msg)
