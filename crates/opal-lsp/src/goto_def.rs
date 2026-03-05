use opal_lexer::{source_location, Span};
use opal_parser::*;
use tower_lsp::lsp_types::*;

/// Find the definition location for the symbol at the given position.
pub fn goto_definition(program: &Program, source: &str, position: Position) -> Option<Range> {
    let offset = position_to_offset(source, position)?;
    let target_name = identifier_at_offset(source, offset)?;

    // Build symbol table
    let mut symbols: Vec<(String, Span)> = Vec::new();
    collect_definitions(&program.statements, &mut symbols);

    // Find the definition
    symbols
        .iter()
        .find(|(name, _)| name == &target_name)
        .map(|(_, span)| span_to_range(*span, source))
}

fn position_to_offset(source: &str, position: Position) -> Option<usize> {
    let mut line = 0u32;
    let mut col = 0u32;
    for (i, ch) in source.char_indices() {
        if line == position.line && col == position.character {
            return Some(i);
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    if line == position.line && col == position.character {
        Some(source.len())
    } else {
        None
    }
}

fn identifier_at_offset(source: &str, offset: usize) -> Option<String> {
    let bytes = source.as_bytes();
    if offset >= bytes.len() {
        return None;
    }

    let mut start = offset;
    while start > 0 && is_ident_char(bytes[start - 1]) {
        start -= 1;
    }

    let mut end = offset;
    while end < bytes.len() && is_ident_char(bytes[end]) {
        end += 1;
    }

    if start == end {
        return None;
    }

    Some(source[start..end].to_string())
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'!'
}

fn collect_definitions(stmts: &[Stmt], symbols: &mut Vec<(String, Span)>) {
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::FuncDef {
                name, body, params, ..
            } => {
                symbols.push((name.clone(), stmt.span));
                for param in params {
                    symbols.push((param.name.clone(), stmt.span));
                }
                collect_definitions(body, symbols);
            }
            StmtKind::ClassDef {
                name, methods, ..
            } => {
                symbols.push((name.clone(), stmt.span));
                collect_definitions(methods, symbols);
            }
            StmtKind::ModuleDef { name, body, .. } => {
                symbols.push((name.clone(), stmt.span));
                collect_definitions(body, symbols);
            }
            StmtKind::ProtocolDef { name, .. } => {
                symbols.push((name.clone(), stmt.span));
            }
            StmtKind::EnumDef {
                name, methods, ..
            } => {
                symbols.push((name.clone(), stmt.span));
                collect_definitions(methods, symbols);
            }
            StmtKind::ActorDef {
                name,
                methods,
                init,
                ..
            } => {
                symbols.push((name.clone(), stmt.span));
                if let Some(init_body) = init {
                    collect_definitions(init_body, symbols);
                }
                collect_definitions(methods, symbols);
            }
            StmtKind::ModelDef {
                name, methods, ..
            } => {
                symbols.push((name.clone(), stmt.span));
                collect_definitions(methods, symbols);
            }
            StmtKind::EventDef { name, .. } => {
                symbols.push((name.clone(), stmt.span));
            }
            StmtKind::TypeAlias { name, .. } => {
                symbols.push((name.clone(), stmt.span));
            }
            StmtKind::Assign { name, .. } => {
                symbols.push((name.clone(), stmt.span));
            }
            StmtKind::Let { name, .. } => {
                symbols.push((name.clone(), stmt.span));
            }
            StmtKind::MacroDef { name, .. } => {
                symbols.push((name.clone(), stmt.span));
            }
            StmtKind::For { var, body, .. } => {
                symbols.push((var.clone(), stmt.span));
                collect_definitions(body, symbols);
            }
            _ => {}
        }
    }
}

fn span_to_range(span: Span, source: &str) -> Range {
    let (start_line, start_col) = source_location(source, span.start);
    let (end_line, end_col) = source_location(source, span.end);
    Range::new(
        Position::new(
            start_line.saturating_sub(1) as u32,
            start_col.saturating_sub(1) as u32,
        ),
        Position::new(
            end_line.saturating_sub(1) as u32,
            end_col.saturating_sub(1) as u32,
        ),
    )
}
