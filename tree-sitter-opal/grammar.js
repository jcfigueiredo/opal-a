/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: 'opal',

  extras: $ => [
    /[\s]/,
    $.comment,
  ],

  word: $ => $.identifier,

  conflicts: $ => [
    [$.parameter, $._expression],
  ],

  rules: {
    source_file: $ => repeat($._statement),

    _statement: $ => choice(
      $.assignment,
      $.compound_assignment,
      $.let_binding,
      $.function_definition,
      $.return_statement,
      $.expression_statement,
    ),

    assignment: $ => seq(
      field('name', $.identifier),
      '=',
      field('value', $._expression),
    ),

    compound_assignment: $ => seq(
      field('name', $.identifier),
      field('operator', choice('+=', '-=', '*=', '/=')),
      field('value', $._expression),
    ),

    let_binding: $ => seq(
      'let',
      field('name', $.identifier),
      '=',
      field('value', $._expression),
    ),

    expression_statement: $ => $._expression,

    _expression: $ => choice(
      $.identifier,
      $.integer,
      $.float,
      $.string,
      $.true,
      $.false,
      $.null,
      $.symbol,
      $.call,
      $.binary_expression,
      $.unary_expression,
      $.grouped_expression,
      $.if_expression,
    ),

    call: $ => prec(2, seq(
      field('function', $._expression),
      '(',
      optional(seq($._argument, repeat(seq(',', $._argument)))),
      ')',
    )),

    _argument: $ => choice(
      $.named_argument,
      $._expression,
    ),

    named_argument: $ => seq(
      field('name', $.identifier),
      ':',
      field('value', $._expression),
    ),

    binary_expression: $ => {
      const table = [
        [1, 'or'],
        [2, 'and'],
        [3, choice('==', '!=')],
        [4, choice('<', '<=', '>', '>=')],
        [4, 'in'],
        [4, 'is'],
        [5, choice('+', '-')],
        [6, choice('*', '/', '%')],
        [8, '|>'],
        [9, '..'],
        [9, '...'],
        [10, '??'],
      ];

      return choice(
        ...table.map(([precedence, op]) =>
          prec.left(precedence, seq(
            field('left', $._expression),
            field('operator', op),
            field('right', $._expression),
          ))
        ),
        prec.right(7, seq(
          field('left', $._expression),
          field('operator', '**'),
          field('right', $._expression),
        )),
      );
    },

    unary_expression: $ => prec(11, choice(
      seq('-', $._expression),
      seq('not', $._expression),
    )),

    grouped_expression: $ => prec(-1, seq('(', $._expression, ')')),

    // Functions
    function_definition: $ => {
      const header = seq(
        optional(field('visibility', choice('public', 'private'))),
        'def',
        field('name', $.identifier),
      );
      return choice(
        prec.dynamic(10, seq(header, field('params', $.parameters), optional(seq('->', $.return_type)), field('body', $.body), 'end')),
        prec.dynamic(1, seq(header, optional(seq('->', $.return_type)), field('body', $.body), 'end')),
      );
    },

    parameters: $ => seq(
      '(',
      optional(seq($.parameter, repeat(seq(',', $.parameter)))),
      ')',
    ),

    parameter: $ => seq(
      field('name', $.identifier),
      optional(seq(':', field('type', $.type_annotation))),
      optional(seq('=', field('default', $._expression))),
    ),

    return_type: $ => seq(
      $.identifier,
      optional(seq('[', $.type_annotation, repeat(seq(',', $.type_annotation)), ']')),
      optional('?'),
    ),

    type_annotation: $ => seq(
      $.identifier,
      optional(seq('[', $.type_annotation, repeat(seq(',', $.type_annotation)), ']')),
      optional('?'),
    ),

    return_statement: $ => prec.right(seq('return', optional($._expression))),

    body: $ => repeat1($._statement),

    // Control flow
    if_expression: $ => prec(-1, seq(
      'if',
      field('condition', $._expression),
      optional('then'),
      optional(field('consequence', $.body)),
      repeat($.elsif_clause),
      optional($.else_clause),
      'end',
    )),

    elsif_clause: $ => seq(
      'elsif',
      field('condition', $._expression),
      optional('then'),
      optional(field('body', $.body)),
    ),

    else_clause: $ => seq(
      'else',
      optional(field('body', $.body)),
    ),

    // Literals
    identifier: $ => /[a-zA-Z_][a-zA-Z0-9_]*!?/,
    integer: $ => /[0-9][0-9_]*/,
    float: $ => /[0-9][0-9_]*\.[0-9][0-9_]*/,
    string: $ => choice(
      seq('"""', /([^"]|"[^"]|""[^"])*/, '"""'),
      seq("'''", /([^']|'[^']|''[^'])*/, "'''"),
      /"([^"\\]|\\.)*"/,
      /'([^'\\]|\\.)*'/,
    ),
    symbol: $ => /:[a-zA-Z_][a-zA-Z0-9_]*/,
    true: $ => 'true',
    false: $ => 'false',
    null: $ => 'null',

    comment: $ => token(choice(
      seq('#', /[^#\n][^\n]*/),
      seq('#', /\n/),
      seq('###', /(.|\n)*?/, '###'),
    )),

    _terminator: $ => '\n',
  },
});
