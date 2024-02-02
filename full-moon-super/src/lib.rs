mod super_lexer;
mod symbols;
mod visitors;

use symbols::Symbol;
use full_moon_common::language::Language;
use super_lexer::SuperLexer;

struct SuperLanguage {}

pub struct SuperLua {
    lexer: SuperLexer,
}

impl Language<Symbol> for SuperLua {
    type Lex = SuperLexer;
}
