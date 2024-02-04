use std::borrow::Cow;

use crate::{
    language::Language,
    lexer::{Lexer, LexerResult},
    symbols::AnySymbol,
    tokenizer::TokenReference,
};

pub struct ParserState<S: AnySymbol, L: Language<S>> {
    errors: Vec<crate::Error<S>>,
    lexer: L::Lex,
}

impl<S: AnySymbol, L: Language<S>> ParserState<S, L> {
    pub fn new(lexer: L::Lex) -> Self {
        Self {
            errors: Vec::new(),
            lexer,
        }
    }

    pub fn current(&self) -> Result<&TokenReference<S>, ()> {
        match self.lexer.current() {
            Some(LexerResult::Ok(token) | LexerResult::Recovered(token, _)) => Ok(token),
            Some(LexerResult::Fatal(_)) => Err(()),
            None => unreachable!("current() called past EOF"),
        }
    }

    pub fn peek(&self) -> Result<&TokenReference<S>, ()> {
        match self.lexer.peek() {
            Some(LexerResult::Ok(token) | LexerResult::Recovered(token, _)) => Ok(token),
            Some(LexerResult::Fatal(_)) => Err(()),
            None => unreachable!("peek() called past EOF"),
        }
    }

    pub fn consume(&mut self) -> ParserResult<TokenReference<S>> {
        let token = self.lexer.consume();

        match token {
            Some(LexerResult::Ok(token)) => ParserResult::Value(token),

            Some(LexerResult::Recovered(token, errors)) => {
                for error in errors {
                    self.errors.push(crate::Error::TokenizerError(error));
                }

                ParserResult::Value(token)
            }

            Some(LexerResult::Fatal(errors)) => {
                for error in errors {
                    self.errors.push(crate::Error::TokenizerError(error));
                }

                ParserResult::LexerMoved
            }

            None => ParserResult::NotFound,
        }
    }

    pub fn consume_if(&mut self, symbol: S) -> Option<TokenReference<S>> {
        match self.current() {
            Ok(token) => {
                if token.is_symbol(symbol) {
                    Some(self.consume().unwrap())
                } else {
                    None
                }
            }

            Err(()) => None,
        }
    }

    pub fn require(&mut self, symbol: S, error: &'static str) -> Option<TokenReference<S>> {
        match self.current() {
            Ok(token) => {
                if token.is_symbol(symbol) {
                    Some(self.consume().unwrap())
                } else {
                    self.token_error(token.clone(), error);
                    None
                }
            }

            Err(()) => None,
        }
    }

    pub fn require_with_reference_token(
        &mut self,
        symbol: S,
        error: &'static str,
        reference_token: &TokenReference<S>,
    ) -> Option<TokenReference<S>> {
        match self.current() {
            Ok(token) => {
                if token.is_symbol(symbol) {
                    Some(self.consume().unwrap())
                } else {
                    self.token_error(reference_token.clone(), error);
                    None
                }
            }

            Err(()) => None,
        }
    }

    pub fn require_with_reference_range(
        &mut self,
        symbol: S,
        error: impl MaybeLazyString,
        start_token: &TokenReference<S>,
        end_token: &TokenReference<S>,
    ) -> Option<TokenReference<S>> {
        match self.current() {
            Ok(token) => {
                if token.is_symbol(symbol) {
                    Some(self.consume().unwrap())
                } else {
                    self.token_error_ranged(token.clone(), error.to_str(), start_token, end_token);
                    None
                }
            }

            Err(()) => None,
        }
    }

    pub fn require_with_reference_range_callback(
        &mut self,
        symbol: S,
        error: impl MaybeLazyString,
        tokens: impl FnOnce() -> (TokenReference<S>, TokenReference<S>),
    ) -> Option<TokenReference<S>> {
        match self.current() {
            Ok(token) => {
                if token.is_symbol(symbol) {
                    Some(self.consume().unwrap())
                } else {
                    let (start_token, end_token) = tokens();

                    self.token_error_ranged(
                        token.clone(),
                        error.to_str(),
                        &start_token,
                        &end_token,
                    );

                    None
                }
            }

            Err(()) => None,
        }
    }

    pub fn token_error<E: Into<Cow<'static, str>>>(
        &mut self,
        token_reference: TokenReference<S>,
        error: E,
    ) {
        self.errors
            .push(crate::Error::AstError(crate::ast::AstError {
                token: token_reference.token,
                additional: error.into(),
                range: None,
            }));
    }

    // This takes start_token and end_token as owned references because otherwise, we tend to stack an immutable over mutable borrow.
    pub fn token_error_ranged<E: Into<Cow<'static, str>>>(
        &mut self,
        token_reference: TokenReference<S>,
        error: E,
        start_token: &TokenReference<S>,
        end_token: &TokenReference<S>,
    ) {
        self.errors
            .push(crate::Error::AstError(crate::ast::AstError {
                token: token_reference.token,
                additional: error.into(),
                range: Some((start_token.start_position(), end_token.end_position())),
            }));
    }
}

pub trait MaybeLazyString {
    fn to_str(self) -> Cow<'static, str>;
}

impl MaybeLazyString for &'static str {
    fn to_str(self) -> Cow<'static, str> {
        Cow::Borrowed(self)
    }
}

impl<F: FnOnce() -> String> MaybeLazyString for F {
    fn to_str(self) -> Cow<'static, str> {
        Cow::Owned(self())
    }
}

#[derive(Debug)]
pub enum ParserResult<T> {
    // This doesn't necessarily mean that there were no errors,
    // because this can sometimes be a recovered value.
    Value(T),

    // Couldn't get any sort of value, but the lexer has moved.
    // This should always come with an error.
    LexerMoved,

    NotFound,
}

impl<T> ParserResult<T> {
    pub fn unwrap(self) -> T {
        match self {
            ParserResult::Value(value) => value,
            ParserResult::LexerMoved => panic!("unwrap() called when value was LexerMoved"),
            ParserResult::NotFound => panic!("unwrap() called when value was NotFound"),
        }
    }
}
