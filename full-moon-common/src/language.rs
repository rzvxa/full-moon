use crate::{lexer::Lexer, symbols::AnySymbol};

pub trait Language<S: AnySymbol> {
    type Lex: Lexer<S>;
}
