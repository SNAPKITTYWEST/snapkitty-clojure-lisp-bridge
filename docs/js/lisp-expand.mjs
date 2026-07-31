var LispExpand = {
  expansions: {
    '(agent trust audit)': [
      '(progn',
      '  (let ((agent-state (agent-status)))',
      '    (let ((trust-score (trust-score)))',
      '      (let ((audit-data (audit-log)))',
      '        (list agent-state trust-score audit-data)))))'
    ].join('\n'),
    '(agent status)':     '(agent-status)',
    '(trust score)':      '(trust-score)',
    '(seal state)':       '(seal-state)',
    '(vault count)':      '(woz-vault-count)',
    '(boot system)':      '(boot)',
    '(run debate topic)': '(debate topic)',
    '(export snapshot)':  '(export-snapshot)',
    '(draw agent)':       '(draw-agent)',
    '(eval expr)':        '(eval expr)'
  },

  expand: function(exprStr) {
    var key = exprStr.toLowerCase().trim();
    var expansion = this.expansions[key];
    if (expansion) {
      return {
        input: exprStr,
        expansion: expansion,
        steps: expansion.split('\n').length,
        safe: true
      };
    }
    // Try generic expansion
    var parsed = SExprParser.parse(exprStr);
    if (parsed && Array.isArray(parsed)) {
      var expanded = this._expandForm(parsed);
      return {
        input: exprStr,
        expansion: expanded,
        steps: 1,
        safe: true
      };
    }
    return {
      input: exprStr,
      expansion: exprStr,
      steps: 1,
      safe: true
    };
  },

  _expandForm: function(form) {
    if (!Array.isArray(form) || form.length === 0) return SExprParser.toString(form);
    var op = form[0];
    var args = form.slice(1);

    if (op === '+' || op === '-' || op === '*' || op === '/') {
      return '(calc ' + op + ' ' + args.map(SExprParser.toString).join(' ') + ')';
    }
    if (op === 'agent-status' || op === 'trust-score' || op === 'seal-state' || op === 'woz-vault-count' || op === 'draw-agent') {
      return '(' + op + ')';
    }
    return SExprParser.toString(form);
  },

  render: function(r) {
    return [
      '╔══════════════════════════════════════════════════════════╗',
      '║  LISP EXPAND                                            ║',
      '╠══════════════════════════════════════════════════════════╣',
      '║  INPUT:     ' + r.input.padEnd(45) + '║',
      '║  EXPANSION: ' + r.expansion.substring(0, 45).padEnd(45) + '║',
      '║  STEPS:     ' + String(r.steps).padEnd(45) + '║',
      '║  SAFE:      ' + String(r.safe).padEnd(45) + '║',
      '╚══════════════════════════════════════════════════════════╝'
    ].join('\n');
  }
};
