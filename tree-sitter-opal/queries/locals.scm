; Scopes
(function_definition) @scope
(class_definition) @scope
(module_definition) @scope
(actor_definition) @scope
(for_loop) @scope
(while_loop) @scope
(block_closure) @scope
(if_expression) @scope

; Definitions
(assignment name: (identifier) @definition.var)
(let_binding name: (identifier) @definition.var)
(parameter name: (identifier) @definition.parameter)
(function_definition name: (identifier) @definition.function)

; References
(identifier) @reference
