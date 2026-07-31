var SExprParser = {
  parse: function(input) {
    input = input.trim();
    if (!input) return null;
    return this._readAtom(input, 0).value;
  },

  _readAtom: function(input, pos) {
    while (pos < input.length && input[pos] === ' ') pos++;
    if (pos >= input.length) return { value: null, pos: pos };

    if (input[pos] === '(') {
      return this._readList(input, pos + 1);
    } else if (input[pos] === '"') {
      return this._readString(input, pos + 1);
    } else {
      return this._readSymbol(input, pos);
    }
  },

  _readList: function(input, pos) {
    var items = [];
    while (pos < input.length) {
      while (pos < input.length && input[pos] === ' ') pos++;
      if (pos >= input.length) break;
      if (input[pos] === ')') return { value: items, pos: pos + 1 };
      var result = this._readAtom(input, pos);
      items.push(result.value);
      pos = result.pos;
    }
    return { value: items, pos: pos };
  },

  _readString: function(input, pos) {
    var start = pos;
    while (pos < input.length && input[pos] !== '"') pos++;
    return { value: input.substring(start, pos), pos: pos + 1 };
  },

  _readSymbol: function(input, pos) {
    var start = pos;
    while (pos < input.length && '() '.indexOf(input[pos]) === -1) pos++;
    var raw = input.substring(start, pos);
    var num = Number(raw);
    if (!isNaN(num) && raw !== '') return { value: num, pos: pos };
    return { value: raw, pos: pos };
  },

  toString: function(expr) {
    if (expr === null) return 'nil';
    if (typeof expr === 'string') return '"' + expr + '"';
    if (typeof expr === 'number') return String(expr);
    if (Array.isArray(expr)) {
      return '(' + expr.map(this.toString.bind(this)).join(' ') + ')';
    }
    return String(expr);
  }
};
