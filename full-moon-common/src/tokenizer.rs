use crate::short_string::ShortString;
use std::{
    cmp::Ordering,
    fmt::{self, Display},
};
use crate::visitors::{Visit, VisitMut, Visitor, VisitorMut};
use serde::{Serialize, Deserialize};


/// Used to represent exact positions of tokens in code
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Position {
    pub(crate) bytes: usize,
    pub(crate) line: usize,
    pub(crate) character: usize,
}

impl Position {
    /// How many bytes, ignoring lines, it would take to find this position
    pub fn bytes(self) -> usize {
        self.bytes
    }

    /// Index of the character on the line for this position
    pub fn character(self) -> usize {
        self.character
    }

    /// Line the position lies on
    pub fn line(self) -> usize {
        self.line
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        self.bytes.cmp(&other.bytes)
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// The type of tokens in parsed code
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
#[non_exhaustive]
pub enum TokenType<S> {
    /// End of file, should always be the very last token
    Eof,

    /// An identifier, such as `foo`
    Identifier {
        /// The identifier itself
        identifier: ShortString,
    },

    /// A multi line comment in the format of `--[[ comment ]]`
    MultiLineComment {
        /// Number of equals signs, if any, for the multi line comment
        /// For example, `--[=[` would have a `blocks` value of `1`
        blocks: usize,
        /// The comment itself, ignoring opening and closing tags
        comment: ShortString,
    },

    /// A literal number, such as `3.3`
    Number {
        /// The text representing the number, includes details such as `0x`
        text: ShortString,
    },

    /// A shebang line
    Shebang {
        /// The shebang line itself
        line: ShortString,
    },

    /// A single line comment, such as `-- comment`
    SingleLineComment {
        /// The comment, ignoring initial `--`
        comment: ShortString,
    },

    /// A literal string, such as "Hello, world"
    StringLiteral {
        /// The literal itself, ignoring quotation marks
        literal: ShortString,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_usize_zero"))]
        /// Number of equals signs used for a multi line string, if it is one
        /// For example, `[=[string]=]` would have a `multi_line_depth` value of 1
        /// `[[string]]` would have a `multi_line_depth` value of 0
        /// A string such as `"string"` would have also have a `multi_line_depth` value of 0
        multi_line_depth: usize,
        /// The type of quotation mark used to make the string
        quote_type: StringLiteralQuoteType,
    },

    /// A [`Symbol`], such as `local` or `+`
    Symbol {
        /// The symbol itself
        symbol: S,
    },

    /// Whitespace, such as tabs or new lines
    Whitespace {
        /// Characters consisting of the whitespace
        characters: ShortString,
    },

    /// Some form of interpolated string
    #[cfg(feature = "luau")]
    InterpolatedString {
        /// The literal itself, ignoring backticks
        literal: ShortString,

        /// The kind of interpolated string.
        /// If it is the beginning, middle, end, or a standalone string.
        kind: InterpolatedStringKind,
    },
}

impl<S> TokenType<S> {
    /// Returns whether a token can be practically ignored in most cases
    /// Comments and whitespace will return `true`, everything else will return `false`
    pub fn is_trivia(&self) -> bool {
        matches!(
            self,
            TokenType::Shebang { .. }
                | TokenType::SingleLineComment { .. }
                | TokenType::MultiLineComment { .. }
                | TokenType::Whitespace { .. }
        )
    }

    /// Returns the kind of the token type.
    ///
    /// ```rust
    /// use full_moon::{ShortString, tokenizer::{TokenKind, TokenType}};
    ///
    /// assert_eq!(
    ///     TokenType::Identifier {
    ///         identifier: ShortString::new("hello")
    ///     }.kind(),
    ///     TokenKind::Identifier,
    /// );
    /// ```
    pub fn kind(&self) -> TokenKind {
        match self {
            TokenType::Eof => TokenKind::Eof,
            TokenType::Identifier { .. } => TokenKind::Identifier,
            TokenType::MultiLineComment { .. } => TokenKind::MultiLineComment,
            TokenType::Number { .. } => TokenKind::Number,
            TokenType::Shebang { .. } => TokenKind::Shebang,
            TokenType::SingleLineComment { .. } => TokenKind::SingleLineComment,
            TokenType::StringLiteral { .. } => TokenKind::StringLiteral,
            TokenType::Symbol { .. } => TokenKind::Symbol,
            TokenType::Whitespace { .. } => TokenKind::Whitespace,

            #[cfg(feature = "luau")]
            TokenType::InterpolatedString { .. } => TokenKind::InterpolatedString,
        }
    }

    /// Returns a whitespace `TokenType` consisting of spaces
    pub fn spaces(spaces: usize) -> Self {
        TokenType::Whitespace {
            characters: " ".repeat(spaces).into(),
        }
    }

    /// Returns a whitespace `TokenType` consisting of tabs
    pub fn tabs(tabs: usize) -> Self {
        TokenType::Whitespace {
            characters: "\t".repeat(tabs).into(),
        }
    }
}




/// A token such consisting of its [`Position`] and a [`TokenType`]
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Token<S> {
    pub(crate) start_position: Position,
    pub(crate) end_position: Position,
    pub(crate) token_type: TokenType<S>,
}

impl<S> Token<S> {
    /// Creates a token with a zero position
    pub fn new(token_type: TokenType<S>) -> Self {
        Self {
            start_position: Position::default(),
            end_position: Position::default(),
            token_type,
        }
    }

    /// The position a token begins at
    pub fn start_position(&self) -> Position {
        self.start_position
    }

    /// The position a token ends at
    pub fn end_position(&self) -> Position {
        self.end_position
    }

    /// The type of token as well as the data needed to represent it
    /// If you don't need any other information, use [`token_kind`](Token::token_kind) instead.
    pub fn token_type(&self) -> &TokenType<S> {
        &self.token_type
    }

    /// The kind of token with no additional data.
    /// If you need any information such as idenitfier names, use [`token_type`](Token::token_type) instead.
    pub fn token_kind(&self) -> TokenKind {
        self.token_type().kind()
    }
}

impl<S> fmt::Display for Token<S> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use self::TokenType::*;

        match self.token_type() {
            Eof => Ok(()),
            Number { text } => text.fmt(formatter),
            Identifier { identifier } => identifier.fmt(formatter),
            MultiLineComment { blocks, comment } => {
                write!(formatter, "--[{0}[{1}]{0}]", "=".repeat(*blocks), comment)
            }
            Shebang { line } => line.fmt(formatter),
            SingleLineComment { comment } => write!(formatter, "--{comment}"),
            StringLiteral {
                literal,
                multi_line_depth,
                quote_type,
            } => {
                if *quote_type == StringLiteralQuoteType::Brackets {
                    write!(
                        formatter,
                        "[{0}[{1}]{0}]",
                        "=".repeat(*multi_line_depth),
                        literal
                    )
                } else {
                    write!(formatter, "{0}{1}{0}", quote_type.to_string(), literal)
                }
            }
            Symbol { symbol } => symbol.fmt(formatter),
            Whitespace { characters } => characters.fmt(formatter),

            #[cfg(feature = "luau")]
            InterpolatedString { literal, kind } => match kind {
                InterpolatedStringKind::Begin => {
                    write!(formatter, "`{literal}{{")
                }

                InterpolatedStringKind::Middle => {
                    write!(formatter, "}}{literal}{{")
                }

                InterpolatedStringKind::End => {
                    write!(formatter, "}}{literal}`")
                }

                InterpolatedStringKind::Simple => {
                    write!(formatter, "`{literal}`")
                }
            },
        }
    }
}

impl<S> Ord for Token<S> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start_position().cmp(&other.start_position())
    }
}

impl<S> PartialOrd for Token<S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<S> Visit for Token<S> {
    fn visit<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_token(self);

        match self.token_kind() {
            TokenKind::Eof => {}
            TokenKind::Identifier => visitor.visit_identifier(self),
            TokenKind::MultiLineComment => visitor.visit_multi_line_comment(self),
            TokenKind::Number => visitor.visit_number(self),
            TokenKind::Shebang => {}
            TokenKind::SingleLineComment => visitor.visit_single_line_comment(self),
            TokenKind::StringLiteral => visitor.visit_string_literal(self),
            TokenKind::Symbol => visitor.visit_symbol(self),
            TokenKind::Whitespace => visitor.visit_whitespace(self),

            #[cfg(feature = "luau")]
            TokenKind::InterpolatedString => visitor.visit_interpolated_string_segment(self),
        }
    }
}

impl<S> VisitMut for Token<S> {
    fn visit_mut<V: VisitorMut>(self, visitor: &mut V) -> Self {
        let token = visitor.visit_token(self);

        match token.token_kind() {
            TokenKind::Eof => token,
            TokenKind::Identifier => visitor.visit_identifier(token),
            TokenKind::MultiLineComment => visitor.visit_multi_line_comment(token),
            TokenKind::Number => visitor.visit_number(token),
            TokenKind::Shebang => token,
            TokenKind::SingleLineComment => visitor.visit_single_line_comment(token),
            TokenKind::StringLiteral => visitor.visit_string_literal(token),
            TokenKind::Symbol => visitor.visit_symbol(token),
            TokenKind::Whitespace => visitor.visit_whitespace(token),

            #[cfg(feature = "luau")]
            TokenKind::InterpolatedString => visitor.visit_interpolated_string_segment(token),
        }
    }
}

/// The kind of token. Contains no additional data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TokenKind {
    /// End of file, should always be the very last token
    Eof,
    /// An identifier, such as `foo`
    Identifier,
    /// A multi line comment in the format of `--[[ comment ]]`
    MultiLineComment,
    /// A literal number, such as `3.3`
    Number,
    /// The shebang line
    Shebang,
    /// A single line comment, such as `-- comment`
    SingleLineComment,
    /// A literal string, such as "Hello, world"
    StringLiteral,
    /// A [`Symbol`], such as `local` or `+`
    Symbol,
    /// Whitespace, such as tabs or new lines
    Whitespace,

    #[cfg(feature = "luau")]
    /// Some form of interpolated string
    InterpolatedString,
}

/// A reference to a token used by Ast's.
/// Dereferences to a [`Token`]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct TokenReference<S> {
    pub(crate) leading_trivia: Vec<Token<S>>,
    pub(crate) token: Token<S>,
    pub(crate) trailing_trivia: Vec<Token<S>>,
}

impl<S> TokenReference<S> {
    /// Creates a TokenReference from leading/trailing trivia as well as the leading token
    pub fn new(
        leading_trivia: Vec<Token<S>>,
        token: Token<S>,
        trailing_trivia: Vec<Token<S>>,
    ) -> Self {
        Self {
            leading_trivia,
            token,
            trailing_trivia,
        }
    }

    // /// Returns a symbol with the leading and trailing whitespace
    // /// Only whitespace is supported
    // /// ```rust
    // /// # use full_moon::tokenizer::{Symbol, TokenReference, TokenType, TokenizerErrorType};
    // /// # fn main() -> Result<(), Box<TokenizerErrorType>> {
    // /// let symbol = TokenReference::symbol("\nreturn ")?;
    // /// assert_eq!(symbol.leading_trivia().next().unwrap().to_string(), "\n");
    // /// assert_eq!(symbol.token().token_type(), &TokenType::Symbol {
    // ///     symbol: Symbol::Return,
    // /// });
    // /// assert_eq!(symbol.trailing_trivia().next().unwrap().to_string(), " ");
    // /// assert!(TokenReference::symbol("isnt whitespace").is_err());
    // /// assert!(TokenReference::symbol(" notasymbol ").is_err());
    // /// # Ok(())
    // /// # }
    // /// ```
    // pub fn symbol(text: &str) -> Result<Self, TokenizerErrorType> {
    //     TokenReference::symbol_specific_lua_version(text, LuaVersion::new())
    // }

    pub(crate) fn basic_symbol<L: Language<S>>(text: &str) -> Self {
        TokenReference::symbol_specific_lua_version::<L>(text).unwrap()
    }

    /// Returns a symbol with the leading and trailing whitespace,
    /// much like [`TokenReference::symbol`], but only if it's valid
    /// for the given Lua version.
    #[cfg_attr(
        feature = "lua52",
        doc = r##"
        ```rust
        # use full_moon::tokenizer::{Symbol, TokenReference, TokenType, TokenizerErrorType};
        # use full_moon::LuaVersion;
        # fn main() -> Result<(), Box<TokenizerErrorType>> {
        assert!(TokenReference::symbol_specific_lua_version("goto", LuaVersion::lua51()).is_err());
        assert!(TokenReference::symbol_specific_lua_version("goto", LuaVersion::lua52()).is_ok());
        # Ok(())
        # }
    "##
    )]
    pub fn symbol_specific_lua_version<L: Language<S>>(
        text: &str,
    ) -> Result<Self, TokenizerErrorType> {
        let mut lexer = L::Lex::new_lazy(text);

        let mut leading_trivia = Vec::new();
        let symbol;

        loop {
            match lexer.process_next() {
                Some(LexerResult::Ok(
                    token @ Token {
                        token_type: TokenType::Whitespace { .. },
                        ..
                    },
                )) => {
                    leading_trivia.push(token);
                }

                Some(LexerResult::Ok(
                    token @ Token {
                        token_type: TokenType::Symbol { .. },
                        ..
                    },
                )) => {
                    symbol = token;
                    break;
                }

                Some(LexerResult::Ok(Token {
                    token_type: TokenType::Eof,
                    ..
                })) => {
                    return Err(TokenizerErrorType::InvalidSymbol(text.to_owned()));
                }

                Some(LexerResult::Ok(token)) => {
                    return Err(TokenizerErrorType::UnexpectedToken(
                        token.to_string().chars().next().unwrap(),
                    ));
                }

                Some(LexerResult::Fatal(mut errors) | LexerResult::Recovered(_, mut errors)) => {
                    return Err(errors.remove(0).error);
                }

                None => unreachable!("we shouldn't have hit eof"),
            }
        }

        let mut trailing_trivia = Vec::new();

        loop {
            match lexer.process_next() {
                Some(LexerResult::Ok(
                    token @ Token {
                        token_type: TokenType::Whitespace { .. },
                        ..
                    },
                )) => {
                    trailing_trivia.push(token);
                }

                Some(LexerResult::Ok(Token {
                    token_type: TokenType::Eof,
                    ..
                })) => {
                    break;
                }

                Some(LexerResult::Ok(token)) => {
                    return Err(TokenizerErrorType::UnexpectedToken(
                        token.to_string().chars().next().unwrap(),
                    ));
                }

                Some(LexerResult::Fatal(mut errors) | LexerResult::Recovered(_, mut errors)) => {
                    return Err(errors.remove(0).error);
                }

                None => {
                    unreachable!("we shouldn't have hit eof");
                }
            }
        }

        Ok(TokenReference {
            leading_trivia,
            token: symbol,
            trailing_trivia,
        })
    }

    /// Returns the inner token.
    pub fn token(&self) -> &Token<S> {
        &self.token
    }

    /// Returns the leading trivia
    pub fn leading_trivia(&self) -> impl Iterator<Item = &Token<S>> {
        self.leading_trivia.iter()
    }

    /// Returns the trailing trivia
    pub fn trailing_trivia(&self) -> impl Iterator<Item = &Token> {
        self.trailing_trivia.iter()
    }

    /// Creates a clone of the current TokenReference with the new inner token, preserving trivia.
    pub fn with_token(&self, token: Token) -> Self {
        Self {
            token,
            leading_trivia: self.leading_trivia.clone(),
            trailing_trivia: self.trailing_trivia.clone(),
        }
    }

    /// Checks if the token is the given symbol
    pub fn is_symbol(&self, symbol: Symbol) -> bool {
        self.token.token_type() == &TokenType::Symbol { symbol }
    }
}

impl<S> std::borrow::Borrow<Token<S>> for &TokenReference<S> {
    fn borrow(&self) -> &Token<S> {
        self
    }
}

impl<S> std::ops::Deref for TokenReference<S> {
    type Target = Token<S>;

    fn deref(&self) -> &Self::Target {
        &self.token
    }
}

impl<S> fmt::Display for TokenReference<S> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        for trivia in &self.leading_trivia {
            trivia.fmt(formatter)?;
        }

        self.token.fmt(formatter)?;

        for trivia in &self.trailing_trivia {
            trivia.fmt(formatter)?;
        }

        Ok(())
    }
}

impl PartialEq<Self> for TokenReference<S> {
    fn eq(&self, other: &Self) -> bool {
        (**self).eq(other)
            && self.leading_trivia == other.leading_trivia
            && self.trailing_trivia == other.trailing_trivia
    }
}

impl Eq for TokenReference {}

impl Ord for TokenReference {
    fn cmp(&self, other: &Self) -> Ordering {
        (**self).cmp(&**other)
    }
}

impl<S> PartialOrd for TokenReference<S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<S> Visit for TokenReference<S> {
    fn visit<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_token(self);

        if matches!(self.token().token_kind(), TokenKind::Eof) {
            visitor.visit_eof(self);
        }

        self.leading_trivia.visit(visitor);
        self.token.visit(visitor);
        self.trailing_trivia.visit(visitor);
    }
}

impl<S> VisitMut for TokenReference<S> {
    fn visit_mut<V: VisitorMut>(self, visitor: &mut V) -> Self {
        let mut token_reference = visitor.visit_token_reference(self);

        if matches!(token_reference.token().token_kind(), TokenKind::Eof) {
            token_reference = visitor.visit_eof(token_reference);
        }

        token_reference.leading_trivia = token_reference.leading_trivia.visit_mut(visitor);
        token_reference.token = token_reference.token.visit_mut(visitor);
        token_reference.trailing_trivia = token_reference.trailing_trivia.visit_mut(visitor);
        token_reference
    }
}

/// The types of quotes used in a Lua string
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[non_exhaustive]
pub enum StringLiteralQuoteType {
    /// Strings formatted \[\[with brackets\]\]
    Brackets,
    /// Strings formatted "with double quotes"
    Double,
    /// Strings formatted 'with single quotes'
    Single,
}

impl fmt::Display for StringLiteralQuoteType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            // Brackets cannot be properly displayed, as not only do they have
            // variable depth (`=`), but also they don't open the same as
            // they end, meaning this can't really be used for display purposes.
            StringLiteralQuoteType::Brackets => Err(fmt::Error),
            StringLiteralQuoteType::Double => "\"".fmt(formatter),
            StringLiteralQuoteType::Single => "'".fmt(formatter),
        }
    }
}

/// Information about an error that occurs while tokenizing
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct TokenizerError {
    /// The type of error
    pub(crate) error: TokenizerErrorType,
    /// The range of the token that caused the error
    pub(crate) range: (Position, Position),
}

impl TokenizerError {
    /// The type of error
    pub fn error(&self) -> &TokenizerErrorType {
        &self.error
    }

    /// The position of the first token that caused the error
    pub fn position(&self) -> Position {
        self.range.0
    }

    /// The range of the token that caused the error
    pub fn range(&self) -> (Position, Position) {
        self.range
    }
}

impl fmt::Display for TokenizerError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{} ({}:{} to {}:{})",
            self.error,
            self.range.0.line,
            self.range.0.character,
            self.range.1.line,
            self.range.1.character
        )
    }
}

impl std::error::Error for TokenizerError {}

/// The possible errors that can happen while tokenizing.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum TokenizerErrorType {
    /// An unclosed multi-line comment was found
    UnclosedComment,
    /// An unclosed string was found
    UnclosedString,
    /// An invalid number was found
    InvalidNumber,
    /// An unexpected token was found
    UnexpectedToken(char),
    /// Symbol passed is not valid
    /// Returned from [`TokenReference::symbol`]
    InvalidSymbol(String),
}

// Used by serde
fn is_usize_zero(input: &usize) -> bool {
    *input == 0
}
