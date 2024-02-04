//! A representation of a "contained span", or a span within specific bounds.
//!
//! Examples of contained spans include:
//! - Arguments in a function call use parentheses `(...)`
//! - Indexing a table uses brackets `[...]`
//! - Creating a table uses braces `{...}`
//!
//! Contained spans don't contain the inner data, just the start and end bounds.
use crate::{
    node::{Node, Tokens},
    symbols::AnySymbol,
    tokenizer::{Position, TokenReference},
};

use full_moon_derive::Visit;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A contained span with the beginning and ending bounds.
/// Refer to the [module documentation](index.html) for more details.
// #[derive(Clone, Debug, PartialEq, Eq, Visit)]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ContainedSpan<S: AnySymbol> {
    pub(crate) tokens: (TokenReference<S>, TokenReference<S>),
}

impl<S: AnySymbol> ContainedSpan<S> {
    /// Creates a contained span from the start and end bounds
    pub fn new(start: TokenReference<S>, end: TokenReference<S>) -> Self {
        Self {
            tokens: (start, end),
        }
    }

    /// Returns the start and end bounds in a tuple as references
    pub fn tokens(&self) -> (&TokenReference<S>, &TokenReference<S>) {
        (&self.tokens.0, &self.tokens.1)
    }
}

impl<S: AnySymbol> Node<S> for ContainedSpan<S> {
    fn start_position(&self) -> Option<Position> {
        self.tokens.0.start_position()
    }

    fn end_position(&self) -> Option<Position> {
        self.tokens.1.end_position()
    }

    fn similar(&self, other: &Self) -> bool {
        self.tokens.0.similar(&other.tokens.0) && self.tokens.1.similar(&other.tokens.1)
    }

    fn tokens(&self) -> Tokens<S> {
        self.tokens.tokens()
    }
}

// impl Sealed for ContainedSpan {}
