var LispToVM = {
  opcodes: {
    'boot':          { bytecode: 'PUSH 0xA1 CALL_VERIFY CALL_SEAL HALT', steps: 4 },
    'agent-status':  { bytecode: 'PUSH 0xB2 PRINT HALT', steps: 3 },
    'trust-score':   { bytecode: 'PUSH 0xC1 PRINT HALT', steps: 3 },
    'seal-state':    { bytecode: 'PUSH 0xE5 PRINT HALT', steps: 3 },
    'woz-vault-count': { bytecode: 'PUSH 0xC9 PRINT HALT', steps: 3 },
    'draw-agent':    { bytecode: 'PUSH 0xB2 PRINT HALT', steps: 3 },
    'debate':        { bytecode: 'PUSH 0xB7 CALL_VERIFY PUSH 1 CALL_SEAL HALT', steps: 5 },
    'export':        { bytecode: 'PUSH 0xDB CALL_VERIFY CALL_SEAL HALT', steps: 4 },
    'quote':         { bytecode: 'PUSH 0x01 HALT', steps: 2 },
    '+':             { bytecode: 'PUSH ADD PRINT HALT', steps: 4 },
    '-':             { bytecode: 'PUSH SUB PRINT HALT', steps: 4 },
    '*':             { bytecode: 'PUSH MUL PRINT HALT', steps: 4 },
    '/':             { bytecode: 'PUSH DIV PRINT HALT', steps: 4 }
  },

  convert: function(lispExpr) {
    var parsed = SExprParser.parse(lispExpr);
    if (!parsed) return { input: lispExpr, bytecode: 'HALT', steps: 1, ops: [] };

    if (Array.isArray(parsed)) {
      var op = parsed[0];
      var args = parsed.slice(1);

      if (typeof op === 'string' && this.opcodes[op]) {
        var entry = this.opcodes[op];
        var bytecode = entry.bytecode;
        // For arithmetic, add operands
        if (op === '+' || op === '-' || op === '*' || op === '/') {
          var nums = args.filter(function(a) { return typeof a === 'number'; });
          var pushes = nums.map(function(n) { return 'PUSH ' + n; }).join(' ');
          bytecode = pushes + ' ' + (op === '+' ? 'ADD' : op === '-' ? 'SUB' : op === '*' ? 'MUL' : 'DIV') + ' PRINT HALT';
        }
        return {
          input: lispExpr,
          op: op,
          bytecode: bytecode,
          steps: entry.steps + nums.length,
          ops: bytecode.split(/\s+/)
        };
      }
      // Unknown form — generate generic bytecode
      return {
        input: lispExpr,
        op: String(op),
        bytecode: 'PUSH 0xFF PRINT HALT',
        steps: 3,
        ops: ['PUSH', '0xFF', 'PRINT', 'HALT']
      };
    }
    // Atom
    return {
      input: lispExpr,
      op: String(parsed),
      bytecode: 'PUSH ' + parsed + ' PRINT HALT',
      steps: 3,
      ops: ['PUSH', String(parsed), 'PRINT', 'HALT']
    };
  },

  render: function(r) {
    return [
      '╔══════════════════════════════════════════════════════════╗',
      '║  LISP → VM COMPILER (SIMULATED)                        ║',
      '╠══════════════════════════════════════════════════════════╣',
      '║  INPUT:     ' + r.input.padEnd(45) + '║',
      '║  OP:        ' + r.op.padEnd(45) + '║',
      '║  BYTECODE:  ' + r.bytecode.substring(0, 45).padEnd(45) + '║',
      '║  STEPS:     ' + String(r.steps).padEnd(45) + '║',
      '║  OPS:       ' + r.ops.join(' ').substring(0, 45).padEnd(45) + '║',
      '╚══════════════════════════════════════════════════════════╝'
    ].join('\n');
  }
};
