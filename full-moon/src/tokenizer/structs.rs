use crate::visitors::{Visit, VisitMut, Visitor, VisitorMut};


use full_moon_common::{
    lexer::{Lexer, LexerResult},
    tokenizer::{
        Position, StringLiteralQuoteType, Token, TokenReference, TokenType, TokenizerError,
        TokenizerErrorType,
    },
};
use full_moon_derive::Visit;

#[cfg(feature = "luau")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Whether or not this section is the beginning, middle, end, or if this is a standalone string.
pub enum InterpolatedStringKind {
    /// `begin{
    Begin,

    /// }middle{
    Middle,

    /// }end`
    End,

    /// `simple`
    Simple,
}

impl fmt::Display for TokenizerErrorType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenizerErrorType::UnclosedComment => "unclosed comment".fmt(formatter),
            TokenizerErrorType::UnclosedString => "unclosed string".fmt(formatter),
            TokenizerErrorType::UnexpectedToken(character) => {
                write!(formatter, "unexpected character {character}")
            }
            TokenizerErrorType::InvalidNumber => "invalid number".fmt(formatter),
            TokenizerErrorType::InvalidSymbol(symbol) => {
                write!(formatter, "invalid symbol {symbol}")
            }
        }
    }
}


// #[cfg(test)]
#[cfg(feature = "rewrite todo: tokenizer tests")]
mod tests {
    use crate::tokenizer::*;
    use pretty_assertions::assert_eq;

    macro_rules! test_rule {
        ($code:expr, $result:expr) => {
            let code: &str = $code;
            let result: RawToken = $result.into();

            match result {
                Ok(token) => {
                    let tokens = tokens(code).expect("couldn't tokenize");
                    let first_token = &tokens.get(0).expect("tokenized response is empty");
                    assert_eq!(*first_token.token_type(), token);
                }

                Err(expected) => {
                    if let Err(TokenizerError { error, .. }) = tokens($code) {
                        assert_eq!(error, expected);
                    } else {
                        panic!("tokenization should fail");
                    }
                }
            };
        };
    }

    #[test]
    fn test_rule_comment() {
        test_rule!(
            "-- hello world",
            TokenType::SingleLineComment {
                comment: " hello world".into()
            }
        );

        test_rule!(
            "--[[ hello world ]]",
            TokenType::MultiLineComment {
                blocks: 0,
                comment: " hello world ".into()
            }
        );

        test_rule!(
            "--[=[ hello world ]=]",
            TokenType::MultiLineComment {
                blocks: 1,
                comment: " hello world ".into()
            }
        );
        test_rule!("--", TokenType::SingleLineComment { comment: "".into() });
    }

    #[test]
    fn test_rule_numbers() {
        test_rule!("213", TokenType::Number { text: "213".into() });

        test_rule!("1", TokenType::Number { text: "1".into() });

        test_rule!(
            "123.45",
            TokenType::Number {
                text: "123.45".into(),
            }
        );
    }

    #[test]
    #[cfg_attr(not(feature = "luau"), ignore)]
    fn test_rule_binary_literals() {
        test_rule!(
            "0b101",
            TokenType::Number {
                text: "0b101".into(),
            }
        );
    }

    #[test]
    fn test_rule_identifier() {
        test_rule!(
            "hello",
            TokenType::Identifier {
                identifier: "hello".into(),
            }
        );

        test_rule!(
            "hello world",
            TokenType::Identifier {
                identifier: "hello".into(),
            }
        );

        test_rule!(
            "hello___",
            TokenType::Identifier {
                identifier: "hello___".into(),
            }
        );
    }

    #[test]
    fn test_rule_symbols() {
        test_rule!(
            "local",
            TokenType::Symbol {
                symbol: Symbol::Local
            }
        );
    }

    #[test]
    fn test_rule_whitespace() {
        test_rule!(
            "\t  \n\t",
            TokenType::Whitespace {
                characters: "\t  \n".into(),
            }
        );

        test_rule!(
            "\thello",
            TokenType::Whitespace {
                characters: "\t".into(),
            }
        );

        test_rule!(
            "\t\t\nhello",
            TokenType::Whitespace {
                characters: "\t\t\n".into(),
            }
        );

        test_rule!(
            "\n\thello",
            TokenType::Whitespace {
                characters: "\n".into(),
            }
        );
    }

    #[test]
    fn test_rule_string_literal() {
        test_rule!(
            "\"hello\"",
            TokenType::StringLiteral {
                literal: "hello".into(),
                multi_line: None,
                quote_type: StringLiteralQuoteType::Double,
            }
        );

        test_rule!(
            "\"hello\\\nworld\"",
            TokenType::StringLiteral {
                literal: "hello\\\nworld".into(),
                multi_line: None,
                quote_type: StringLiteralQuoteType::Double,
            }
        );

        test_rule!(
            "'hello world \\'goodbye\\''",
            TokenType::StringLiteral {
                literal: "hello world \\'goodbye\\'".into(),
                multi_line: None,
                quote_type: StringLiteralQuoteType::Single,
            }
        );

        test_rule!("\"hello", TokenizerErrorType::UnclosedString);
    }

    #[test]
    #[cfg(feature = "lua52")]
    fn test_string_z_escape() {
        test_rule!(
            "'hello \\z\nworld'",
            TokenType::StringLiteral {
                literal: "hello \\z\nworld".into(),
                multi_line: None,
                quote_type: StringLiteralQuoteType::Single,
            }
        );
    }

    #[test]
    fn test_symbols_within_symbols() {
        // "index" should not return "in"
        test_rule!(
            "index",
            TokenType::Identifier {
                identifier: "index".into()
            }
        );

        // "<=" should not return "<"
        test_rule!(
            "<=",
            TokenType::Symbol {
                symbol: Symbol::LessThanEqual,
            }
        );
    }

    #[test]
    fn test_rule_shebang() {
        test_rule!(
            "#!/usr/bin/env lua\n",
            TokenType::Shebang {
                line: "#!/usr/bin/env lua\n".into()
            }
        );
        // Don't recognize with a whitespace.
        test_rule!(
            " #!/usr/bin/env lua\n",
            TokenizerErrorType::UnexpectedShebang
        );
    }

    #[test]
    fn test_rule_bom() {
        let bom = String::from_utf8(b"\xEF\xBB\xBF".to_vec()).unwrap();
        test_rule!(
            &bom,
            TokenType::Whitespace {
                characters: ShortString::new(&bom),
            }
        );
        // Don't recognize if not in the beggining.
        test_rule!(
            &format!("#!/usr/bin/env lua\n {bom}"),
            TokenizerErrorType::UnexpectedToken('\u{feff}')
        );
    }

    #[test]
    fn test_new_line_on_same_line() {
        assert_eq!(
            tokens("\n").unwrap()[0],
            Token {
                start_position: Position {
                    bytes: 0,
                    character: 1,
                    line: 1,
                },

                end_position: Position {
                    bytes: 1,
                    character: 1,
                    line: 1,
                },

                token_type: TokenType::Whitespace {
                    characters: "\n".into()
                },
            }
        );
    }

    #[cfg(feature = "luau")]
    #[test]
    fn test_string_interpolation_multi_line() {
        let tokens = tokens("`Welcome to \\\n{name}!`").unwrap();
        assert_eq!(tokens[0].to_string(), "`Welcome to \\\n{");
    }

    #[test]
    fn test_fuzzer() {
        let _ = tokens("*ա");
        let _ = tokens("̹(");
        let _ = tokens("¹;");
    }
}
