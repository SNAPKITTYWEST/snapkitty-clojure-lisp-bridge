/* tslint:disable */
/* eslint-disable */

/**
 * Route a message from agent `from_idx` (0-15) to the next agent via QRA.
 * Returns {next_agent, glyph_current, glyph_next, wire_byte, worm_id}
 */
export function agent_route(from_idx: number, prev_glyph_idx: number): string;

/**
 * Return glyph name for an index (0-5).
 */
export function glyph_name_for(index: number): string;

/**
 * Wire byte encoding for a glyph index.
 */
export function glyph_wire_byte(index: number): number;

/**
 * Norm squared of the QLG point for a glyph (should always be 1).
 */
export function qlg_norm_sq(index: number): number;

/**
 * Get the QLG sphere point [x, y, z] for a glyph index.
 */
export function qlg_point_for_glyph(index: number): Int32Array;

/**
 * Evolve a 3-glyph witness one step. Input/output as flat [curr0,prev0, curr1,prev1, curr2,prev2].
 * Returns [next0, next1, next2] (3 bytes) or empty on bad input.
 */
export function qra_evolve_witness(w0: number, w1: number, w2: number): Uint8Array;

/**
 * Q[current][previous] — deterministic next glyph index.
 * Returns 255 for invalid input.
 */
export function qra_next(current: number, previous: number): number;

/**
 * Full 6x6 Q tensor as flat 36-byte array (row-major).
 */
export function qra_tensor(): Uint8Array;

/**
 * Returns true if canonical witness [Pi,Gamma,Delta] exhausts in exactly 2 steps.
 */
export function qra_validate_exhaustion(): boolean;

/**
 * Get reconciliation JSON for all 6 glyphs.
 */
export function reconcile_all_json(): string;

/**
 * Reconcile a glyph through all three layers.
 * Returns JSON string: {glyph, qlg_norm_sq, sla_balanced, sla_omega, qra_target, valid}
 */
export function reconcile_glyph(index: number): string;

/**
 * Accumulate multiple 24-byte ledgers. Returns 4-byte wire frame.
 */
export function sla_accumulate(ledgers_flat: Uint8Array): Uint8Array;

/**
 * Compose two 24-byte encoded ledgers. Returns 24-byte result or empty on omega mismatch.
 */
export function sla_compose(a: Uint8Array, b: Uint8Array): Uint8Array;

/**
 * Evolve a 24-byte ledger by a 24-byte increment. Returns result or empty on violation.
 */
export function sla_evolve(base: Uint8Array, increment: Uint8Array): Uint8Array;

/**
 * Hash a symbol string to u64, returned as hex string.
 */
export function sla_hash_symbol(s: string): string;

/**
 * Is a 24-byte encoded ledger balanced?
 */
export function sla_is_balanced(encoded: Uint8Array): boolean;

/**
 * Get the canonical SLA ledger for a glyph index. Returns 24-byte encoding.
 */
export function sla_ledger_for_glyph(index: number): Uint8Array;

/**
 * Create a balanced ledger. Returns 24-byte encoding [s:8][delta:8][omega:8].
 */
export function sla_ledger_new(s: number, delta: number, omega: number): Uint8Array;

/**
 * Validate the full tripartite isomorphism K_QLG = ω_SLA = target_QRA for all 6 glyphs.
 */
export function validate_isomorphism(): boolean;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly agent_route: (a: number, b: number) => [number, number];
    readonly glyph_name_for: (a: number) => [number, number];
    readonly glyph_wire_byte: (a: number) => number;
    readonly qlg_norm_sq: (a: number) => number;
    readonly qlg_point_for_glyph: (a: number) => [number, number];
    readonly qra_evolve_witness: (a: number, b: number, c: number) => [number, number];
    readonly qra_next: (a: number, b: number) => number;
    readonly qra_tensor: () => [number, number];
    readonly qra_validate_exhaustion: () => number;
    readonly reconcile_all_json: () => [number, number];
    readonly reconcile_glyph: (a: number) => [number, number];
    readonly sla_accumulate: (a: number, b: number) => [number, number];
    readonly sla_compose: (a: number, b: number, c: number, d: number) => [number, number];
    readonly sla_evolve: (a: number, b: number, c: number, d: number) => [number, number];
    readonly sla_hash_symbol: (a: number, b: number) => [number, number];
    readonly sla_is_balanced: (a: number, b: number) => number;
    readonly sla_ledger_for_glyph: (a: number) => [number, number];
    readonly sla_ledger_new: (a: number, b: number, c: number) => [number, number];
    readonly validate_isomorphism: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
