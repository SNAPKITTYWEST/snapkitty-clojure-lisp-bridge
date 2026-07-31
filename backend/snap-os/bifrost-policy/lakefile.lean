import Lake
open Lake DSL

package «bifrostPolicy» where
  name    := "BifrostPolicy"
  srcDir  := "lean"

lean_lib «BifrostPolicy» where
  roots := #[`BifrostPolicy]
