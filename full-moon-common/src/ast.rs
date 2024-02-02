use crate::{symbols::AnySymbol, tokenizer::TokenReference};
use full_moon_derive::{Node, Visit};
use serde::{Deserialize, Serialize};
use derive_more::Display;

/// An abstract syntax tree, contains all the nodes used in the code
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Ast<S: AnySymbol> {
    pub(crate) nodes: Block,
    pub(crate) eof: TokenReference<S>,
}

impl<S: AnySymbol> Ast<S> {
    /// Returns a new Ast with the given nodes
    pub fn with_nodes(self, nodes: Block) -> Self {
        Self { nodes, ..self }
    }

    /// Returns a new Ast with the given EOF token
    pub fn with_eof(self, eof: TokenReference<S>) -> Self {
        Self { eof, ..self }
    }

    /// The entire code of the function
    ///
    /// ```rust
    /// # fn main() -> Result<(), Vec<full_moon::Error>> {
    /// assert_eq!(full_moon::parse("local x = 1; local y = 2")?.nodes().stmts().count(), 2);
    /// # Ok(())
    /// # }
    /// ```
    pub fn nodes(&self) -> &Block {
        &self.nodes
    }

    /// The entire code of the function, but mutable
    pub fn nodes_mut(&mut self) -> &mut Block {
        &mut self.nodes
    }

    /// The EOF token at the end of every Ast
    pub fn eof(&self) -> &TokenReference<S> {
        &self.eof
    }
}

/// A block of statements, such as in if/do/etc block
#[derive(Clone, Debug, Default, Display, PartialEq, Node, Visit)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[display(
    fmt = "{}{}",
    "display_optional_punctuated_vec(stmts)",
    "display_option(&last_stmt.as_ref().map(display_optional_punctuated))"
)]
pub struct Block {
    stmts: Vec<(Stmt, Option<TokenReference>)>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    last_stmt: Option<(LastStmt, Option<TokenReference>)>,
}

impl Block {
    /// Creates an empty block
    pub fn new() -> Self {
        Self {
            stmts: Vec::new(),
            last_stmt: None,
        }
    }

    /// An iterator over the statements in the block, such as `local foo = 1`.
    ///
    /// Note that this does not contain the final statement which can be
    /// attained via [`Block::last_stmt`].
    pub fn stmts(&self) -> impl Iterator<Item = &Stmt> {
        self.stmts.iter().map(|(stmt, _)| stmt)
    }

    /// An iterator over the statements in the block, including any optional
    /// semicolon token reference present
    pub fn stmts_with_semicolon(&self) -> impl Iterator<Item = &(Stmt, Option<TokenReference>)> {
        self.stmts.iter()
    }

    /// The last statement of the block if one exists, such as `return foo`
    pub fn last_stmt(&self) -> Option<&LastStmt> {
        Some(&self.last_stmt.as_ref()?.0)
    }

    /// The last statement of the block if on exists, including any optional semicolon token reference present
    pub fn last_stmt_with_semicolon(&self) -> Option<&(LastStmt, Option<TokenReference>)> {
        self.last_stmt.as_ref()
    }

    /// Returns a new block with the given statements
    /// Takes a vector of statements, followed by an optional semicolon token reference
    pub fn with_stmts(self, stmts: Vec<(Stmt, Option<TokenReference>)>) -> Self {
        Self { stmts, ..self }
    }

    /// Returns a new block with the given last statement, if one is given
    /// Takes an optional last statement, with an optional semicolon
    pub fn with_last_stmt(self, last_stmt: Option<(LastStmt, Option<TokenReference>)>) -> Self {
        Self { last_stmt, ..self }
    }

    pub(crate) fn merge_blocks(&mut self, other: Self) {
        self.stmts.extend(other.stmts);

        if self.last_stmt.is_none() {
            self.last_stmt = other.last_stmt;
        }
    }
}
