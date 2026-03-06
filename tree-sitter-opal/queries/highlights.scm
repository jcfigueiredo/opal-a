; Keywords
[
  "def" "end" "class" "module" "protocol" "enum" "model"
  "if" "elsif" "else" "then"
  "for" "while" "in" "do"
  "match" "case"
  "return" "break" "next"
  "try" "catch" "ensure" "raise"
  "let" "needs" "requires"
  "import" "from" "export" "as"
  "actor" "receive" "reply" "send" "await"
  "macro" "ast" "emit" "event" "on"
  "type" "implements" "with" "where" "defaults"
  "and" "or" "not" "is"
  "extern" "parallel" "async"
  "self"
  "Self"
  "super"
] @keyword

["true" "false"] @boolean
"null" @constant.builtin

; Functions
(function_definition name: (identifier) @function.definition)
(call function: (identifier) @function.call)
(call function: (member_access field: (identifier) @function.method))

; Types
(class_definition name: (identifier) @type.definition)
(protocol_definition name: (identifier) @type.definition)
(module_definition name: (identifier) @type.definition)
(enum_definition name: (identifier) @type.definition)
(model_definition name: (identifier) @type.definition)
(actor_definition name: (identifier) @type.definition)
(event_definition name: (identifier) @type.definition)
(type_annotation (identifier) @type)
(implements_clause (identifier) @type)

; Variables
(assignment name: (identifier) @variable)
(let_binding name: (identifier) @variable)
(parameter name: (identifier) @variable.parameter)
(needs_declaration name: (identifier) @variable.parameter)

; Literals
(integer) @number
(float) @number.float
(string) @string
(fstring) @string
(fstring_content) @string
(interpolation) @punctuation.special
(symbol) @string.special.symbol

; Instance variables
(instance_variable) @variable.member

; Comments
(comment) @comment
(multiline_comment) @comment

; Operators
[
  "+" "-" "*" "/" "%" "**"
  "==" "!=" "<" "<=" ">" ">="
  "+=" "-=" "*=" "/="
  "|>" ".." "..." "?." "??"
  "=" "->" "|"
] @operator

; Punctuation
["(" ")" "[" "]" "{" "}"] @punctuation.bracket
["," ":" "."] @punctuation.delimiter

; Macros and annotations
(macro_invocation "@" @attribute name: (identifier) @attribute)
(annotation "@[" @attribute (identifier) @attribute "]" @attribute)
