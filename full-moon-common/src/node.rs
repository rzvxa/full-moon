use crate::{
    ast::Ast,
    tokenizer::{Position, Token, TokenReference},
    symbols::AnySymbol,
};
use std::fmt;

/// Used to represent nodes such as tokens or function definitions
///
/// This trait is sealed and cannot be implemented for types outside of `full-moon`
pub trait Node<S: AnySymbol> {
    /// The start position of a node. None if can't be determined
    fn start_position(&self) -> Option<Position>;

    /// The end position of a node. None if it can't be determined
    fn end_position(&self) -> Option<Position>;

    /// Whether another node of the same type is the same as this one semantically, ignoring position
    fn similar(&self, other: &Self) -> bool
    where
        Self: Sized;

    /// The token references that comprise a node
    fn tokens(&self) -> Tokens<S>;

    /// The full range of a node, if it has both start and end positions
    fn range(&self) -> Option<(Position, Position)> {
        Some((self.start_position()?, self.end_position()?))
    }

    /// The tokens surrounding a node that are ignored and not accessible through the node's own accessors.
    /// Use this if you want to get surrounding comments or whitespace.
    /// Returns a tuple of the leading and trailing trivia.
    fn surrounding_trivia(&self) -> (Vec<&Token<S>>, Vec<&Token<S>>) {
        let mut tokens = self.tokens();
        let leading = tokens.next();
        let trailing = tokens.next_back();

        (
            match leading {
                Some(token) => token.leading_trivia().collect(),
                None => Vec::new(),
            },
            match trailing {
                Some(token) => token.trailing_trivia().collect(),
                None => Vec::new(),
            },
        )
    }
}

pub(crate) enum TokenItem<'a, S: AnySymbol> {
    MoreTokens(&'a dyn Node<S>),
    TokenReference(&'a TokenReference<S>),
}

impl<S: AnySymbol> fmt::Debug for TokenItem<'_, S> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenItem::MoreTokens(_) => write!(formatter, "TokenItem::MoreTokens"),
            TokenItem::TokenReference(token) => {
                write!(formatter, "TokenItem::TokenReference({token})")
            }
        }
    }
}

/// An iterator that iterates over the tokens of a node
/// Returned by [`Node::tokens`]
#[derive(Default)]
pub struct Tokens<'a, S: AnySymbol> {
    pub(crate) items: Vec<TokenItem<'a, S>>,
}

impl<'a, S: AnySymbol> Iterator for Tokens<'a, S> {
    type Item = &'a TokenReference<S>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.items.is_empty() {
            return None;
        }

        match self.items.remove(0) {
            TokenItem::TokenReference(reference) => Some(reference),
            TokenItem::MoreTokens(node) => {
                let mut tokens = node.tokens();
                tokens.items.append(&mut self.items);
                self.items = tokens.items;
                self.next()
            }
        }
    }
}

impl<'a, S: AnySymbol> DoubleEndedIterator for Tokens<'a, S> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.items.is_empty() {
            return None;
        }

        match self.items.pop()? {
            TokenItem::TokenReference(reference) => Some(reference),
            TokenItem::MoreTokens(node) => {
                let mut tokens = node.tokens();
                self.items.append(&mut tokens.items);
                self.next_back()
            }
        }
    }
}

impl<S: AnySymbol> Node<S> for Ast<S> {
    fn start_position(&self) -> Option<Position> {
        self.nodes().start_position()
    }

    fn end_position(&self) -> Option<Position> {
        self.nodes().end_position()
    }

    fn similar(&self, other: &Self) -> bool {
        self.nodes().similar(other.nodes())
    }

    fn tokens(&self) -> Tokens<S> {
        self.nodes().tokens()
    }
}

impl<S: AnySymbol, T: Node<S>> Node<S> for Box<T> {
    fn start_position(&self) -> Option<Position> {
        (**self).start_position()
    }

    fn end_position(&self) -> Option<Position> {
        (**self).end_position()
    }

    fn similar(&self, other: &Self) -> bool {
        (**self).similar(other)
    }

    fn tokens(&self) -> Tokens<S> {
        (**self).tokens()
    }
}

impl<S: AnySymbol, T: Node<S>> Node<S> for &T {
    fn start_position(&self) -> Option<Position> {
        (**self).start_position()
    }

    fn end_position(&self) -> Option<Position> {
        (**self).end_position()
    }

    fn similar(&self, other: &Self) -> bool {
        (**self).similar(other)
    }

    fn tokens(&self) -> Tokens<S> {
        (**self).tokens()
    }
}

impl<S: AnySymbol, T: Node<S>> Node<S> for &mut T {
    fn start_position(&self) -> Option<Position> {
        (**self).start_position()
    }

    fn end_position(&self) -> Option<Position> {
        (**self).end_position()
    }

    fn similar(&self, other: &Self) -> bool {
        (**self).similar(other)
    }

    fn tokens(&self) -> Tokens<S> {
        (**self).tokens()
    }
}

impl<S: AnySymbol> Node<S> for TokenReference<S> {
    fn start_position(&self) -> Option<Position> {
        Some((**self).start_position())
    }

    fn end_position(&self) -> Option<Position> {
        Some((**self).end_position())
    }

    fn similar(&self, other: &Self) -> bool {
        *self.token_type() == *other.token_type()
    }

    fn tokens(&self) -> Tokens<S> {
        Tokens {
            items: vec![TokenItem::TokenReference(self)],
        }
    }
}

impl<S: AnySymbol, T: Node<S>> Node<S> for Option<T> {
    fn start_position(&self) -> Option<Position> {
        self.as_ref().and_then(Node::start_position)
    }

    fn end_position(&self) -> Option<Position> {
        self.as_ref().and_then(Node::end_position)
    }

    fn similar(&self, other: &Self) -> bool {
        match (self.as_ref(), other.as_ref()) {
            (Some(x), Some(y)) => x.similar(y),
            (None, None) => true,
            _ => false,
        }
    }

    fn tokens(&self) -> Tokens<S> {
        match self {
            Some(node) => node.tokens(),
            None => Tokens::default(),
        }
    }
}

impl<S: AnySymbol, T: Node<S>> Node<S> for Vec<T> {
    fn start_position(&self) -> Option<Position> {
        self.first()?.start_position()
    }

    fn end_position(&self) -> Option<Position> {
        self.last()?.end_position()
    }

    fn similar(&self, other: &Self) -> bool {
        if self.len() == other.len() {
            self.iter().zip(other.iter()).all(|(x, y)| x.similar(y))
        } else {
            false
        }
    }

    fn tokens(&self) -> Tokens<S> {
        Tokens {
            items: self.iter().flat_map(|node| node.tokens().items).collect(),
        }
    }
}

impl<A: Node<S>, B: Node<S>, S: AnySymbol> Node<S> for (A, B) {
    fn start_position(&self) -> Option<Position> {
        match (self.0.start_position(), self.1.start_position()) {
            (Some(x), Some(y)) => Some(std::cmp::min(x, y)),
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None,
        }
    }

    fn end_position(&self) -> Option<Position> {
        match (self.0.end_position(), self.1.end_position()) {
            (Some(x), Some(y)) => Some(std::cmp::max(x, y)),
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None,
        }
    }

    fn similar(&self, other: &Self) -> bool {
        self.0.similar(&other.0) && self.1.similar(&other.1)
    }

    fn tokens(&self) -> Tokens<S> {
        let mut items = self.0.tokens().items;
        items.append(&mut self.1.tokens().items);

        Tokens { items }
    }
}
