/* @ts-self-types="./hyperkitty_wasm.d.ts" */

/**
 * Route a message from agent `from_idx` (0-15) to the next agent via QRA.
 * Returns {next_agent, glyph_current, glyph_next, wire_byte, worm_id}
 * @param {number} from_idx
 * @param {number} prev_glyph_idx
 * @returns {string}
 */
export function agent_route(from_idx, prev_glyph_idx) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.agent_route(from_idx, prev_glyph_idx);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * Return glyph name for an index (0-5).
 * @param {number} index
 * @returns {string}
 */
export function glyph_name_for(index) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.glyph_name_for(index);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * Wire byte encoding for a glyph index.
 * @param {number} index
 * @returns {number}
 */
export function glyph_wire_byte(index) {
    const ret = wasm.glyph_wire_byte(index);
    return ret;
}

/**
 * Norm squared of the QLG point for a glyph (should always be 1).
 * @param {number} index
 * @returns {number}
 */
export function qlg_norm_sq(index) {
    const ret = wasm.qlg_norm_sq(index);
    return ret;
}

/**
 * Get the QLG sphere point [x, y, z] for a glyph index.
 * @param {number} index
 * @returns {Int32Array}
 */
export function qlg_point_for_glyph(index) {
    const ret = wasm.qlg_point_for_glyph(index);
    var v1 = getArrayI32FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
    return v1;
}

/**
 * Evolve a 3-glyph witness one step. Input/output as flat [curr0,prev0, curr1,prev1, curr2,prev2].
 * Returns [next0, next1, next2] (3 bytes) or empty on bad input.
 * @param {number} w0
 * @param {number} w1
 * @param {number} w2
 * @returns {Uint8Array}
 */
export function qra_evolve_witness(w0, w1, w2) {
    const ret = wasm.qra_evolve_witness(w0, w1, w2);
    var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
    return v1;
}

/**
 * Q[current][previous] — deterministic next glyph index.
 * Returns 255 for invalid input.
 * @param {number} current
 * @param {number} previous
 * @returns {number}
 */
export function qra_next(current, previous) {
    const ret = wasm.qra_next(current, previous);
    return ret;
}

/**
 * Full 6x6 Q tensor as flat 36-byte array (row-major).
 * @returns {Uint8Array}
 */
export function qra_tensor() {
    const ret = wasm.qra_tensor();
    var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
    return v1;
}

/**
 * Returns true if canonical witness [Pi,Gamma,Delta] exhausts in exactly 2 steps.
 * @returns {boolean}
 */
export function qra_validate_exhaustion() {
    const ret = wasm.qra_validate_exhaustion();
    return ret !== 0;
}

/**
 * Get reconciliation JSON for all 6 glyphs.
 * @returns {string}
 */
export function reconcile_all_json() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.reconcile_all_json();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * Reconcile a glyph through all three layers.
 * Returns JSON string: {glyph, qlg_norm_sq, sla_balanced, sla_omega, qra_target, valid}
 * @param {number} index
 * @returns {string}
 */
export function reconcile_glyph(index) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.reconcile_glyph(index);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * Accumulate multiple 24-byte ledgers. Returns 4-byte wire frame.
 * @param {Uint8Array} ledgers_flat
 * @returns {Uint8Array}
 */
export function sla_accumulate(ledgers_flat) {
    const ptr0 = passArray8ToWasm0(ledgers_flat, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.sla_accumulate(ptr0, len0);
    var v2 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
    return v2;
}

/**
 * Compose two 24-byte encoded ledgers. Returns 24-byte result or empty on omega mismatch.
 * @param {Uint8Array} a
 * @param {Uint8Array} b
 * @returns {Uint8Array}
 */
export function sla_compose(a, b) {
    const ptr0 = passArray8ToWasm0(a, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArray8ToWasm0(b, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.sla_compose(ptr0, len0, ptr1, len1);
    var v3 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
    return v3;
}

/**
 * Evolve a 24-byte ledger by a 24-byte increment. Returns result or empty on violation.
 * @param {Uint8Array} base
 * @param {Uint8Array} increment
 * @returns {Uint8Array}
 */
export function sla_evolve(base, increment) {
    const ptr0 = passArray8ToWasm0(base, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArray8ToWasm0(increment, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.sla_evolve(ptr0, len0, ptr1, len1);
    var v3 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
    return v3;
}

/**
 * Hash a symbol string to u64, returned as hex string.
 * @param {string} s
 * @returns {string}
 */
export function sla_hash_symbol(s) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(s, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.sla_hash_symbol(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * Is a 24-byte encoded ledger balanced?
 * @param {Uint8Array} encoded
 * @returns {boolean}
 */
export function sla_is_balanced(encoded) {
    const ptr0 = passArray8ToWasm0(encoded, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.sla_is_balanced(ptr0, len0);
    return ret !== 0;
}

/**
 * Get the canonical SLA ledger for a glyph index. Returns 24-byte encoding.
 * @param {number} index
 * @returns {Uint8Array}
 */
export function sla_ledger_for_glyph(index) {
    const ret = wasm.sla_ledger_for_glyph(index);
    var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
    return v1;
}

/**
 * Create a balanced ledger. Returns 24-byte encoding [s:8][delta:8][omega:8].
 * @param {number} s
 * @param {number} delta
 * @param {number} omega
 * @returns {Uint8Array}
 */
export function sla_ledger_new(s, delta, omega) {
    const ret = wasm.sla_ledger_new(s, delta, omega);
    var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
    return v1;
}

/**
 * Validate the full tripartite isomorphism K_QLG = ω_SLA = target_QRA for all 6 glyphs.
 * @returns {boolean}
 */
export function validate_isomorphism() {
    const ret = wasm.validate_isomorphism();
    return ret !== 0;
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./hyperkitty_wasm_bg.js": import0,
    };
}

function getArrayI32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getInt32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedInt32ArrayMemory0 = null;
function getInt32ArrayMemory0() {
    if (cachedInt32ArrayMemory0 === null || cachedInt32ArrayMemory0.byteLength === 0) {
        cachedInt32ArrayMemory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachedInt32ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedInt32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('hyperkitty_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
