/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: 'opal',

  extras: $ => [
    /[ \t\r]/,
    $.comment,
  ],

  word: $ => $.identifier,

  rules: {
    source_file: $ => repeat($._statement),

    _statement: $ => choice(
      $.expression_statement,
    ),

    expression_statement: $ => seq(
      $._expression,
      $._terminator,
    ),

    _expression: $ => choice(
      $.identifier,
      $.integer,
      $.float,
      $.string,
      $.call,
    ),

    call: $ => prec(1, seq(
      $._expression,
      '(',
      optional(seq($._expression, repeat(seq(',', $._expression)))),
      ')',
    )),

    // Literals
    identifier: $ => /[a-zA-Z_][a-zA-Z0-9_]*!?/,
    integer: $ => /[0-9][0-9_]*/,
    float: $ => /[0-9][0-9_]*\.[0-9][0-9_]*/,
    string: $ => choice(
      /"([^"\\]|\\.)*"/,
      /'([^'\\]|\\.)*'/,
    ),

    // Comments
    comment: $ => token(seq('#', /.*/)),

    // Statement terminator
    _terminator: $ => /\n/,
  },
});
