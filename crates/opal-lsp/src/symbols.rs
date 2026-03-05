use opal_lexer::{source_location, Span};
use opal_parser::{Program, Stmt, StmtKind};
use tower_lsp::lsp_types::*;

pub fn document_symbols(program: &Program, source: &str) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();
    for stmt in &program.statements {
        if let Some(sym) = stmt_to_symbol(stmt, source) {
            symbols.push(sym);
        }
    }
    symbols
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

#[allow(deprecated)]
fn stmt_to_symbol(stmt: &Stmt, source: &str) -> Option<DocumentSymbol> {
    let range = span_to_range(stmt.span, source);

    match &stmt.kind {
        StmtKind::FuncDef { name, .. } => Some(DocumentSymbol {
            name: name.clone(),
            detail: None,
            kind: SymbolKind::FUNCTION,
            tags: None,
            deprecated: None,
            range,
            selection_range: range,
            children: None,
        }),

        StmtKind::ClassDef { name, methods, .. } => {
            let children: Vec<_> = methods.iter().filter_map(|m| stmt_to_symbol(m, source)).collect();
            Some(DocumentSymbol {
                name: name.clone(),
                detail: None,
                kind: SymbolKind::CLASS,
                tags: None,
                deprecated: None,
                range,
                selection_range: range,
                children: if children.is_empty() { None } else { Some(children) },
            })
        }

        StmtKind::ModuleDef { name, body, .. } => {
            let children: Vec<_> = body.iter().filter_map(|s| stmt_to_symbol(s, source)).collect();
            Some(DocumentSymbol {
                name: name.clone(),
                detail: None,
                kind: SymbolKind::MODULE,
                tags: None,
                deprecated: None,
                range,
                selection_range: range,
                children: if children.is_empty() { None } else { Some(children) },
            })
        }

        StmtKind::ProtocolDef { name, .. } => Some(DocumentSymbol {
            name: name.clone(),
            detail: None,
            kind: SymbolKind::INTERFACE,
            tags: None,
            deprecated: None,
            range,
            selection_range: range,
            children: None,
        }),

        StmtKind::EnumDef {
            name, variants, methods, ..
        } => {
            let mut children: Vec<_> = variants
                .iter()
                .map(|v| DocumentSymbol {
                    name: v.name.clone(),
                    detail: None,
                    kind: SymbolKind::ENUM_MEMBER,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children: None,
                })
                .collect();
            children.extend(methods.iter().filter_map(|m| stmt_to_symbol(m, source)));
            Some(DocumentSymbol {
                name: name.clone(),
                detail: None,
                kind: SymbolKind::ENUM,
                tags: None,
                deprecated: None,
                range,
                selection_range: range,
                children: if children.is_empty() { None } else { Some(children) },
            })
        }

        StmtKind::ActorDef { name, methods, .. } => {
            let children: Vec<_> = methods.iter().filter_map(|m| stmt_to_symbol(m, source)).collect();
            Some(DocumentSymbol {
                name: name.clone(),
                detail: None,
                kind: SymbolKind::CLASS,
                tags: None,
                deprecated: None,
                range,
                selection_range: range,
                children: if children.is_empty() { None } else { Some(children) },
            })
        }

        StmtKind::ModelDef { name, methods, .. } => {
            let children: Vec<_> = methods.iter().filter_map(|m| stmt_to_symbol(m, source)).collect();
            Some(DocumentSymbol {
                name: name.clone(),
                detail: None,
                kind: SymbolKind::STRUCT,
                tags: None,
                deprecated: None,
                range,
                selection_range: range,
                children: if children.is_empty() { None } else { Some(children) },
            })
        }

        StmtKind::EventDef { name, .. } => Some(DocumentSymbol {
            name: name.clone(),
            detail: None,
            kind: SymbolKind::EVENT,
            tags: None,
            deprecated: None,
            range,
            selection_range: range,
            children: None,
        }),

        StmtKind::TypeAlias { name, .. } => Some(DocumentSymbol {
            name: name.clone(),
            detail: None,
            kind: SymbolKind::TYPE_PARAMETER,
            tags: None,
            deprecated: None,
            range,
            selection_range: range,
            children: None,
        }),

        _ => None,
    }
}
