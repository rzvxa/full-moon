use crate::tokenizer::{Token, TokenReference, TokenizerError, Position};

pub trait Lexer<S> {
    /// Creates a new Lexer from the given source string.
    fn new(source: &str) -> Self;

    /// Creates a new Lexer from the given source string and Lua version(s), but does not process
    /// the first token.
    fn new_lazy(source: &str) -> Self;

    /// Returns the current token.
    fn current(&self) -> Option<&LexerResult<TokenReference<S>>>;

    /// Returns the next token.
    fn peek(&self) -> Option<&LexerResult<TokenReference<S>>>;

    /// Consumes the current token and returns the next token.
    fn consume(&mut self) -> Option<LexerResult<TokenReference<S>>>;

    /// Returns a vector of all tokens left in the source string.
    fn collect(self) -> LexerResult<Vec<Token<S>>>;

    /// Processes and returns the next token in the source string, ignoring trivia.
    fn process_next(&mut self) -> Option<LexerResult<Token<S>>>;
}

/// The result of a lexer operation.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum LexerResult<T> {
    /// The lexer operation was successful.
    Ok(T),
    /// The lexer operation was unsuccessful, and could not be recovered.
    Fatal(Vec<TokenizerError>),
    /// The lexer operation was unsuccessful, but some result can be extracted.
    Recovered(T, Vec<TokenizerError>),
}

impl<T: std::fmt::Debug> LexerResult<T> {
    fn new(value: T, errors: Vec<TokenizerError>) -> Self {
        if errors.is_empty() {
            Self::Ok(value)
        } else {
            Self::Recovered(value, errors)
        }
    }

    /// Unwraps the result, panicking if it is not [`LexerResult::Ok`].
    pub fn unwrap(self) -> T {
        match self {
            Self::Ok(value) => value,
            _ => panic!("expected ok, got {self:#?}"),
        }
    }

    /// Unwraps the errors, panicking if it is [`LexerResult::Ok`].
    pub fn unwrap_errors(self) -> Vec<TokenizerError> {
        match self {
            Self::Fatal(errors) | Self::Recovered(_, errors) => errors,
            _ => panic!("expected fatal error, got {self:#?}"),
        }
    }

    /// Returns the errors, if there was any.
    pub fn errors(self) -> Vec<TokenizerError> {
        match self {
            Self::Recovered(_, errors) => errors,
            _ => Vec::new(),
        }
    }
}

pub struct LexerSource {
    source: Vec<char>,
    lexer_position: LexerPosition,
}

impl LexerSource {
    fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            lexer_position: LexerPosition::new(),
        }
    }

    pub(crate) fn current(&self) -> Option<char> {
        self.source.get(self.lexer_position.index).copied()
    }

    pub(crate) fn next(&mut self) -> Option<char> {
        let next = self.current()?;

        if next == '\n' {
            self.lexer_position.position.line += 1;
            self.lexer_position.position.character = 1;
        } else {
            self.lexer_position.position.character += 1;
        }

        self.lexer_position.position.bytes += next.len_utf8();
        self.lexer_position.index += 1;

        Some(next)
    }

    pub(crate) fn peek(&self) -> Option<char> {
        self.source.get(self.lexer_position.index + 1).copied()
    }

    pub(crate) fn consume(&mut self, character: char) -> bool {
        if self.current() == Some(character) {
            self.next();
            true
        } else {
            false
        }
    }

    pub(crate) fn position(&self) -> Position {
        self.lexer_position.position
    }
}

#[derive(Clone, Copy)]
struct LexerPosition {
    position: Position,
    index: usize,
}

impl LexerPosition {
    fn new() -> Self {
        Self {
            position: Position {
                line: 1,
                character: 1,
                bytes: 0,
            },
            index: 0,
        }
    }
}

