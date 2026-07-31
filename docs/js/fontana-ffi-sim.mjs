var FontanaFFI = {
  rules: {
    'agent':   { transforms: ['verified', 'sealed', 'audited'] },
    'trust':   { transforms: ['approved', 'granted', 'confirmed'] },
    'audit':   { transforms: ['completed', 'passed', 'logged'] },
    'seal':    { transforms: ['generated', 'stored', 'verified'] },
    'debate':  { transforms: ['resolved', 'concluded', 'settled'] },
    'lisp':    { transforms: ['evaluated', 'parsed', 'executed'] },
    'vault':   { transforms: ['accessed', 'read', 'written'] },
    'runtime': { transforms: ['initialized', 'active', 'running'] }
  },

  react: function(symbols) {
    var self = this;
    var inputList = symbols.join(' ');
    var outputTokens = symbols.map(function(s) {
      var rule = self.rules[s.toLowerCase()];
      if (rule) {
        var idx = Math.floor(Math.random() * rule.transforms.length);
        return rule.transforms[idx];
      }
      return s;
    });
    return {
      input: inputList,
      output: outputTokens.join(' '),
      inputTokens: symbols,
      outputTokens: outputTokens
    };
  },

  render: function(result) {
    return [
      '╔══════════════════════════════════════╗',
      '║  FONTANA REACTION CHAMBER            ║',
      '╠══════════════════════════════════════╣',
      '║  INPUT:  ' + result.input.padEnd(28) + '║',
      '║           ↓                          ║',
      '║  OUTPUT: ' + result.output.padEnd(28) + '║',
      '╚══════════════════════════════════════╝'
    ].join('\n');
  }
};
