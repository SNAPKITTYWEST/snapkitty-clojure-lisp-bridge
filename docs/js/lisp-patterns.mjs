var LispPatterns = {
  agentArt: [
    '',
    '      .---.',
    '     /     \\      [LISP AGENT: AWAKE]',
    '    | () () |     [TRUST: VERIFIED]',
    '     \\  ^  /      [ADA CONTRACT: ALLOWED]',
    '      |||||       [WOZ VAULT: ONLINE]',
    ''
  ].join('\n'),

  vaultArt: [
    '',
    '    ╔══════════════════════╗',
    '    ║  WOZ VAULT (LISP)    ║',
    '    ╠══════════════════════╣',
    '    ║  Local-first memory   ║',
    '    ║  Append-only log      ║',
    '    ║  SHA-256 sealed       ║',
    '    ╚══════════════════════╝',
    ''
  ].join('\n'),

  sealArt: [
    '',
    '    ╔══════════════════════╗',
    '    ║  SEAL STATE           ║',
    '    ╠══════════════════════╣',
    '    ║  SHA-256 (Web Crypto) ║',
    '    ║  Integrity: VERIFIED  ║',
    '    ╚══════════════════════╝',
    ''
  ].join('\n'),

  render: function(pattern) {
    return this[pattern] || '';
  }
};
