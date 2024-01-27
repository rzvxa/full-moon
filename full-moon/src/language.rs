use crate::tokenizer::{Lexer, SuperLexer};

pub trait Language {
    type Lex: Lexer;
}

pub struct SuperLua {
    lexer: SuperLexer,
}

impl Language for SuperLua {
    type Lex = SuperLexer;
}
