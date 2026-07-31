var LispMachine = {
  env: {},

  safeOps: {
    '+': function(args) { return args.reduce(function(a, b) { return a + b; }, 0); },
    '-': function(args) { return args.slice(1).reduce(function(a, b) { return a - b; }, args[0]); },
    '*': function(args) { return args.reduce(function(a, b) { return a * b; }, 1); },
    '/': function(args) { return args.slice(1).reduce(function(a, b) { return a / b; }, args[0]); },
    'quote': function(args) { return args[0]; },
    'list': function(args) { return args; },
    'car': function(args) { return Array.isArray(args[0]) ? args[0][0] : null; },
    'cdr': function(args) { return Array.isArray(args[0]) ? args[0].slice(1) : []; },
    'cons': function(args) {
      if (Array.isArray(args[1])) return [args[0]].concat(args[1]);
      return [args[0], args[1]];
    },
    'agent-status': function() {
      return 'ENKI: active (sim) — tension proposer\nSENTINEL: active (sim) — claim auditor\nAll local, no remote.';
    },
    'trust-score': function() {
      var events = WozVault.readAuditLog();
      var seals = events.filter(function(e) { return e.type && e.type.indexOf('SEAL') >= 0; });
      var score = Math.min(1, seals.length * 0.1 + 0.5);
      return 'trust: ' + Math.round(score * 100) + '%\nevents: ' + events.length + '\nseals: ' + seals.length;
    },
    'draw-agent': function() {
      return [
        '      .---.',
        '     /     \\      [LISP AGENT: AWAKE]',
        '    | () () |     [TRUST: VERIFIED]',
        '     \\  ^  /      [ADA CONTRACT: ALLOWED]',
        '      |||||       [WOZ VAULT: ONLINE]'
      ].join('\n');
    },
    'woz-vault-count': function() {
      return WozVault.readAuditLog().length;
    },
    'seal-state': function() {
      var events = WozVault.readAuditLog();
      var seals = events.filter(function(e) { return e.type && e.type.indexOf('SEAL') >= 0; });
      return 'sealed_events: ' + seals.length + '\nlast_seal: ' + (seals.length > 0 ? (seals[seals.length - 1].seal || 'none').slice(0, 16) + '...' : 'none');
    }
  },

  eval: function(expr) {
    if (expr === null || expr === undefined) return null;
    if (typeof expr === 'number') return expr;
    if (typeof expr === 'string') {
      if (this.env.hasOwnProperty(expr)) return this.env[expr];
      return expr;
    }
    if (Array.isArray(expr)) {
      if (expr.length === 0) return [];
      var op = expr[0];
      var args = expr.slice(1).map(this.eval.bind(this));

      if (typeof op === 'string' && this.safeOps[op]) {
        return this.safeOps[op](args);
      }
      return [op].concat(args);
    }
    return expr;
  }
};
