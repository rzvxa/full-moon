pub mod ast;
pub mod node;
pub mod lexer;
pub mod symbols;
pub mod tokenizer;
pub mod short_string;
pub mod visitors;
pub mod language;
pub mod util;

/// An error type that consists of both [`AstError`](ast::AstError) and [`TokenizerError`](tokenizer::TokenizerError)
/// Used by [`parse`]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Error<S: symbols::AnySymbol> {
    /// Triggered if there's an issue creating an AST, but tokenizing must have succeeded
    AstError(ast::AstError<S>),
    /// Triggered if there's an issue when tokenizing, and an AST can't be made
    TokenizerError(tokenizer::TokenizerError),
}

impl<S: symbols::AnySymbol> Error<S> {
    /// Returns a human readable error message
    pub fn error_message(&self) -> std::borrow::Cow<'static, str> {
        match self {
            Error::AstError(error) => error.error_message(),
            Error::TokenizerError(error) => error.to_string().into(),
        }
    }

    /// Returns the range of the error
    pub fn range(&self) -> (tokenizer::Position, tokenizer::Position) {
        match self {
            Error::AstError(error) => error.range(),
            Error::TokenizerError(error) => error.range(),
        }
    }
}

impl<S: symbols::AnySymbol> std::fmt::Display for Error<S> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::AstError(error) => {
                write!(formatter, "error occurred while creating ast: {error}")
            }
            Error::TokenizerError(error) => {
                write!(formatter, "error occurred while tokenizing: {error}")
            }
        }
    }
}

impl<S: symbols::AnySymbol> std::error::Error for Error<S> {}
