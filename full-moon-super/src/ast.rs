use crate::symbols::Symbol;
use full_moon_common::{
    ast::{make_bin_op, ContainedSpan, Expression, Field, Punctuated, Return as ReturnTrait, UnOp as UnOpTrait, BinOp as BinOpTrait},
    tokenizer::TokenReference,
};

// #[derive(Clone, Debug, Display, PartialEq, Node, Visit)]
#[derive(Clone, Debug, Display, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[display(fmt = "{token}{returns}")]
pub struct Return<S: AnySymbol> {
    token: TokenReference<S>,
    returns: Punctuated<Expression<S>, S>,
}

impl Return {
    /// Creates a new empty Return
    /// Default return token is followed by a single space
    pub fn new() -> Self {
        Self {
            token: TokenReference::basic_symbol("return "),
            returns: Punctuated::new(),
        }
    }
}
impl ReturnTrait<Symbol> for Return {
    fn token(&self) -> &TokenReference<Symbol> {
        &self.token
    }

    fn returns(&self) -> &Punctuated<Expression<Symbol>, Symbol> {
        &self.returns
    }

    fn with_token(self, token: TokenReference<Symbol>) -> Self {
        Self { token, ..self }
    }

    fn with_returns(self, returns: Punctuated<Expression<Symbol>, Symbol>) -> Self {
        Self { returns, ..self }
    }
}

impl Default for Return {
    fn default() -> Self {
        Self::new()
    }
}

make_bin_op!(
    #[doc = "Operators that require two operands, such as X + Y or X - Y"]
    // #[visit(skip_visit_self)]
    {
        Caret = 12,

        Percent = 10,
        Slash = 10,
        Star = 10,
        [luau | lua53] DoubleSlash = 10,

        Minus = 9,
        Plus = 9,

        TwoDots = 8,
        [lua53] DoubleLessThan = 7,
        [lua53] DoubleGreaterThan = 7,

        [lua53] Ampersand = 6,

        [lua53] Tilde = 5,

        [lua53] Pipe = 4,

        GreaterThan = 3,
        GreaterThanEqual = 3,
        LessThan = 3,
        LessThanEqual = 3,
        TildeEqual = 3,
        TwoEqual = 3,

        And = 2,

        Or = 1,
    }
);

impl<S: AnySymbol> BinOpTrait<S> for BinOp {
    fn precedence(&self) -> u8 {
        BinOp::precedence_of_token(self.token()).expect("invalid token")
    }

    fn is_right_associative(&self) -> bool {
        matches!(*self, BinOp::Caret(_) | BinOp::TwoDots(_))
    }

    fn is_right_associative_token(token: &TokenReference<S>) -> bool {
        matches!(
            token.token_type(),
            TokenType::Symbol {
                symbol: Symbol::Caret
            } | TokenType::Symbol {
                symbol: Symbol::TwoDots
            }
        )
    }
}

// #[derive(Clone, Debug, Display, PartialEq, Eq, Node, Visit)]
#[derive(Clone, Debug, Display, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[allow(missing_docs)]
#[non_exhaustive]
#[display(fmt = "{}")]
pub enum UnOp {
    Minus(TokenReference<Symbol>),
    Not(TokenReference<Symbol>),
    Hash(TokenReference<Symbol>),
    #[cfg(feature = "lua53")]
    Tilde(TokenReference<Symbol>),
}

impl<S: AnySymbol> UnOpTrait<S> for UnOp {
    fn token(&self) -> &TokenReference<S> {
        match self {
            UnOp::Minus(token) | UnOp::Not(token) | UnOp::Hash(token) => token,
            #[cfg(feature = "lua53")]
            UnOp::Tilde(token) => token,
        }
    }
}
