use opal_lexer::source_location;
use opal_parser::ParseError;
use tower_lsp::lsp_types::*;

pub fn parse_diagnostics(source: &str) -> (Option<opal_parser::Program>, Vec<Diagnostic>) {
    match opal_parser::parse(source) {
        Ok(program) => (Some(program), vec![]),
        Err(err) => {
            let diagnostic = parse_error_to_diagnostic(&err, source);
            (None, vec![diagnostic])
        }
    }
}

fn parse_error_to_diagnostic(err: &ParseError, source: &str) -> Diagnostic {
    let (message, range) = match err {
        ParseError::UnexpectedToken {
            expected,
            found,
            span,
        } => {
            let (line, col) = source_location(source, span.start);
            let (end_line, end_col) = source_location(source, span.end);
            (
                format!("expected {expected}, got {found:?}"),
                Range::new(
                    Position::new(line.saturating_sub(1) as u32, col.saturating_sub(1) as u32),
                    Position::new(
                        end_line.saturating_sub(1) as u32,
                        end_col.saturating_sub(1) as u32,
                    ),
                ),
            )
        }
        ParseError::UnexpectedEof { expected } => {
            let lines = source.lines().count();
            let last_col = source.lines().last().map(|l| l.len()).unwrap_or(0);
            (
                format!("unexpected end of file, expected {expected}"),
                Range::new(
                    Position::new(lines.saturating_sub(1) as u32, last_col as u32),
                    Position::new(lines.saturating_sub(1) as u32, last_col as u32),
                ),
            )
        }
        ParseError::InvalidFString { message, span } => {
            let (line, col) = source_location(source, span.start);
            let (end_line, end_col) = source_location(source, span.end);
            (
                message.clone(),
                Range::new(
                    Position::new(line.saturating_sub(1) as u32, col.saturating_sub(1) as u32),
                    Position::new(
                        end_line.saturating_sub(1) as u32,
                        end_col.saturating_sub(1) as u32,
                    ),
                ),
            )
        }
        ParseError::LexError { message, span } => {
            let (line, col) = source_location(source, span.start);
            let (end_line, end_col) = source_location(source, span.end);
            (
                message.clone(),
                Range::new(
                    Position::new(line.saturating_sub(1) as u32, col.saturating_sub(1) as u32),
                    Position::new(
                        end_line.saturating_sub(1) as u32,
                        end_col.saturating_sub(1) as u32,
                    ),
                ),
            )
        }
    };

    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some("opal".to_string()),
        message,
        ..Default::default()
    }
}
