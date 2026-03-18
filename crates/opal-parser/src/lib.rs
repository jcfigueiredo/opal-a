pub mod ast;
pub mod parser;

pub use ast::*;
pub use parser::{ParseError, Parser};

/// Parse source code into an AST
pub fn parse(source: &str) -> Result<Program, ParseError> {
    let tokens = opal_lexer::lex(source).map_err(|e| ParseError::LexError {
        message: e.to_string(),
        span: e.span,
    })?;
    let mut parser = Parser::new(source, tokens);
    parser.parse_program()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_lex_error_propagates() {
        // Unterminated triple-quoted string triggers a lex error
        let result = parse(r#""""no closing"#);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParseError::LexError { .. }));
    }
}
