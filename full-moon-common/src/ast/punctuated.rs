//! A punctuated sequence of syntax tree nodes separated by punctuation (tokens).
//!
//! Examples of punctuated sequences include:
//! - Arguments in a function call are `Punctuated<Expression>`
//! - Names and definitions in a local assignment are `Punctuated<TokenReference>` and `Punctuated<Expression>` respectively
//! - The values of a return statement are `Punctuated<Expression>`
//!
//! Everything with punctuation uses the [`Punctuated<T>`](Punctuated) type with the following logic.
//! ```rust
//! # use full_moon::parse;
//! # fn main() -> Result<(), Vec<full_moon::Error>> {
//! let ast = parse("call(arg1, arg2, arg3)")?;
//! //                   ^^^^^ ~~~~~ ^^^^^
//! # Ok(())
//! # }
//! ```
use crate::{
    node::{Node, TokenItem, Tokens},
    symbols::AnySymbol,
    tokenizer::{Position, TokenReference},
    util,
    visitors::{Visit, VisitMut},
};
use derive_more::Display;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{fmt::Display, iter::FromIterator};

/// A punctuated sequence of node `T` separated by
/// [`TokenReference`](crate::tokenizer::TokenReference).
/// Refer to the [module documentation](index.html) for more details.
#[derive(Clone, Debug, Display, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[display(bound = "T: Display")]
#[display(fmt = "{}", "util::join_vec(pairs)")]
pub struct Punctuated<T, S: AnySymbol> {
    pairs: Vec<Pair<T, S>>,
}

impl<T, S: AnySymbol> Punctuated<T, S> {
    /// Creates an empty punctuated sequence
    /// ```rust
    /// # use full_moon::ast::punctuated::{Pair, Punctuated};
    /// let mut punctuated: Punctuated<i32> = Punctuated::new();
    /// ```
    pub fn new() -> Self {
        Self { pairs: Vec::new() }
    }

    /// Returns whether there's any nodes in the punctuated sequence
    /// ```rust
    /// # use full_moon::ast::punctuated::{Pair, Punctuated};
    /// let mut punctuated = Punctuated::new();
    /// assert!(punctuated.is_empty());
    /// punctuated.push(Pair::new((), None));
    /// assert!(!punctuated.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of pairs in the punctuated sequence
    /// ```rust
    /// # use full_moon::ast::punctuated::{Pair, Punctuated};
    /// let mut punctuated = Punctuated::new();
    /// assert_eq!(punctuated.len(), 0);
    /// punctuated.push(Pair::new((), None));
    /// assert_eq!(punctuated.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    /// Returns an iterator over references of the sequence values, ignoring punctuation
    /// ```rust
    /// # use full_moon::ast::punctuated::{Pair, Punctuated};
    /// let mut punctuated = Punctuated::new();
    /// punctuated.push(Pair::new(1, None));
    /// let mut iterator = punctuated.iter();
    /// assert_eq!(iterator.next(), Some(&1));
    /// assert_eq!(iterator.next(), None);
    /// ```
    pub fn iter(&self) -> Iter<'_, T, S> {
        self.into_iter()
    }

    /// Returns an iterator over mutable references of the sequence values, ignoring punctuation
    /// ```rust
    /// # use full_moon::ast::punctuated::{Pair, Punctuated};
    /// let mut punctuated = Punctuated::new();
    /// punctuated.push(Pair::new(1, None));
    /// for item in punctuated.iter_mut() {
    ///     *item += 1;
    /// }
    /// assert_eq!(punctuated.pop(), Some(Pair::new(2, None)));
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, T, S> {
        self.into_iter()
    }

    /// Returns an iterator over pairs
    /// ```rust
    /// # use full_moon::ast::punctuated::{Pair, Punctuated};
    /// let mut punctuated = Punctuated::new();
    /// punctuated.push(Pair::new(1, None));
    /// let mut iterator = punctuated.into_pairs();
    /// assert_eq!(iterator.next(), Some(Pair::new(1, None)));
    /// assert_eq!(iterator.next(), None);
    /// ```
    pub fn into_pairs(self) -> impl Iterator<Item = Pair<T, S>> {
        self.pairs.into_iter()
    }

    /// Returns the first pair in the sequence
    /// ```rust
    /// # use full_moon::ast::punctuated::{Pair, Punctuated};
    /// let mut punctuated = Punctuated::new();
    /// assert_eq!(punctuated.first(), None);
    /// punctuated.push(Pair::new(1, None));
    /// assert_eq!(punctuated.first(), Some(&Pair::new(1, None)));
    /// ```
    pub fn first(&self) -> Option<&Pair<T, S>> {
        self.pairs.first()
    }

    /// Returns the last pair in the sequence
    /// ```rust
    /// # use full_moon::ast::punctuated::{Pair, Punctuated};
    /// let mut punctuated = Punctuated::new();
    /// punctuated.push(Pair::new(1, None));
    /// assert_eq!(punctuated.last(), Some(&Pair::new(1, None)));
    /// ```
    pub fn last(&self) -> Option<&Pair<T, S>> {
        self.pairs.last()
    }

    /// Returns an iterator over pairs as references
    /// ```rust
    /// # use full_moon::ast::punctuated::{Pair, Punctuated};
    /// let mut punctuated = Punctuated::new();
    /// punctuated.push(Pair::new(1, None));
    /// let mut iterator = punctuated.pairs();
    /// assert_eq!(iterator.next(), Some(&Pair::new(1, None)));
    /// assert_eq!(iterator.next(), None);
    /// ```
    pub fn pairs(&self) -> impl Iterator<Item = &Pair<T, S>> {
        self.pairs.iter()
    }

    /// Returns an iterator over pairs as mutable references
    /// ```rust
    /// # use full_moon::ast::punctuated::{Pair, Punctuated};
    /// let mut punctuated = Punctuated::new();
    /// punctuated.push(Pair::new(1, None));
    /// for item in punctuated.pairs_mut() {
    ///     *item.value_mut() += 1;
    /// }
    /// assert_eq!(punctuated.pop(), Some(Pair::new(2, None)));
    /// ```
    pub fn pairs_mut(&mut self) -> impl Iterator<Item = &mut Pair<T, S>> {
        self.pairs.iter_mut()
    }

    /// Pops off the last pair if it isn't empty
    /// ```rust
    /// # use full_moon::ast::punctuated::{Pair, Punctuated};
    /// let mut punctuated = Punctuated::new();
    /// punctuated.push(Pair::new(1, None));
    /// assert_eq!(punctuated.pop(), Some(Pair::new(1, None)));
    /// ```
    pub fn pop(&mut self) -> Option<Pair<T, S>> {
        self.pairs.pop()
    }

    /// Pushes a new pair onto the sequence
    /// ```rust
    /// # use full_moon::ast::punctuated::{Pair, Punctuated};
    /// let mut punctuated = Punctuated::new();
    /// punctuated.push(Pair::new(1, None));
    /// assert_eq!(punctuated.pop(), Some(Pair::new(1, None)));
    /// ```
    pub fn push(&mut self, pair: Pair<T, S>) {
        self.pairs.push(pair);
    }

    /// Pushes a new node `T` onto the sequence, with the given punctuation.
    /// Will apply the punctuation to the last item, which must exist.
    pub fn push_punctuated(&mut self, value: T, punctuation: TokenReference<S>) {
        let last_pair = self.pairs.pop().expect(
            "push_punctuated adds the punctuation onto the last element, but there are no elements",
        );

        if last_pair.punctuation().is_some() {
            self.pairs.push(last_pair);
            panic!("push_punctuated adds the punctuation onto the last element, but the last element already has punctuation");
        }

        self.pairs
            .push(Pair::Punctuated(last_pair.into_value(), punctuation));
        self.pairs.push(Pair::new(value, None));
    }
}

impl<T, S: AnySymbol> Default for Punctuated<T, S> {
    fn default() -> Self {
        Self::new()
    }
}

// impl<T> Sealed for Punctuated<T> {}

impl<T: Node<S>, S: AnySymbol> Node<S> for Punctuated<T, S> {
    fn start_position(&self) -> Option<Position> {
        self.pairs.first()?.start_position()
    }

    fn end_position(&self) -> Option<Position> {
        self.pairs.last()?.end_position()
    }

    fn similar(&self, other: &Self) -> bool {
        self.into_iter()
            .collect::<Vec<_>>()
            .similar(&other.into_iter().collect::<Vec<_>>())
    }

    fn tokens(&self) -> Tokens<S> {
        self.pairs.tokens()
    }
}

impl<V, T: Visit<V>, S: AnySymbol> Visit<V> for Punctuated<T, S> {
    fn visit(&self, visitor: &mut V) {
        self.pairs.visit(visitor);
    }
}

impl<V, T: VisitMut<V>, S: AnySymbol> VisitMut<V> for Punctuated<T, S> {
    fn visit_mut(self, visitor: &mut V) -> Self {
        Punctuated {
            pairs: self.pairs.visit_mut(visitor),
        }
    }
}

impl<T, S: AnySymbol> std::iter::Extend<Pair<T, S>> for Punctuated<T, S> {
    fn extend<I: IntoIterator<Item = Pair<T, S>>>(&mut self, iter: I) {
        self.pairs.extend(iter);
    }
}

impl<T, S: AnySymbol> IntoIterator for Punctuated<T, S> {
    type Item = T;
    type IntoIter = IntoIter<T, S>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.pairs.into_iter(),
        }
    }
}

impl<T, S: AnySymbol> FromIterator<Pair<T, S>> for Punctuated<T, S> {
    fn from_iter<I: IntoIterator<Item = Pair<T, S>>>(iter: I) -> Self {
        Punctuated {
            pairs: iter.into_iter().collect(),
        }
    }
}

impl<'a, T, S: AnySymbol> IntoIterator for &'a Punctuated<T, S> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T, S>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            inner: self.pairs.iter(),
        }
    }
}

impl<'a, T, S: AnySymbol> IntoIterator for &'a mut Punctuated<T, S> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T, S>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            inner: self.pairs.iter_mut(),
        }
    }
}

/// An iterator over owned values of type `T`.
/// Refer to the [module documentation](index.html) for more details.
pub struct IntoIter<T, S: AnySymbol> {
    inner: std::vec::IntoIter<Pair<T, S>>,
}

impl<T, S: AnySymbol> Iterator for IntoIter<T, S> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.inner.next()?.into_value())
    }
}

/// An iterator over borrowed values of type `&T`.
/// Refer to the [module documentation](index.html) for more details.
pub struct Iter<'a, T, S: AnySymbol> {
    inner: std::slice::Iter<'a, Pair<T, S>>,
}

impl<'a, T, S: AnySymbol> Iterator for Iter<'a, T, S> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.inner.next()?.value())
    }
}

/// An iterator over borrowed values of type `&mut T`.
/// Refer to the [module documentation](index.html) for more details.
pub struct IterMut<'a, T, S: AnySymbol> {
    inner: std::slice::IterMut<'a, Pair<T, S>>,
}

impl<'a, T, S: AnySymbol> Iterator for IterMut<'a, T, S> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.inner.next()?.value_mut())
    }
}

/// A node `T` followed by the possible trailing
/// [`TokenReference`](crate::tokenizer::TokenReference).
/// Refer to the [module documentation](index.html) for more details.
#[derive(Clone, Debug, Display, PartialEq, Eq)]
#[display(bound = "T: Display")]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Pair<T, S: AnySymbol> {
    /// A node `T` with no trailing punctuation
    #[display(fmt = "{_0}")]
    End(T),

    /// A node `T` followed by punctuation (in the form of a
    /// [`TokenReference`](crate::tokenizer::TokenReference))
    #[display(fmt = "{_0}{_1}")]
    Punctuated(T, TokenReference<S>),
}

impl<T, S: AnySymbol> Pair<T, S> {
    /// Creates a `Pair` with node `T` and optional punctuation
    /// ```rust
    /// # use full_moon::ast::punctuated::Pair;
    /// let pair = Pair::new(1, None);
    /// ```
    pub fn new(value: T, punctuation: Option<TokenReference<S>>) -> Self {
        match punctuation {
            None => Pair::End(value),
            Some(punctuation) => Pair::Punctuated(value, punctuation),
        }
    }

    /// Takes the `Pair` and returns the node `T` and the punctuation, if it exists as a tuple
    /// ```rust
    /// # use full_moon::ast::punctuated::Pair;
    /// let pair = Pair::new(1, None);
    /// assert_eq!(pair.into_tuple(), (1, None));
    /// ```
    pub fn into_tuple(self) -> (T, Option<TokenReference<S>>) {
        match self {
            Pair::End(value) => (value, None),
            Pair::Punctuated(value, punctuation) => (value, Some(punctuation)),
        }
    }

    /// Takes the `Pair` and returns the node `T`
    /// ```rust
    /// # use full_moon::ast::punctuated::Pair;
    /// let pair = Pair::new(1, None);
    /// assert_eq!(pair.into_value(), 1);
    /// ```
    pub fn into_value(self) -> T {
        self.into_tuple().0
    }

    /// Returns a reference to the node `T`
    /// ```rust
    /// # use full_moon::ast::punctuated::Pair;
    /// let pair = Pair::new(1, None);
    /// assert_eq!(pair.value(), &1);
    /// ```
    pub fn value(&self) -> &T {
        match self {
            Pair::End(value) => value,
            Pair::Punctuated(value, _) => value,
        }
    }

    /// Returns a mutable reference to the node `T`
    /// ```rust
    /// # use full_moon::ast::punctuated::Pair;
    /// let mut pair = Pair::new(1, None);
    /// *pair.value_mut() += 1;
    /// assert_eq!(pair.into_value(), 2);
    /// ```
    pub fn value_mut(&mut self) -> &mut T {
        match self {
            Pair::End(value) => value,
            Pair::Punctuated(value, _) => value,
        }
    }

    /// Returns the trailing punctuation, if it exists
    /// ```rust
    /// # use full_moon::ast::punctuated::Pair;
    /// let pair = Pair::new(1, None);
    /// assert_eq!(pair.punctuation(), None);
    /// ```
    pub fn punctuation(&self) -> Option<&TokenReference<S>> {
        match self {
            Pair::End(_) => None,
            Pair::Punctuated(_, punctuation) => Some(punctuation),
        }
    }

    /// Maps a `Pair<T>` to a `Pair<U>` by applying a function to the value of the pair,
    /// while preserving punctuation if it is not the end.
    /// ```rust
    /// # use full_moon::ast::punctuated::Pair;
    /// let pair = Pair::new(2, None);
    /// assert_eq!(*pair.map(|i| i * 2).value(), 4);
    /// ```
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Pair<U, S> {
        match self {
            Pair::End(value) => Pair::End(f(value)),
            Pair::Punctuated(value, punctuated) => Pair::Punctuated(f(value), punctuated),
        }
    }
}

// impl<T> Sealed for Pair<T> {}

impl<T: Node<S>, S: AnySymbol> Node<S> for Pair<T, S> {
    fn start_position(&self) -> Option<Position> {
        self.value().start_position()
    }

    fn end_position(&self) -> Option<Position> {
        self.punctuation()
            .and_then(Node::end_position)
            .or_else(|| self.value().end_position())
    }

    fn similar(&self, other: &Self) -> bool {
        self.value().similar(other.value())
    }

    fn tokens(&self) -> Tokens<S> {
        match self {
            Pair::Punctuated(node, separator) => {
                let mut items = node.tokens().items;
                items.push(TokenItem::TokenReference(separator));

                Tokens { items }
            }

            Pair::End(node) => node.tokens(),
        }
    }
}

impl<V, T: Visit<V>, S: AnySymbol> Visit<V> for Pair<T, S> {
    fn visit(&self, visitor: &mut V) {
        match self {
            Pair::End(value) => value.visit(visitor),
            Pair::Punctuated(value, punctuation) => {
                value.visit(visitor);
                punctuation.visit(visitor);
            }
        }
    }
}

impl<V, T: VisitMut<V>, S: AnySymbol> VisitMut<V> for Pair<T, S> {
    fn visit_mut(self, visitor: &mut V) -> Self {
        match self {
            Pair::End(value) => Pair::End(value.visit_mut(visitor)),
            Pair::Punctuated(value, punctuation) => {
                Pair::Punctuated(value.visit_mut(visitor), punctuation.visit_mut(visitor))
            }
        }
    }
}
