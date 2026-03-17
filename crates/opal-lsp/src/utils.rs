use opal_lexer::{Span, source_location};
use tower_lsp::lsp_types::*;

pub fn span_to_range(span: Span, source: &str) -> Range {
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
