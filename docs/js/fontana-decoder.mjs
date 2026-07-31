var FontanaDecoder = {
  dictionary: {
    'agent':    { lisp: 'agent-status',    vm: 'AGENT_STATUS',  bytes: '0xB2' },
    'trust':    { lisp: 'trust-score',     vm: 'TRUST_SCORE',   bytes: '0xC1' },
    'audit':    { lisp: 'audit-log',       vm: 'AUDIT_READ',    bytes: '0xD3' },
    'seal':     { lisp: 'seal-state',      vm: 'SEAL_CHECK',    bytes: '0xE5' },
    'boot':     { lisp: 'boot-system',     vm: 'BOOT_SEQ',      bytes: '0xA1' },
    'debate':   { lisp: 'run-debate',      vm: 'DEBATE_INIT',   bytes: '0xB7' },
    'vault':    { lisp: 'vault-read',      vm: 'VAULT_ACCESS',  bytes: '0xC9' },
    'export':   { lisp: 'export-snapshot', vm: 'EXPORT_DATA',   bytes: '0xDB' },
    'lisp':     { lisp: 'eval-expr',       vm: 'LISP_EVAL',     bytes: '0xED' },
    'runtime':  { lisp: 'runtime-status',  vm: 'RT_STATUS',     bytes: '0xFF' },
    'verified': { lisp: '(quote verified)', vm: 'PUSH_VERIFIED', bytes: '0x01' },
    'sealed':   { lisp: '(quote sealed)',   vm: 'PUSH SEALED',   bytes: '0x02' },
    'approved': { lisp: '(quote approved)', vm: 'PUSH_APPROVED', bytes: '0x03' },
    'denied':   { lisp: '(quote denied)',   vm: 'PUSH_DENIED',   bytes: '0x04' },
    'active':   { lisp: '(quote active)',   vm: 'PUSH_ACTIVE',   bytes: '0x05' },
    'online':   { lisp: '(quote online)',   vm: 'PUSH_ONLINE',   bytes: '0x06' }
  },

  decode: function(fontanaOutput) {
    var tokens = fontanaOutput.trim().split(/\s+/);
    var self = this;
    var decoded = tokens.map(function(t) {
      var entry = self.dictionary[t.toLowerCase()];
      return entry ? entry.lisp : t;
    });
    var lispExpr = '(' + decoded.join(' ') + ')';
    var vmTokens = tokens.map(function(t) {
      var entry = self.dictionary[t.toLowerCase()];
      return entry ? entry.vm : t.toUpperCase();
    });
    var bytes = tokens.map(function(t) {
      var entry = self.dictionary[t.toLowerCase()];
      return entry ? entry.bytes : '0x00';
    });
    return {
      input: fontanaOutput,
      lispExpr: lispExpr,
      vmProgram: vmTokens.join(' '),
      bytes: bytes,
      tokenCount: tokens.length,
      matched: tokens.filter(function(t) { return self.dictionary[t.toLowerCase()]; }).length
    };
  },

  render: function(r) {
    return [
      '╔══════════════════════════════════════════════════════════╗',
      '║  FONTANA → LISP DECODER                                 ║',
      '╠══════════════════════════════════════════════════════════╣',
      '║  INPUT:   ' + r.input.padEnd(47) + '║',
      '║  LISP:    ' + r.lispExpr.substring(0, 47).padEnd(47) + '║',
      '║  VM:      ' + r.vmProgram.substring(0, 47).padEnd(47) + '║',
      '║  BYTES:   ' + r.bytes.join(' ').substring(0, 47).padEnd(47) + '║',
      '║  TOKENS:  ' + String(r.tokenCount).padEnd(4) + ' MATCHED: ' + String(r.matched).padEnd(35) + '║',
      '╚══════════════════════════════════════════════════════════╝'
    ].join('\n');
  }
};
