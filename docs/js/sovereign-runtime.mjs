/**
 * Sovereign Runtime Bridge
 * Loads real WASM crypto, provides stable API for Lisp machine + SoulVM
 * Phase: Production Integration
 */

// ============================================================================
// WASM Module Loading
// ============================================================================

let wasmModule = null;
let wasmReady = false;

async function initWasm() {
  try {
    // Dynamic import of WASM module (ES modules)
    const wasmUrl = new URL('../wasm/skclisp_crypto_wasm.js', import.meta.url).href;
    const wasmImport = await import(wasmUrl);

    // Initialize WASM
    await wasmImport.default();

    wasmModule = wasmImport;
    wasmReady = true;

    console.log('[SovereignRuntime] ✓ WASM crypto module initialized');
    return true;
  } catch (err) {
    console.error('[SovereignRuntime] ✗ WASM initialization failed:', err);
    return false;
  }
}

// ============================================================================
// LISP Parser (Real Implementation)
// ============================================================================

/**
 * Parse LISP source code into AST
 * Implements McCarthy-1958 LISP reader
 */
function parseLisp(source) {
  try {
    // Tokenize
    const tokens = tokenizeLisp(source);
    if (tokens.length === 0) {
      return { error: true, message: 'Empty input', phase: 'tokenize' };
    }

    // Parse
    const ast = parseTokens(tokens, 0);
    if (ast.error) {
      return ast;
    }

    return {
      ast: ast.ast,
      tokens: tokens.length,
      valid: true,
      phase: 'parse',
    };
  } catch (err) {
    return {
      error: true,
      message: err.message,
      phase: 'parse',
      stack: err.stack,
    };
  }
}

function tokenizeLisp(source) {
  const tokens = [];
  let i = 0;

  while (i < source.length) {
    const ch = source[i];

    // Skip whitespace
    if (/\s/.test(ch)) {
      i++;
      continue;
    }

    // Parentheses
    if (ch === '(' || ch === ')') {
      tokens.push(ch);
      i++;
      continue;
    }

    // Quote
    if (ch === "'") {
      tokens.push("'");
      i++;
      continue;
    }

    // Comments
    if (ch === ';') {
      while (i < source.length && source[i] !== '\n') i++;
      continue;
    }

    // String literals
    if (ch === '"') {
      let str = '';
      i++;
      while (i < source.length && source[i] !== '"') {
        str += source[i];
        i++;
      }
      i++; // closing quote
      tokens.push(str);
      continue;
    }

    // Numbers and symbols
    let token = '';
    while (i < source.length && !/[\s()';]/.test(source[i])) {
      token += source[i];
      i++;
    }
    if (token) tokens.push(token);
  }

  return tokens;
}

function parseTokens(tokens, pos) {
  if (pos >= tokens.length) {
    return { error: true, message: 'Unexpected end of input' };
  }

  const token = tokens[pos];

  if (token === '(') {
    // List
    const list = [];
    pos++;

    while (pos < tokens.length && tokens[pos] !== ')') {
      const result = parseTokens(tokens, pos);
      if (result.error) return result;
      list.push(result.ast);
      pos = result.pos;
    }

    if (pos >= tokens.length) {
      return { error: true, message: 'Unmatched opening parenthesis' };
    }

    return {
      ast: { type: 'list', elements: list },
      pos: pos + 1,
    };
  } else if (token === ')') {
    return { error: true, message: 'Unexpected closing parenthesis' };
  } else if (token === "'") {
    // Quote
    const result = parseTokens(tokens, pos + 1);
    if (result.error) return result;
    return {
      ast: { type: 'quote', value: result.ast },
      pos: result.pos,
    };
  } else if (/^-?\d+$/.test(token)) {
    // Number
    return {
      ast: { type: 'number', value: parseInt(token) },
      pos: pos + 1,
    };
  } else {
    // Symbol
    return {
      ast: { type: 'symbol', value: token },
      pos: pos + 1,
    };
  }
}

// ============================================================================
// LISP Compiler (Real Evaluation)
// ============================================================================

/**
 * Compile and evaluate LISP expression
 */
function evaluateLisp(ast, env = {}) {
  try {
    const result = evalAst(ast, env);
    return {
      result,
      valid: true,
      phase: 'evaluate',
    };
  } catch (err) {
    return {
      error: true,
      message: err.message,
      phase: 'evaluate',
      stack: err.stack,
    };
  }
}

function evalAst(ast, env) {
  if (ast.type === 'number') {
    return ast.value;
  }

  if (ast.type === 'symbol') {
    if (ast.value in env) return env[ast.value];
    throw new Error(`Unbound symbol: ${ast.value}`);
  }

  if (ast.type === 'quote') {
    return ast.value;
  }

  if (ast.type === 'list' && ast.elements.length > 0) {
    const [fn, ...args] = ast.elements;

    if (fn.type === 'symbol') {
      const fnName = fn.value;
      const fnImpl = builtinFunctions[fnName];

      if (!fnImpl) {
        throw new Error(`Unknown function: ${fnName}`);
      }

      const evaledArgs = args.map(arg => evalAst(arg, env));
      return fnImpl(...evaledArgs);
    }

    throw new Error('First element of list must be a symbol');
  }

  if (ast.type === 'list' && ast.elements.length === 0) {
    return null;
  }

  throw new Error(`Cannot evaluate: ${JSON.stringify(ast)}`);
}

const builtinFunctions = {
  '+': (...args) => args.reduce((a, b) => a + b, 0),
  '-': (...args) => args.length === 0 ? 0 : args.reduce((a, b) => a - b),
  '*': (...args) => args.reduce((a, b) => a * b, 1),
  '/': (...args) => args.length === 0 ? 0 : args.reduce((a, b) => a / b),
  'list': (...args) => args,
  'quote': (x) => x,
  'eq': (a, b) => a === b ? 'T' : 'nil',
  'atom': (x) => (typeof x === 'object' ? 'nil' : 'T'),
  'null': (x) => (x === null || x === 'nil' ? 'T' : 'nil'),
  'car': (lst) => Array.isArray(lst) ? lst[0] : null,
  'cdr': (lst) => Array.isArray(lst) ? lst.slice(1) : null,
  'cons': (x, lst) => [x, ...(Array.isArray(lst) ? lst : [])],
};

// ============================================================================
// EmojiScript Bytecode Execution (Real SoulVM)
// ============================================================================

/**
 * EmojiScript opcode set (15 instructions)
 */
const EMOJI_OPCODES = {
  '🔢': 'PUSH',      // Push constant
  '➕': 'ADD',       // Add top 2 stack items
  '➖': 'SUB',       // Subtract
  '✖️': 'MUL',       // Multiply
  '➗': 'DIV',       // Divide
  '🤝': 'AND',       // Bitwise AND
  '👐': 'OR',        // Bitwise OR
  '🌀': 'XOR',       // Bitwise XOR
  '➡️': 'JUMP',      // Unconditional jump
  '❓': 'JIF',       // Jump if false
  '↩️': 'RET',       // Return
  '🔑': 'CAP',       // Capability check
  '⚡': 'CALL',      // Function call
  '🏗️': 'ALLOC',     // Allocate memory
  '📤': 'LOAD',      // Load from memory
};

/**
 * Execute EmojiScript bytecode
 */
function executeEmojiScript(source) {
  try {
    // Parse emoji operations
    const ops = parseEmojiScript(source);
    if (ops.error) return ops;

    // Execute with SoulVM
    const vm = new SoulVM();
    const result = vm.execute(ops.bytecode);

    return {
      result: result.stack.length > 0 ? result.stack[result.stack.length - 1] : null,
      stack: result.stack,
      registers: result.registers,
      instructionCount: result.instructionCount,
      valid: true,
      phase: 'execute',
    };
  } catch (err) {
    return {
      error: true,
      message: err.message,
      phase: 'execute',
      stack: err.stack,
    };
  }
}

function parseEmojiScript(source) {
  const bytecode = [];
  let i = 0;

  while (i < source.length) {
    // Skip whitespace
    if (/\s/.test(source[i])) {
      i++;
      continue;
    }

    // Try to match emoji
    let found = false;
    for (const [emoji, opcode] of Object.entries(EMOJI_OPCODES)) {
      if (source.slice(i).startsWith(emoji)) {
        if (opcode === 'PUSH') {
          // PUSH must be followed by a number
          i += emoji.length;
          let numStr = '';
          while (i < source.length && /\d/.test(source[i])) {
            numStr += source[i];
            i++;
          }
          if (numStr === '') {
            return { error: true, message: 'PUSH requires a number operand', phase: 'parse' };
          }
          bytecode.push({ op: 'PUSH', operand: parseInt(numStr) });
        } else {
          bytecode.push({ op: opcode });
          i += emoji.length;
        }
        found = true;
        break;
      }
    }

    if (!found) {
      return { error: true, message: `Unknown emoji at position ${i}: ${source[i]}`, phase: 'parse' };
    }
  }

  return { bytecode };
}

class SoulVM {
  constructor() {
    this.stack = [];
    this.registers = { IP: 0, SP: 0 };
    this.memory = new Uint32Array(1024);
    this.instructionCount = 0;
  }

  execute(bytecode) {
    this.registers.IP = 0;

    while (this.registers.IP < bytecode.length) {
      const instr = bytecode[this.registers.IP];
      this.instructionCount++;

      switch (instr.op) {
        case 'PUSH':
          this.stack.push(instr.operand);
          break;
        case 'ADD':
          this.stack.push(this.stack.pop() + this.stack.pop());
          break;
        case 'SUB':
          const b = this.stack.pop();
          this.stack.push(this.stack.pop() - b);
          break;
        case 'MUL':
          this.stack.push(this.stack.pop() * this.stack.pop());
          break;
        case 'DIV':
          const divisor = this.stack.pop();
          this.stack.push(Math.floor(this.stack.pop() / divisor));
          break;
        case 'AND':
          this.stack.push(this.stack.pop() & this.stack.pop());
          break;
        case 'OR':
          this.stack.push(this.stack.pop() | this.stack.pop());
          break;
        case 'XOR':
          this.stack.push(this.stack.pop() ^ this.stack.pop());
          break;
        case 'RET':
          return { stack: this.stack, registers: this.registers, instructionCount: this.instructionCount };
        default:
          throw new Error(`Unknown opcode: ${instr.op}`);
      }

      this.registers.IP++;
    }

    return { stack: this.stack, registers: this.registers, instructionCount: this.instructionCount };
  }
}

// ============================================================================
// WASM Cryptography Bridge
// ============================================================================

async function blake3Hash(data) {
  if (!wasmReady) {
    return { error: true, message: 'WASM module not ready', code: 'NOT_READY' };
  }

  try {
    const bytes = typeof data === 'string' ? new TextEncoder().encode(data) : new Uint8Array(data);
    const hash = wasmModule.blake3_hash(bytes);

    // Convert to hex
    const hexHash = Array.from(hash)
      .map(b => b.toString(16).padStart(2, '0'))
      .join('');

    return {
      hash: hexHash,
      bytes: Array.from(hash),
      valid: true,
      phase: 'blake3',
    };
  } catch (err) {
    return {
      error: true,
      message: err.message,
      phase: 'blake3',
    };
  }
}

async function ed25519Verify(message, signature, publicKey) {
  if (!wasmReady) {
    return { error: true, message: 'WASM module not ready', code: 'NOT_READY' };
  }

  try {
    const msgBytes = typeof message === 'string' ? new TextEncoder().encode(message) : new Uint8Array(message);
    const sigBytes = new Uint8Array(signature);
    const pkBytes = new Uint8Array(publicKey);

    const result = wasmModule.ed25519_verify_wasm(msgBytes, sigBytes, pkBytes);

    return {
      valid: result.valid,
      errorCode: result.error_code,
      phase: 'ed25519',
    };
  } catch (err) {
    return {
      error: true,
      message: err.message,
      phase: 'ed25519',
    };
  }
}

// ============================================================================
// Public API
// ============================================================================

const SovereignRuntime = {
  // Initialization
  async initialize() {
    return await initWasm();
  },

  getStatus() {
    return {
      wasm: wasmReady ? 'ready' : 'loading',
      wasmModule: wasmReady ? 'initialized' : 'not loaded',
    };
  },

  // LISP operations
  parseLisp,
  evaluateLisp,

  // EmojiScript + SoulVM
  executeEmojiScript,

  // Cryptography
  blake3Hash,
  ed25519Verify,

  // Utility
  wasmModule: () => wasmModule,
};

// Export for browser + Node.js
if (typeof window !== 'undefined') {
  window.SovereignRuntime = SovereignRuntime;
}

export default SovereignRuntime;
export { SovereignRuntime, parseLisp, evaluateLisp, executeEmojiScript, blake3Hash, ed25519Verify };
