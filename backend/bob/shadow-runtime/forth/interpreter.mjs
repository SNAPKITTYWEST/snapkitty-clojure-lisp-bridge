// Sovereign Forth Runtime — the ancient output engine.
// Stack-based. 1970s lineage. Charles Moore's language.
// We use it because dependency graphs ARE stacks —
// push dependency, resolve, pop. Natural representation.
// This interpreter runs .forth files the worm generates.

export class ForthMachine {
  constructor() {
    this.stack    = [];
    this.rstack   = [];  // return stack
    this.dict     = {};  // word definitions
    this.memory   = new Map();
    this.output   = [];
    this._installBuiltins();
  }

  // ── Stack ops ─────────────────────────────────────────
  push(v)  { this.stack.push(v); }
  pop()    { if (!this.stack.length) throw new Error('stack underflow'); return this.stack.pop(); }
  peek()   { return this.stack[this.stack.length - 1]; }
  dup()    { this.push(this.peek()); }
  swap()   { const a = this.pop(), b = this.pop(); this.push(a); this.push(b); }
  drop()   { this.pop(); }
  over()   { const a = this.pop(), b = this.peek(); this.push(a); this.push(b); this.push(a); }

  emit(s)  { this.output.push(String(s)); }

  // ── Execute a Forth source string ─────────────────────
  run(source) {
    const tokens = this._tokenize(source);
    let i = 0;

    while (i < tokens.length) {
      const tok = tokens[i];

      // Word definition: : name ... ;
      if (tok === ':') {
        const name = tokens[++i];
        const body = [];
        while (tokens[++i] !== ';') body.push(tokens[i]);
        this.dict[name.toLowerCase()] = body;
        i++;
        continue;
      }

      // Variable declaration
      if (tok === 'variable') {
        const name = tokens[++i].toLowerCase();
        this.memory.set(name, 0);
        this.dict[name]         = () => this.push(name);
        this.dict[name + '!']   = () => { const v = this.pop(); this.memory.set(name, v); };
        this.dict[name + '@']   = () => this.push(this.memory.get(name) ?? 0);
        i++;
        continue;
      }

      // String literal: ." text "
      if (tok === '."') {
        let str = '';
        while (tokens[++i] !== '"') str += tokens[i] + ' ';
        this.emit(str.trim());
        i++;
        continue;
      }

      this._execToken(tok);
      i++;
    }
  }

  _execToken(tok) {
    const low = tok.toLowerCase();

    // Number literal
    if (!isNaN(tok) && tok !== '') { this.push(Number(tok)); return; }

    // Float
    if (!isNaN(parseFloat(tok))) { this.push(parseFloat(tok)); return; }

    // String push
    if (tok.startsWith('"') && tok.endsWith('"')) {
      this.push(tok.slice(1, -1)); return;
    }

    // User-defined word
    if (this.dict[low]) {
      const word = this.dict[low];
      if (typeof word === 'function') { word(); return; }
      if (Array.isArray(word)) { word.forEach(t => this._execToken(t)); return; }
    }

    throw new Error(`unknown word: ${tok}`);
  }

  _tokenize(src) {
    return src
      .replace(/\( [^)]*\)/g, '') // strip ( comments )
      .replace(/\\[^\n]*/g, '')   // strip \ line comments
      .split(/\s+/)
      .filter(Boolean);
  }

  _installBuiltins() {
    const d = this.dict;

    // Arithmetic
    d['+']    = () => { const b = this.pop(), a = this.pop(); this.push(a + b); };
    d['-']    = () => { const b = this.pop(), a = this.pop(); this.push(a - b); };
    d['*']    = () => { const b = this.pop(), a = this.pop(); this.push(a * b); };
    d['/']    = () => { const b = this.pop(), a = this.pop(); this.push(a / b); };
    d['mod']  = () => { const b = this.pop(), a = this.pop(); this.push(a % b); };

    // Comparison
    d['=']    = () => { const b = this.pop(), a = this.pop(); this.push(a === b ? -1 : 0); };
    d['<']    = () => { const b = this.pop(), a = this.pop(); this.push(a < b  ? -1 : 0); };
    d['>']    = () => { const b = this.pop(), a = this.pop(); this.push(a > b  ? -1 : 0); };
    d['0=']   = () => this.push(this.pop() === 0 ? -1 : 0);
    d['0<']   = () => this.push(this.pop() <  0 ? -1 : 0);

    // Stack
    d['dup']  = () => this.dup();
    d['drop'] = () => this.drop();
    d['swap'] = () => this.swap();
    d['over'] = () => this.over();
    d['rot']  = () => { const c=this.pop(),b=this.pop(),a=this.pop(); this.push(b);this.push(c);this.push(a); };
    d['depth']= () => this.push(this.stack.length);
    d['.']    = () => this.emit(this.pop());
    d['cr']   = () => this.emit('\n');
    d['space']= () => this.emit(' ');

    // Logic
    d['and']  = () => { const b=this.pop(),a=this.pop(); this.push(a & b); };
    d['or']   = () => { const b=this.pop(),a=this.pop(); this.push(a | b); };
    d['invert']= () => this.push(~this.pop());
    d['not']  = () => this.push(this.pop() === 0 ? -1 : 0);

    // Memory
    d['!']    = () => { const addr=this.pop(), val=this.pop(); this.memory.set(addr, val); };
    d['@']    = () => this.push(this.memory.get(this.pop()) ?? 0);

    // Sovereign words — the graveyard vocabulary
    d['gravity']    = () => { /* gravity score is on stack */ };
    d['alive?']     = () => this.push(this.peek() >= 0.6 ? -1 : 0);
    d['broken?']    = () => { const g=this.peek(); this.push(g >= 0.3 && g < 0.6 ? -1 : 0); };
    d['orphan?']    = () => this.push(this.peek() < 0.3 ? -1 : 0);
    d['invert-repo']= () => this.emit(`[INVERT] ${this.pop()}`);
    d['seal']       = () => this.emit(`[SEAL] ${this.pop()}`);
    d['flag']       = () => this.emit(`[FLAG] ${this.pop()}`);
    d['repair']     = () => this.emit(`[REPAIR] ${this.pop()}`);
  }

  getOutput() { return this.output.join(''); }
}

// Run a Forth file and return output
export function runForth(source) {
  const machine = new ForthMachine();
  machine.run(source);
  return machine.getOutput();
}
