/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: 'opal',

  extras: $ => [
    /[\s]/,
    $.comment,
  ],

  word: $ => $.identifier,

  externals: $ => [
    $.fstring_start_double,
    $.fstring_start_single,
    $.fstring_content,
    $.fstring_end,
    $.interpolation_start,
    $.interpolation_end,
    $.multiline_comment,
  ],

  conflicts: $ => [
    [$.parameter, $._expression],
    [$.pattern, $.constructor_pattern],
    [$.pattern, $.closure_params],
    [$.list_pattern, $.or_pattern],
    [$._expression, $.catch_clause],
    [$.closure, $.suffix_if],
  ],

  rules: {
    source_file: $ => repeat($._statement),

    _statement: $ => choice(
      $.assignment,
      $.compound_assignment,
      $.let_binding,
      $.function_definition,
      $.return_statement,
      $.class_definition,
      $.protocol_definition,
      $.module_definition,
      $.enum_definition,
      $.model_definition,
      $.needs_declaration,
      $.instance_assign,
      $.for_loop,
      $.while_loop,
      $.break_statement,
      $.next_statement,
      $.actor_definition,
      $.try_catch,
      $.from_import,
      $.import_statement,
      $.export_block,
      $.macro_definition,
      $.macro_invocation,
      $.annotated_statement,
      $.event_definition,
      $.on_handler,
      $.emit_statement,
      $.type_alias,
      $.requires_statement,
      $.raise_statement,
      $.reply_statement,
      $.extern_definition,
      $.retroactive_impl,
      $.parallel_assign,
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
      $.instance_variable,
      $.member_access,
      $.match_expression,
      $.closure,
      $.block_closure_call,
      $.list,
      $.dict,
      $.list_comprehension,
      $.self,
      $.await_expression,
      $.index_expression,
      $.null_safe_access,
      $.cast_expression,
      $.fstring,
      $.suffix_if,
      $.ast_block,
      $.splice,
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

    return_type: $ => prec.left(seq(
      $.identifier,
      optional(seq('[', $.type_annotation, repeat(seq(',', $.type_annotation)), ']')),
      optional('?'),
    )),

    type_annotation: $ => prec.left(seq(
      $.identifier,
      optional(seq('[', $.type_annotation, repeat(seq(',', $.type_annotation)), ']')),
      optional('?'),
    )),

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

    // Classes, protocols, modules, enums, models
    class_definition: $ => seq(
      'class',
      field('name', $.identifier),
      optional($.implements_clause),
      repeat(choice(
        $.needs_declaration,
        $.function_definition,
      )),
      'end',
    ),

    implements_clause: $ => seq(
      'implements',
      $.identifier,
      repeat(seq(',', $.identifier)),
    ),

    needs_declaration: $ => seq(
      'needs',
      field('name', $.identifier),
      optional(seq(':', field('type', $.type_annotation))),
      optional(seq('=', field('default', $._expression))),
    ),

    protocol_definition: $ => seq(
      'protocol',
      field('name', $.identifier),
      repeat($.protocol_method),
      'end',
    ),

    protocol_method: $ => seq(
      'def',
      field('name', $.identifier),
      optional(field('params', $.parameters)),
      optional(seq('->', $.return_type)),
    ),

    module_definition: $ => seq(
      'module',
      field('name', $.identifier),
      optional(field('body', $.body)),
      'end',
    ),

    enum_definition: $ => seq(
      'enum',
      field('name', $.identifier),
      repeat($.enum_variant),
      repeat($.function_definition),
      'end',
    ),

    enum_variant: $ => seq(
      field('name', $.identifier),
      optional($.enum_fields),
    ),

    enum_fields: $ => seq(
      '(',
      $.enum_field,
      repeat(seq(',', $.enum_field)),
      ')',
    ),

    enum_field: $ => seq(
      field('name', $.identifier),
      optional(seq(':', field('type', $.type_annotation))),
    ),

    model_definition: $ => seq(
      'model',
      field('name', $.identifier),
      repeat($.needs_declaration),
      repeat($.function_definition),
      'end',
    ),

    instance_assign: $ => seq(
      $.instance_variable,
      '=',
      field('value', $._expression),
    ),

    instance_variable: $ => seq('.', $.identifier),

    member_access: $ => prec(3, seq(
      field('object', $._expression),
      '.',
      field('field', $.identifier),
    )),

    // Loops
    for_loop: $ => seq(
      'for',
      field('var', choice($.identifier, $.destructure_pattern)),
      'in',
      field('iterable', $._expression),
      field('body', $.body),
      'end',
    ),

    while_loop: $ => seq(
      'while',
      field('condition', $._expression),
      field('body', $.body),
      'end',
    ),

    break_statement: $ => 'break',
    next_statement: $ => 'next',

    destructure_pattern: $ => seq(
      '[',
      $.identifier,
      repeat(seq(',', $.identifier)),
      optional(seq('|', $.identifier)),
      ']',
    ),

    // Match/case
    match_expression: $ => prec(-1, seq(
      'match',
      field('subject', $._expression),
      repeat1($.match_case),
      'end',
    )),

    match_case: $ => seq(
      'case',
      field('pattern', $.pattern),
      optional(seq('if', field('guard', $._expression))),
      optional(field('body', $.body)),
    ),

    pattern: $ => choice(
      $.wildcard,
      $.symbol,
      $.integer,
      $.float,
      $.string,
      $.true,
      $.false,
      $.null,
      $.constructor_pattern,
      $.list_pattern,
      $.or_pattern,
      $.range_pattern,
      $.identifier,
    ),

    range_pattern: $ => seq($.integer, choice('..', '...'), $.integer),

    wildcard: $ => '_',

    constructor_pattern: $ => seq(
      $.identifier,
      '(',
      optional(seq($.pattern, repeat(seq(',', $.pattern)))),
      ')',
    ),

    list_pattern: $ => seq(
      '[',
      optional(seq($.pattern, repeat(seq(',', $.pattern)))),
      optional(seq('|', $.pattern)),
      ']',
    ),

    or_pattern: $ => prec.left(seq($.pattern, '|', $.pattern)),

    // Closures
    closure: $ => prec(-2, seq(
      '|',
      optional($.closure_params),
      '|',
      $._expression,
    )),

    block_closure: $ => seq(
      'do',
      optional(seq('|', optional($.closure_params), '|')),
      optional($.body),
      'end',
    ),

    block_closure_call: $ => prec(1, seq(
      $.call,
      $.block_closure,
    )),

    closure_params: $ => seq(
      $.identifier,
      repeat(seq(',', $.identifier)),
    ),

    // Collections
    list: $ => seq(
      '[',
      optional(seq($._expression, repeat(seq(',', $._expression)), optional(','))),
      ']',
    ),

    dict: $ => seq(
      '{',
      choice(
        seq(':', '}'),
        seq(
          optional(seq($.dict_entry, repeat(seq(',', $.dict_entry)), optional(','))),
          '}',
        ),
      ),
    ),

    dict_entry: $ => seq(
      field('key', $._expression),
      ':',
      field('value', $._expression),
    ),

    list_comprehension: $ => seq(
      '[',
      $._expression,
      'for',
      $.identifier,
      'in',
      $._expression,
      optional(seq('if', $._expression)),
      ']',
    ),

    self: $ => 'self',

    // Actors
    actor_definition: $ => seq(
      'actor',
      field('name', $.identifier),
      repeat(choice(
        $.needs_declaration,
        $.function_definition,
        $.receive_block,
      )),
      'end',
    ),

    receive_block: $ => seq(
      'receive',
      repeat1($.match_case),
      'end',
    ),

    reply_statement: $ => prec.right(seq('reply', $._expression)),

    // Error handling
    try_catch: $ => seq(
      'try',
      optional(field('body', $.body)),
      repeat1($.catch_clause),
      optional($.ensure_clause),
      'end',
    ),

    catch_clause: $ => choice(
      prec.dynamic(10, seq('catch', field('type', $.identifier), 'as', field('var', $.identifier), optional(field('body', $.body)))),
      prec.dynamic(9, seq('catch', field('type', $.identifier), optional(field('body', $.body)))),
      seq('catch', optional(field('body', $.body))),
    ),

    ensure_clause: $ => seq(
      'ensure',
      optional(field('body', $.body)),
    ),

    raise_statement: $ => prec.right(seq('raise', $._expression)),

    requires_statement: $ => prec.right(seq(
      'requires',
      $._expression,
      optional(seq(',', $._expression)),
    )),

    // Imports/exports
    from_import: $ => seq(
      'from',
      field('module', $.identifier),
      'import',
      $.identifier,
      repeat(seq(',', $.identifier)),
    ),

    import_statement: $ => prec.left(5, seq(
      'import',
      $.identifier,
      repeat(seq('.', $.identifier)),
      optional(choice(
        seq('as', $.identifier),
        seq('.', '{', $.identifier, repeat(seq(',', $.identifier)), '}'),
      )),
    )),

    export_block: $ => seq(
      'export',
      '{',
      $.identifier,
      repeat(seq(',', $.identifier)),
      '}',
    ),

    // Macros and annotations
    macro_definition: $ => seq(
      'macro',
      field('name', $.identifier),
      '(',
      optional(seq($.identifier, repeat(seq(',', $.identifier)))),
      ')',
      optional(field('body', $.body)),
      'end',
    ),

    macro_invocation: $ => seq(
      '@',
      field('name', $.identifier),
    ),

    annotated_statement: $ => seq(
      $.annotation,
      $._statement,
    ),

    annotation: $ => seq(
      '@[',
      $.identifier,
      optional(seq(':', $._expression)),
      repeat(seq(',', $.identifier, optional(seq(':', $._expression)))),
      ']',
    ),

    // Events
    event_definition: $ => seq(
      'event',
      field('name', $.identifier),
      '(',
      optional(seq($.enum_field, repeat(seq(',', $.enum_field)))),
      ')',
    ),

    on_handler: $ => seq(
      'on',
      field('event', $.identifier),
      'do',
      '|',
      field('param', $.identifier),
      '|',
      optional(field('body', $.body)),
      'end',
    ),

    emit_statement: $ => prec.right(seq('emit', $._expression)),

    // Type aliases
    type_alias: $ => seq(
      'type',
      field('name', $.identifier),
      '=',
      field('type', $.type_expression),
    ),

    type_expression: $ => prec.left(1, choice(
      $.identifier,
      $.symbol,
      seq($.type_expression, '|', $.type_expression),
    )),

    // Extern and retroactive conformance
    extern_definition: $ => seq(
      'extern',
      $.string,
      repeat($.extern_declaration),
      'end',
    ),

    extern_declaration: $ => seq(
      'def',
      $.identifier,
      optional($.parameters),
      optional(seq('->', $.type_annotation)),
    ),

    retroactive_impl: $ => seq(
      'implements',
      field('protocol', $.identifier),
      'for',
      field('type', $.identifier),
      repeat($.function_definition),
      'end',
    ),

    // Additional expressions
    await_expression: $ => prec(12, seq('await', $._expression)),

    index_expression: $ => prec(3, seq(
      $._expression,
      '[',
      $._expression,
      ']',
    )),

    null_safe_access: $ => prec(3, seq(
      $._expression,
      '?.',
      $.identifier,
    )),

    cast_expression: $ => prec(1, seq(
      $._expression,
      'as',
      $.identifier,
    )),

    // Parallel assignment
    parallel_assign: $ => seq(
      $.identifier,
      repeat1(seq(',', $.identifier)),
      '=',
      $._expression,
      repeat(seq(',', $._expression)),
    ),

    // Suffix if (inline conditional)
    suffix_if: $ => prec.right(-2, seq(
      $._expression,
      'if',
      $._expression,
    )),

    // AST and splice (metaprogramming)
    ast_block: $ => seq(
      'ast',
      optional($.body),
      'end',
    ),

    splice: $ => seq('$', $.identifier),

    // F-strings
    fstring: $ => choice(
      seq(
        $.fstring_start_double,
        repeat(choice($.fstring_content, $.interpolation)),
        $.fstring_end,
      ),
      seq(
        $.fstring_start_single,
        repeat(choice($.fstring_content, $.interpolation)),
        $.fstring_end,
      ),
    ),

    interpolation: $ => seq(
      $.interpolation_start,
      $._expression,
      $.interpolation_end,
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

    comment: $ => choice(
      $.multiline_comment,
      token(choice(
        seq('#', /[^#\n][^\n]*/),
        seq('#', /\n/),
      )),
    ),

    _terminator: $ => '\n',
  },
});
