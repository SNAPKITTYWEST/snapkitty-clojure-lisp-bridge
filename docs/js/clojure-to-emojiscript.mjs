/**
 * Clojure → EmojiScript Bytecode Compiler
 * Phase 1: Browser-executable compiler
 * Phase 2: CLI tool for backend/terminal
 *
 * Maps Clojure semantics to 15-opcode SoulVM bytecode
 * Runs in-browser or as Node.js CLI depending on size
 */

// ============================================================================
// CLOJURE PARSER — Understand Clojure/EDN syntax
// ============================================================================

class ClojureParser {
  constructor(source) {
    this.source = source;
    this.pos = 0;
    this.line = 1;
    this.col = 1;
  }

  parse() {
    const forms = [];
    while (this.pos < this.source.length) {
      this.skipWhitespaceAndComments();
      if (this.pos >= this.source.length) break;
      forms.push(this.parseForm());
    }
    return forms;
  }

  parseForm() {
    this.skipWhitespaceAndComments();
    const ch = this.peek();

    if (ch === '(') return this.parseList();
    if (ch === '[') return this.parseVector();
    if (ch === '{') return this.parseMap();
    if (ch === "'") return this.parseQuote();
    if (ch === '"') return this.parseString();
    if (ch === ':') return this.parseKeyword();
    if (/[0-9-]/.test(ch)) return this.parseNumber();
    if (/[a-zA-Z_*+?!<>=&|]/.test(ch)) return this.parseSymbol();

    throw new Error(`Unexpected character: ${ch} at line ${this.line}, col ${this.col}`);
  }

  parseList() {
    this.consume('(');
    const elements = [];
    this.skipWhitespaceAndComments();

    while (this.peek() !== ')') {
      elements.push(this.parseForm());
      this.skipWhitespaceAndComments();
    }

    this.consume(')');
    return { type: 'list', elements };
  }

  parseVector() {
    this.consume('[');
    const elements = [];
    this.skipWhitespaceAndComments();

    while (this.peek() !== ']') {
      elements.push(this.parseForm());
      this.skipWhitespaceAndComments();
    }

    this.consume(']');
    return { type: 'vector', elements };
  }

  parseMap() {
    this.consume('{');
    const pairs = [];
    this.skipWhitespaceAndComments();

    while (this.peek() !== '}') {
      const key = this.parseForm();
      this.skipWhitespaceAndComments();
      const value = this.parseForm();
      pairs.push([key, value]);
      this.skipWhitespaceAndComments();
    }

    this.consume('}');
    return { type: 'map', pairs };
  }

  parseQuote() {
    this.consume("'");
    const form = this.parseForm();
    return { type: 'quote', value: form };
  }

  parseString() {
    this.consume('"');
    let str = '';
    while (this.peek() !== '"') {
      if (this.peek() === '\\') {
        this.pos++;
        const escaped = this.source[this.pos];
        str += escaped === 'n' ? '\n' : escaped === 't' ? '\t' : escaped;
      } else {
        str += this.peek();
      }
      this.pos++;
    }
    this.consume('"');
    return { type: 'string', value: str };
  }

  parseKeyword() {
    this.consume(':');
    let kw = '';
    while (this.pos < this.source.length && /[a-zA-Z0-9_-]/.test(this.peek())) {
      kw += this.peek();
      this.pos++;
    }
    return { type: 'keyword', value: kw };
  }

  parseNumber() {
    let num = '';
    if (this.peek() === '-') num += this.consume('-');
    while (this.pos < this.source.length && /[0-9.]/.test(this.peek())) {
      num += this.peek();
      this.pos++;
    }
    return { type: 'number', value: parseFloat(num) };
  }

  parseSymbol() {
    let sym = '';
    while (this.pos < this.source.length && /[a-zA-Z0-9_*+?!<>=&|/-]/.test(this.peek())) {
      sym += this.peek();
      this.pos++;
    }
    return { type: 'symbol', value: sym };
  }

  skipWhitespaceAndComments() {
    while (this.pos < this.source.length) {
      const ch = this.peek();

      if (/\s/.test(ch)) {
        if (ch === '\n') {
          this.line++;
          this.col = 0;
        }
        this.pos++;
        this.col++;
      } else if (ch === ';') {
        while (this.pos < this.source.length && this.peek() !== '\n') {
          this.pos++;
        }
      } else {
        break;
      }
    }
  }

  peek() {
    return this.source[this.pos];
  }

  consume(expected) {
    if (this.peek() !== expected) {
      throw new Error(`Expected '${expected}', got '${this.peek()}' at line ${this.line}`);
    }
    this.pos++;
    this.col++;
    return expected;
  }
}

// ============================================================================
// CLOJURE → EMOJISCRIPT COMPILER
// ============================================================================

class ClojureToEmojiScript {
  constructor() {
    this.bytecode = [];
    this.constants = [];
    this.symbolTable = new Map();
    this.labels = new Map();
    this.labelCounter = 0;
  }

  compile(clojureSource) {
    try {
      const parser = new ClojureParser(clojureSource);
      const forms = parser.parse();

      for (const form of forms) {
        this.compileForm(form);
      }

      return {
        success: true,
        bytecode: this.bytecode,
        constants: this.constants,
        symbolTable: Object.fromEntries(this.symbolTable),
        emojiscript: this.generateEmojiscript(),
      };
    } catch (err) {
      return {
        success: false,
        error: err.message,
        phase: 'compile',
      };
    }
  }

  compileForm(form) {
    if (form.type === 'number') {
      this.emitPush(form.value);
    } else if (form.type === 'string') {
      this.emitPush(form.value);
    } else if (form.type === 'symbol') {
      this.handleSymbol(form.value);
    } else if (form.type === 'list') {
      this.compileList(form);
    } else if (form.type === 'vector') {
      this.compileVector(form);
    } else if (form.type === 'keyword') {
      this.emitPush(`:${form.value}`);
    } else if (form.type === 'quote') {
      this.emitPush(form.value); // Quoted form as literal
    }
  }

  compileList(form) {
    if (form.elements.length === 0) {
      this.emitPush(null);
      return;
    }

    const [fn, ...args] = form.elements;

    // Built-in operations
    if (fn.type === 'symbol') {
      const op = fn.value;

      // Arithmetic
      if (op === '+') {
        for (const arg of args) this.compileForm(arg);
        for (let i = 1; i < args.length; i++) {
          this.bytecode.push('➕'); // ADD
        }
        return;
      }

      if (op === '-') {
        for (const arg of args) this.compileForm(arg);
        for (let i = 1; i < args.length; i++) {
          this.bytecode.push('➖'); // SUB
        }
        return;
      }

      if (op === '*') {
        for (const arg of args) this.compileForm(arg);
        for (let i = 1; i < args.length; i++) {
          this.bytecode.push('✖️'); // MUL
        }
        return;
      }

      if (op === '/') {
        for (const arg of args) this.compileForm(arg);
        for (let i = 1; i < args.length; i++) {
          this.bytecode.push('➗'); // DIV
        }
        return;
      }

      // Logic
      if (op === 'and') {
        for (const arg of args) this.compileForm(arg);
        for (let i = 1; i < args.length; i++) {
          this.bytecode.push('🤝'); // AND
        }
        return;
      }

      if (op === 'or') {
        for (const arg of args) this.compileForm(arg);
        for (let i = 1; i < args.length; i++) {
          this.bytecode.push('👐'); // OR
        }
        return;
      }

      // Control flow
      if (op === 'if') {
        const [cond, thenForm, elseForm] = args;
        this.compileForm(cond);
        const jifLabel = this.genLabel();
        this.bytecode.push('❓'); // JIF
        this.bytecode.push(jifLabel);
        this.compileForm(thenForm);
        const endLabel = this.genLabel();
        this.bytecode.push('➡️'); // JUMP
        this.bytecode.push(endLabel);
        this.labels.set(jifLabel, this.bytecode.length);
        this.compileForm(elseForm || null);
        this.labels.set(endLabel, this.bytecode.length);
        return;
      }

      // Function call (generic)
      this.compileForm(fn);
      for (const arg of args) {
        this.compileForm(arg);
      }
      this.bytecode.push('⚡'); // CALL
      this.bytecode.push(args.length);
      return;
    }

    // Quoted form as list
    for (const elem of form.elements) {
      this.compileForm(elem);
    }
  }

  compileVector(form) {
    for (const elem of form.elements) {
      this.compileForm(elem);
    }
    this.bytecode.push('🏗️'); // ALLOC for vector
    this.bytecode.push(form.elements.length);
  }

  handleSymbol(sym) {
    if (!this.symbolTable.has(sym)) {
      this.symbolTable.set(sym, this.symbolTable.size);
    }
    this.bytecode.push('📤'); // LOAD
    this.bytecode.push(this.symbolTable.get(sym));
  }

  emitPush(value) {
    this.constants.push(value);
    this.bytecode.push('🔢'); // PUSH
    this.bytecode.push(this.constants.length - 1);
  }

  genLabel() {
    return `__label_${this.labelCounter++}`;
  }

  generateEmojiscript() {
    return this.bytecode.join(' ');
  }
}

// ============================================================================
// EXPORT API — Browser + CLI compatible
// ============================================================================

export class ClojureCompiler {
  static compile(clojureSource) {
    const compiler = new ClojureToEmojiScript();
    return compiler.compile(clojureSource);
  }

  static compileFile(path) {
    // Node.js only
    if (typeof require !== 'undefined') {
      const fs = require('fs');
      const source = fs.readFileSync(path, 'utf-8');
      return this.compile(source);
    }
    throw new Error('File I/O not available in browser');
  }

  static generateCLI() {
    return `#!/usr/bin/env node
// Clojure → EmojiScript Bytecode Compiler CLI
// Usage: ./clojure-to-emojiscript.mjs <file.clj> [output.emojis]

import { ClojureCompiler } from './clojure-to-emojiscript.mjs';
import fs from 'fs';
import path from 'path';

const [inputFile, outputFile] = process.argv.slice(2);

if (!inputFile) {
  console.error('Usage: clojure-to-emojiscript.mjs <file.clj> [output.emojis]');
  process.exit(1);
}

try {
  const source = fs.readFileSync(inputFile, 'utf-8');
  const result = ClojureCompiler.compile(source);

  if (!result.success) {
    console.error('Compilation failed:', result.error);
    process.exit(1);
  }

  const output = outputFile || inputFile.replace(/\\.clj$/, '.emojis');
  fs.writeFileSync(output, result.emojiscript, 'utf-8');

  console.log(\`✓ Compiled: \${inputFile} → \${output}\`);
  console.log(\`  Bytecode: \${result.bytecode.length} ops\`);
  console.log(\`  Constants: \${result.constants.length}\`);
  console.log(\`  Symbols: \${Object.keys(result.symbolTable).length}\`);
} catch (err) {
  console.error('Error:', err.message);
  process.exit(1);
}`;
  }
}

export default ClojureCompiler;
