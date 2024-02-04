mod ast;
mod super_lexer;
mod symbols;
mod visitors;

use full_moon_common::language::Language;
use super_lexer::SuperLexer;
use symbols::Symbol;

struct SuperLanguage {}

pub struct SuperLua {
    lexer: SuperLexer,
}

impl Language<Symbol> for SuperLua {
    type Lex = SuperLexer;
}
