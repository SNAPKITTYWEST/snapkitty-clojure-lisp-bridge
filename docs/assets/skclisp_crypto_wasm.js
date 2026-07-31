export class Blake3VerificationResult {
    static __wrap(ptr) {
        const obj = Object.create(Blake3VerificationResult.prototype);
        obj.__wbg_ptr = ptr;
        Blake3VerificationResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        Blake3VerificationResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_blake3verificationresult_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get error_code() {
        const ret = wasm.__wbg_get_blake3verificationresult_error_code(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {boolean}
     */
    get valid() {
        const ret = wasm.__wbg_get_blake3verificationresult_valid(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {number} arg0
     */
    set error_code(arg0) {
        wasm.__wbg_set_blake3verificationresult_error_code(this.__wbg_ptr, arg0);
    }
    /**
     * @param {boolean} arg0
     */
    set valid(arg0) {
        wasm.__wbg_set_blake3verificationresult_valid(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) Blake3VerificationResult.prototype[Symbol.dispose] = Blake3VerificationResult.prototype.free;

export class Ed25519VerificationResult {
    static __wrap(ptr) {
        const obj = Object.create(Ed25519VerificationResult.prototype);
        obj.__wbg_ptr = ptr;
        Ed25519VerificationResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        Ed25519VerificationResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_ed25519verificationresult_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get error_code() {
        const ret = wasm.__wbg_get_ed25519verificationresult_error_code(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {boolean}
     */
    get valid() {
        const ret = wasm.__wbg_get_ed25519verificationresult_valid(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {number} arg0
     */
    set error_code(arg0) {
        wasm.__wbg_set_ed25519verificationresult_error_code(this.__wbg_ptr, arg0);
    }
    /**
     * @param {boolean} arg0
     */
    set valid(arg0) {
        wasm.__wbg_set_ed25519verificationresult_valid(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) Ed25519VerificationResult.prototype[Symbol.dispose] = Ed25519VerificationResult.prototype.free;

export class MutationValidationResult {
    static __wrap(ptr) {
        const obj = Object.create(MutationValidationResult.prototype);
        obj.__wbg_ptr = ptr;
        MutationValidationResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        MutationValidationResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_mutationvalidationresult_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get error_code() {
        const ret = wasm.__wbg_get_mutationvalidationresult_error_code(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {boolean}
     */
    get valid() {
        const ret = wasm.__wbg_get_mutationvalidationresult_valid(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {number} arg0
     */
    set error_code(arg0) {
        wasm.__wbg_set_mutationvalidationresult_error_code(this.__wbg_ptr, arg0);
    }
    /**
     * @param {boolean} arg0
     */
    set valid(arg0) {
        wasm.__wbg_set_mutationvalidationresult_valid(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) MutationValidationResult.prototype[Symbol.dispose] = MutationValidationResult.prototype.free;

export class ProofCertificateValidationResult {
    static __wrap(ptr) {
        const obj = Object.create(ProofCertificateValidationResult.prototype);
        obj.__wbg_ptr = ptr;
        ProofCertificateValidationResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ProofCertificateValidationResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_proofcertificatevalidationresult_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get error_code() {
        const ret = wasm.__wbg_get_proofcertificatevalidationresult_error_code(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get theorem_id() {
        const ret = wasm.__wbg_get_proofcertificatevalidationresult_theorem_id(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get theorems_covered() {
        const ret = wasm.__wbg_get_proofcertificatevalidationresult_theorems_covered(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {boolean}
     */
    get valid() {
        const ret = wasm.__wbg_get_proofcertificatevalidationresult_valid(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {number} arg0
     */
    set error_code(arg0) {
        wasm.__wbg_set_proofcertificatevalidationresult_error_code(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set theorem_id(arg0) {
        wasm.__wbg_set_proofcertificatevalidationresult_theorem_id(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set theorems_covered(arg0) {
        wasm.__wbg_set_proofcertificatevalidationresult_theorems_covered(this.__wbg_ptr, arg0);
    }
    /**
     * @param {boolean} arg0
     */
    set valid(arg0) {
        wasm.__wbg_set_proofcertificatevalidationresult_valid(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) ProofCertificateValidationResult.prototype[Symbol.dispose] = ProofCertificateValidationResult.prototype.free;

/**
 * Compute Blake3 hash of input (returns 32-byte digest)
 * @param {Uint8Array} input
 * @returns {Uint8Array}
 */
export function blake3_hash(input) {
    const ptr0 = passArray8ToWasm0(input, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.blake3_hash(ptr0, len0);
    var v2 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
    return v2;
}

/**
 * Verify Blake3 digest matches expected value
 * @param {Uint8Array} payload
 * @param {Uint8Array} expected_digest
 * @returns {Blake3VerificationResult}
 */
export function blake3_verify_wasm(payload, expected_digest) {
    const ptr0 = passArray8ToWasm0(payload, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArray8ToWasm0(expected_digest, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.blake3_verify_wasm(ptr0, len0, ptr1, len1);
    return Blake3VerificationResult.__wrap(ret);
}

/**
 * Verify Ed25519 signature
 * @param {Uint8Array} message
 * @param {Uint8Array} signature
 * @param {Uint8Array} public_key
 * @returns {Ed25519VerificationResult}
 */
export function ed25519_verify_wasm(message, signature, public_key) {
    const ptr0 = passArray8ToWasm0(message, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArray8ToWasm0(signature, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passArray8ToWasm0(public_key, wasm.__wbindgen_malloc);
    const len2 = WASM_VECTOR_LEN;
    const ret = wasm.ed25519_verify_wasm(ptr0, len0, ptr1, len1, ptr2, len2);
    return Ed25519VerificationResult.__wrap(ret);
}

export function init_wasm() {
    wasm.init_wasm();
}

/**
 * 8-point mutation validation gate (ported from NASM)
 * @param {number} event_id
 * @param {number} generation
 * @param {Uint8Array} source_hash
 * @param {Uint8Array} bytecode_hash
 * @param {Uint8Array} native_code_hash
 * @param {Uint8Array} actor_signature
 * @returns {MutationValidationResult}
 */
export function validate_mutation_wasm(event_id, generation, source_hash, bytecode_hash, native_code_hash, actor_signature) {
    const ptr0 = passArray8ToWasm0(source_hash, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArray8ToWasm0(bytecode_hash, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passArray8ToWasm0(native_code_hash, wasm.__wbindgen_malloc);
    const len2 = WASM_VECTOR_LEN;
    const ptr3 = passArray8ToWasm0(actor_signature, wasm.__wbindgen_malloc);
    const len3 = WASM_VECTOR_LEN;
    const ret = wasm.validate_mutation_wasm(event_id, generation, ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3);
    return MutationValidationResult.__wrap(ret);
}

/**
 * Validate proof certificate structure + signature
 * @param {Uint8Array} cert_bytes
 * @returns {ProofCertificateValidationResult}
 */
export function validate_proof_certificate_wasm(cert_bytes) {
    const ptr0 = passArray8ToWasm0(cert_bytes, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.validate_proof_certificate_wasm(ptr0, len0);
    return ProofCertificateValidationResult.__wrap(ret);
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_throw_344f42d3211c4765: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_error_a6fa202b58aa1cd3: function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_new_227d7c05414eb861: function() {
            const ret = new Error();
            return ret;
        },
        __wbg_stack_3b0d974bbf31e44f: function(arg0, arg1) {
            const ret = arg1.stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
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
        "./skclisp_crypto_wasm_bg.js": import0,
    };
}

const Blake3VerificationResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_blake3verificationresult_free(ptr, 1));
const Ed25519VerificationResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_ed25519verificationresult_free(ptr, 1));
const MutationValidationResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_mutationvalidationresult_free(ptr, 1));
const ProofCertificateValidationResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_proofcertificatevalidationresult_free(ptr, 1));

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
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
    cachedDataViewMemory0 = null;
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
        module_or_path = new URL('skclisp_crypto_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
