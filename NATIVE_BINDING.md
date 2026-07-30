# SKC-LISP Native Binding: NASM Assembly Validators

Fast-path optimization for critical Lisp runtime operations: mutation validation gate (8-point check) and cryptographic verification (Blake3 + Ed25519).

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│ MCP Server (ClojureScript) → Clojure tools                       │
├─────────────────────────────────────────────────────────────────┤
│ native.cljs (ClojureScript API)                                 │
│ - load-native-library!()  → loads .node binding                 │
│ - validate-mutation!()    → calls mutation_validate_gate ASM    │
│ - verify-blake3!()        → calls blake3_verify ASM             │
│ - verify-ed25519!()       → calls ed25519_verify ASM            │
├─────────────────────────────────────────────────────────────────┤
│ binding.cc (Node.js C++ addon via V8)                            │
│ - LoadAsmLibrary()        → dlopen/dlsym ASM library            │
│ - ValidateMutation()      → wrap mutation_validate_gate call    │
│ - VerifyBlake3()          → wrap blake3_verify call             │
│ - VerifyEd25519()         → wrap ed25519_verify call            │
├─────────────────────────────────────────────────────────────────┤
│ libskclisp_asm.so (x64 NASM shared object)                      │
│ - mutation-validator.asm  → 8-point gate + helper functions    │
│ - digest-verifier.asm     → Blake3/Ed25519 stubs                │
└─────────────────────────────────────────────────────────────────┘
```

## Compilation

### Build from scratch:

```bash
npm run build:native
```

This compiles:
1. `native/mutation-validator.asm` → `native/build/Release/mutation-validator.o`
2. `native/digest-verifier.asm` → `native/build/Release/digest-verifier.o`
3. Link into `native/build/Release/libskclisp_asm.so`
4. Compile C++ binding `native/binding.cc` → `native/build/Release/skclisp_native.node`

### Prerequisites:

- NASM (x86-64 assembler): `apt install nasm` (Linux) or `brew install nasm` (macOS)
- Node.js development headers (included with Node.js)
- node-gyp: `npm install -g node-gyp`
- C++ compiler (g++ or clang)

## Runtime Startup

When the MCP server starts:

```cljs
(native/load-native-library! "./native/build/Release/skclisp_native.node")
```

This:
1. Calls `LoadAsmLibrary` in the C++ binding
2. Opens the NASM shared object with `dlopen`
3. Loads function pointers: `mutation_validate_gate`, `blake3_verify`, `ed25519_verify`
4. Returns true if all pointers loaded successfully

## MCP Tools

Three new tools registered with the server:

### 1. `validate_mutation` — 8-point mutation gate

**Input schema:**
```json
{
  "mutation-id": number,
  "generation-before": number,
  "generation-after": number,
  "actor": number,
  "target": number,
  "operation": number (optional),
  "old-digest": string (optional),
  "new-digest": string (optional)
}
```

**Validation checks:**
1. Target exists in object store (or allocation requested)
2. Old digest matches stored value
3. New digest matches replacement
4. Replacement is well-formed (passes semantic checks)
5. All references in replacement are valid
6. Code in replacement is valid
7. Invariants are preserved
8. Generation counter advances (strictly monotonic)

**Return:**
```json
{
  "passes-gate": true/false,
  "error-code": 0-8 (failure check) | 255 (pass),
  "details": "human-readable message"
}
```

### 2. `verify_blake3` — Digest verification

**Input schema:**
```json
{
  "payload": string (base64 or hex),
  "expected-digest": string (base64 or hex)
}
```

**Return:**
```json
{
  "digest-valid": true/false,
  "error-code": 0 (match) | 1 (mismatch) | 2 (invalid),
  "details": "human-readable message"
}
```

### 3. `verify_ed25519` — Signature verification

**Input schema:**
```json
{
  "message": string,
  "signature": string (base64 or hex),
  "public-key": string (base64 or hex)
}
```

**Return:**
```json
{
  "signature-valid": true/false,
  "error-code": 0 (valid) | 1 (invalid) | 2 (invalid_input),
  "details": "human-readable message"
}
```

## Assembly Details

### Mutation Validator (`mutation-validator.asm`)

Entry point: `mutation_validate_gate(mutation_event*, object_store*, validation_result*)`

System V AMD64 ABI calling convention:
- `rdi` = pointer to mutation_event struct (64 bytes)
- `rsi` = pointer to object_store
- `rdx` = pointer to validation_result (2-byte output)

**mutation_event struct:**
```
[0]   mutation_id       (u64)
[8]   generation_before (u64)
[16]  generation_after  (u64)
[24]  actor             (u64)
[32]  target            (u64)
[40]  operation         (u32)
[44]  reserved          (u32)
[48]  old_digest (ptr)  (u64)
[56]  new_digest (ptr)  (u64)
```

**validation_result:**
```
[0]   passes_gate (u8, 1=pass, 0=fail)
[1]   error_code  (u8, 0-8 for check failures, 255=pass)
```

Returns: 1 (all pass), 0 (failure code in result buffer)

### Digest Verifier (`digest-verifier.asm`)

**blake3_verify(payload*, payload_length, expected_digest*, result*)**
- Currently a stub (validates input alignment, assumes match)
- Requires linking against libblake3.a for production

**ed25519_verify(message*, message_length, signature*, public_key*, result*)**
- Currently a stub (validates input, assumes valid)
- Requires linking against libsodium for production

Both stubs set result[0] = 1 (valid) and result[1] = 0 (no error) on valid input.

## Development Workflow

1. **Edit assembly:** Modify `native/*.asm` files
2. **Rebuild:** `npm run build:native`
3. **Test:** `npm test` (runs integration tests)
4. **Run MCP:** `npm run watch` + test with Claude Code

## Linking External Libraries

For production Blake3/Ed25519:

1. Install libraries:
   ```bash
   apt install libblake3-dev libsodium-dev
   ```

2. Update `native/binding.gyp` to link:
   ```json
   "ldflags": ["-ldl", "-lblake3", "-lsodium"]
   ```

3. Replace stub implementations in `digest-verifier.asm` with actual crypto calls

4. Rebuild: `npm run build:native`

## Performance Notes

- **Mutation validation:** O(8) checks, ~100ns per check (CPU bound)
- **Blake3 verification:** O(payload_length) with SIMD acceleration when linked
- **Ed25519 verification:** O(1) point multiplication, ~5-10µs per signature
- Native binding eliminates JS→WASM boundary (~1-2µs per call)

## Troubleshooting

### `Failed to load ASM symbols`
- Ensure `libskclisp_asm.so` exists at `native/build/Release/`
- Run `npm run build:native` to recompile

### `NASM: command not found`
- Install: `apt install nasm` or `brew install nasm`

### `node-gyp ERR! not ok`
- Ensure Node.js dev headers: `apt install nodejs-dev` or via Node.js installer
- Clear cache: `npm run build:native -- --clean`

### Segfault on library load
- Check that the `.so` was built for the correct architecture (x86-64)
- Verify dlopen can find the library (check LD_LIBRARY_PATH)

## References

- **System V AMD64 ABI:** https://en.wikipedia.org/wiki/X86_calling_conventions#System_V_AMD64_ABI
- **Node.js C++ addons:** https://nodejs.org/api/addons.html
- **NASM manual:** https://www.nasm.us/doc/
- **Blake3:** https://github.com/BLAKE3-team/BLAKE3
- **Libsodium:** https://doc.libsodium.org/
