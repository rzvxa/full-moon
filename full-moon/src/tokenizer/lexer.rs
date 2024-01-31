use crate::{version_switch, ShortString};

use full_moon_common::{ lexer::{Lexer, LexerResult}, tokenizer::{
    Position, Symbol, Token, TokenReference, TokenType, TokenizerError, TokenizerErrorType,
} };


fn is_identifier_start(character: char) -> bool {
    matches!(character, 'a'..='z' | 'A'..='Z' | '_')
}


enum MultiLineBodyResult {
    Ok { blocks: usize, body: String },
    NotMultiLine { blocks: usize },
    Unclosed { blocks: usize, body: String },
}
