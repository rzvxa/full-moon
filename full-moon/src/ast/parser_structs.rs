use std::borrow::Cow;

use crate::{
    tokenizer::{Lexer, LexerResult, Symbol, TokenKind, TokenReference},
    Language,
};

use super::{parsers::parse_block, Ast, Block};




/// A produced [`Ast`](crate::ast::Ast), along with any errors found during parsing.
/// This Ast may not be exactly the same as the input code, as reconstruction may have occurred.
/// For more information, read the documentation for [`parse_fallible`](crate::parse_fallible).
pub struct AstResult {
    ast: Ast,
    errors: Vec<crate::Error>,
}

impl AstResult {
    /// Returns a reference to the [`Ast`](crate::ast::Ast) that was parsed.
    /// If there were any errors, this will not be exactly the same,
    /// as reconstruction will have occurred.
    /// For more information, read the documentation for [`parse_fallible`](crate::parse_fallible).
    pub fn ast(&self) -> &Ast {
        &self.ast
    }

    /// Consumes the [`Ast`](crate::ast::Ast) that was parsed.
    /// If there were any errors, this will not be exactly the same,
    /// as reconstruction will have occurred.
    /// For more information, read the documentation for [`parse_fallible`](crate::parse_fallible).
    pub fn into_ast(self) -> Ast {
        self.ast
    }

    /// Returns all errors that occurred during parsing.
    pub fn errors(&self) -> &[crate::Error] {
        &self.errors
    }

    pub(crate) fn parse_fallible<L: Language>(code: &str) -> Self {
        const UNEXPECTED_TOKEN_ERROR: &str = "unexpected token, this needs to be a statement";

        let lexer: L::Lex = L::Lex::new(code);
        let mut parser_state = ParserState::<L>::new(lexer);

        let mut block = match parse_block(&mut parser_state) {
            ParserResult::Value(block) => block,
            _ => Block::new(),
        };

        loop {
            match parser_state.lexer.current() {
                Some(LexerResult::Ok(token)) if token.token_kind() == TokenKind::Eof => {
                    break;
                }

                Some(LexerResult::Ok(_) | LexerResult::Recovered(_, _)) => {
                    if let ParserResult::Value(new_block) = parse_block(&mut parser_state) {
                        if new_block.stmts.is_empty() {
                            if let Ok(token) = parser_state.current() {
                                if token.token_kind() == TokenKind::Eof {
                                    break;
                                }
                            }

                            match parser_state.consume() {
                                ParserResult::Value(token) => {
                                    if let Some(crate::Error::AstError(crate::ast::AstError {
                                        additional,
                                        ..
                                    })) = parser_state.errors.last()
                                    {
                                        if additional == UNEXPECTED_TOKEN_ERROR {
                                            continue;
                                        }
                                    }

                                    parser_state.token_error(token, UNEXPECTED_TOKEN_ERROR);
                                }

                                ParserResult::LexerMoved => {}

                                ParserResult::NotFound => unreachable!(),
                            }

                            continue;
                        }

                        block.merge_blocks(new_block);
                    }
                }

                Some(LexerResult::Fatal(_)) => {
                    for error in parser_state.lexer.consume().unwrap().unwrap_errors() {
                        parser_state
                            .errors
                            .push(crate::Error::TokenizerError(error));
                    }
                }

                None => break,
            }
        }

        let eof = match parser_state.lexer.consume().unwrap() {
            LexerResult::Ok(token) => token,

            LexerResult::Recovered(token, errors) => {
                for error in errors {
                    parser_state
                        .errors
                        .push(crate::Error::TokenizerError(error));
                }

                token
            }

            LexerResult::Fatal(error) => unreachable!("error: {error:?}"),
        };

        debug_assert_eq!(eof.token_kind(), TokenKind::Eof);

        Self {
            ast: Ast { nodes: block, eof },
            errors: parser_state.errors,
        }
    }

    /// Consumes this AstResult, returning the [`Ast`](crate::ast::Ast) that was parsed.
    pub fn into_result(self) -> Result<Ast, Vec<crate::Error>> {
        self.into()
    }
}

impl From<AstResult> for Result<Ast, Vec<crate::Error>> {
    fn from(ast_result: AstResult) -> Self {
        if ast_result.errors.is_empty() {
            Ok(ast_result.ast)
        } else {
            Err(ast_result.errors)
        }
    }
}
