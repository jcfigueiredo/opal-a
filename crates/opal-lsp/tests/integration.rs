//! Basic tests for LSP functionality (unit-level, not full LSP protocol).

#[test]
fn test_diagnostics_valid_code() {
    let source = "x = 42\nprint(x)\n";
    let result = opal_parser::parse(source);
    assert!(result.is_ok(), "Valid code should parse without errors");
}

#[test]
fn test_diagnostics_invalid_code() {
    let source = "def\n";
    let result = opal_parser::parse(source);
    assert!(result.is_err(), "Invalid code should produce parse error");
}

#[test]
fn test_document_symbols_function_and_class() {
    let source = r#"
def greet(name)
  print(name)
end

class Bounty
  needs title: String
end
"#;
    let program = opal_parser::parse(source).unwrap();
    assert!(
        program.statements.len() >= 2,
        "Should have at least a function and a class"
    );

    // Check we got a FuncDef and ClassDef
    let has_func = program.statements.iter().any(|s| {
        matches!(&s.kind, opal_parser::StmtKind::FuncDef { name, .. } if name == "greet")
    });
    let has_class = program.statements.iter().any(|s| {
        matches!(&s.kind, opal_parser::StmtKind::ClassDef { name, .. } if name == "Bounty")
    });
    assert!(has_func, "Should find greet function");
    assert!(has_class, "Should find Bounty class");
}

#[test]
fn test_goto_definition_finds_variable() {
    let source = "x = 42\nprint(x)\n";
    let program = opal_parser::parse(source).unwrap();

    // Check that 'x' is defined in the AST
    let has_x = program.statements.iter().any(|s| {
        matches!(&s.kind, opal_parser::StmtKind::Assign { name, .. } if name == "x")
    });
    assert!(has_x, "Should find x assignment");
}

#[test]
fn test_parse_enum_with_methods() {
    let source = r#"
enum Color
  Red
  Green
  Blue

  def display()
    print(self)
  end
end
"#;
    let program = opal_parser::parse(source).unwrap();
    let has_enum = program.statements.iter().any(|s| {
        matches!(&s.kind, opal_parser::StmtKind::EnumDef { name, variants, .. }
            if name == "Color" && variants.len() == 3)
    });
    assert!(has_enum, "Should find Color enum with 3 variants");
}
