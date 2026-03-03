mod token;

pub use token::{Span, SpannedToken, Token};

use logos::Logos;

/// Lex source code into a vector of spanned tokens, filtering out comments.
pub fn lex(source: &str) -> Result<Vec<SpannedToken>, LexError> {
    let mut tokens = Vec::new();
    let mut lexer = Token::lexer(source);

    while let Some(result) = lexer.next() {
        let span = lexer.span();
        match result {
            Ok(token) => {
                // Skip comments
                match token {
                    Token::Comment | Token::MultilineComment => continue,
                    _ => {}
                }
                tokens.push(SpannedToken {
                    token,
                    span: Span {
                        start: span.start,
                        end: span.end,
                    },
                });
            }
            Err(()) => {
                return Err(LexError {
                    span: Span {
                        start: span.start,
                        end: span.end,
                    },
                    text: source[span].to_string(),
                });
            }
        }
    }

    Ok(tokens)
}

#[derive(Debug, Clone)]
pub struct LexError {
    pub span: Span,
    pub text: String,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "unexpected character '{}' at position {}",
            self.text, self.span.start
        )
    }
}

impl std::error::Error for LexError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_hello_world() {
        let tokens = lex(r#"print("Hello, world!")"#).unwrap();
        assert_eq!(tokens[0].token, Token::Identifier);
        assert_eq!(tokens[1].token, Token::LParen);
        assert!(matches!(tokens[2].token, Token::DoubleString));
        assert_eq!(tokens[3].token, Token::RParen);
    }

    #[test]
    fn lex_assignment() {
        let tokens = lex(r#"name = "Opal""#).unwrap();
        assert_eq!(tokens[0].token, Token::Identifier);
        assert_eq!(tokens[1].token, Token::Eq);
        assert!(matches!(tokens[2].token, Token::DoubleString));
    }

    #[test]
    fn lex_fstring() {
        let tokens = lex(r#"f"Hello, {name}!""#).unwrap();
        assert!(matches!(tokens[0].token, Token::FString));
    }

    #[test]
    fn lex_integer() {
        let tokens = lex("42").unwrap();
        assert_eq!(tokens[0].token, Token::Integer(42));
    }

    #[test]
    fn lex_float() {
        let tokens = lex("1.23").unwrap();
        assert_eq!(tokens[0].token, Token::Float(1.23));
    }

    #[test]
    fn lex_keywords() {
        let tokens = lex("let def end if else").unwrap();
        assert_eq!(tokens[0].token, Token::Let);
        assert_eq!(tokens[1].token, Token::Def);
        assert_eq!(tokens[2].token, Token::End);
        assert_eq!(tokens[3].token, Token::If);
        assert_eq!(tokens[4].token, Token::Else);
    }

    #[test]
    fn lex_symbol() {
        let tokens = lex(":ok").unwrap();
        assert_eq!(tokens[0].token, Token::Symbol);
    }

    #[test]
    fn lex_newline_significant() {
        let tokens = lex("a\nb").unwrap();
        assert_eq!(tokens[0].token, Token::Identifier);
        assert_eq!(tokens[1].token, Token::Newline);
        assert_eq!(tokens[2].token, Token::Identifier);
    }

    #[test]
    fn lex_comment_skipped() {
        let tokens = lex("a # comment\nb").unwrap();
        assert_eq!(tokens[0].token, Token::Identifier);
        assert_eq!(tokens[1].token, Token::Newline);
        assert_eq!(tokens[2].token, Token::Identifier);
    }
}
