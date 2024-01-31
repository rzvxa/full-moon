use full_moon_common::symbol;
use std::fmt::{self, Display};
use serde::{Deserialize, Serialize};

symbol! {
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    #[non_exhaustive]
    #[allow(missing_docs)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    /// A literal symbol, used for both words important to syntax (like while) and operators (like +)
    pub enum Symbol {
        And => "and",
        Break => "break",
        Do => "do",
        Else => "else",
        ElseIf => "elseif",
        End => "end",
        False => "false",
        For => "for",
        Function => "function",
        If => "if",
        In => "in",
        Local => "local",
        Nil => "nil",
        Not => "not",
        Or => "or",
        Repeat => "repeat",
        Return => "return",
        Then => "then",
        True => "true",
        Until => "until",
        While => "while",

        [lua52] Goto => "goto",

        [luau] PlusEqual => "+=",
        [luau] MinusEqual => "-=",
        [luau] StarEqual => "*=",
        [luau] SlashEqual => "/=",
        [luau] DoubleSlashEqual => "//=",
        [luau] PercentEqual => "%=",
        [luau] CaretEqual => "^=",
        [luau] TwoDotsEqual => "..=",

        [luau | lua53] Ampersand => "&",
        [luau] ThinArrow => "->",
        [luau | lua52] TwoColons => "::",

        Caret => "^",
        Colon => ":",
        Comma => ",",
        Dot => ".",
        TwoDots => "..",
        Ellipse => "...",
        Equal => "=",
        TwoEqual => "==",
        GreaterThan => ">",
        GreaterThanEqual => ">=",
        [lua53] DoubleGreaterThan => ">>",
        Hash => "#",
        LeftBrace => "{",
        LeftBracket => "[",
        LeftParen => "(",
        LessThan => "<",
        LessThanEqual => "<=",
        [lua53] DoubleLessThan => "<<",
        Minus => "-",
        Percent => "%",
        [luau | lua53] Pipe => "|",
        Plus => "+",
        [luau] QuestionMark => "?",
        RightBrace => "}",
        RightBracket => "]",
        RightParen => ")",
        Semicolon => ";",
        Slash => "/",
        [luau | lua53] DoubleSlash => "//",
        Star => "*",
        [lua53] Tilde => "~",
        TildeEqual => "~=",
    }
}
