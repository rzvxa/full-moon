use crate::lexer::Lexer;

pub trait Language<S> {
    type Lex: Lexer<S>;
}

