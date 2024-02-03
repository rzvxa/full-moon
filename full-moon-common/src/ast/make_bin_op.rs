#[doc(hidden)]
#[macro_export]
macro_rules! make_bin_op {
    ($(#[$outer:meta])* { $(
        $([$($version:ident)|+])? $operator:ident = $precedence:expr,
    )+ }) => {
        paste::paste! {
            // #[derive(Clone, Debug, Display, PartialEq, Eq, Node, Visit)]
            #[derive(Clone, Debug, Display, PartialEq, Eq)]
            #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
            #[non_exhaustive]
            $(#[$outer])*
            #[display(fmt = "{}")]
            pub enum BinOp<S: AnySymbol> {
                $(
                    #[allow(missing_docs)]
                    $(
                        #[cfg(any(
                            $(feature = "" $version),+
                        ))]
                    )*
                    $operator(TokenReference<S>),
                )+
            }

            impl<S: AnySymbol> BinOp<S> {
                /// The precedence of the operator, from a scale of 1 to 10. The larger the number, the higher the precedence.
                pub fn precedence_of_token(token: &TokenReference<S>) -> Option<u8> {
                    match token.token_type() {
                        TokenType::Symbol { symbol } => match symbol {
                            $(
                                $(
                                    #[cfg(any(
                                        $(feature = "" $version),+
                                    ))]
                                )*
                                _ => todo!(),
                                // Symbol::$operator => Some($precedence),
                            )+
                            _ => None,
                        },

                        _ => None
                    }
                }

                /// The token associated with this operator
                pub fn token(&self) -> &TokenReference<S> {
                    match self {
                        $(
                            $(
                                #[cfg(any(
                                    $(feature = "" $version),+
                                ))]
                            )*
                            BinOp::$operator(token) => token,
                        )+
                    }
                }

                pub(crate) fn consume<L: Language<S>>(state: &mut parser_structs::ParserState<S, L>) -> Option<Self> {
                    match state.current().unwrap().token_type() {
                        TokenType::Symbol { symbol } => match symbol {
                            $(
                                $(
                                    #[cfg(any(
                                        $(feature = "" $version),+
                                    ))]
                                )*
                                _ => todo!(),
                                // Symbol::$operator => {
                                //     if !$crate::has_version!(state.lua_version(), $($($version,)+)?) {
                                //         return None;
                                //     }
                                //
                                //     Some(BinOp::$operator(state.consume().unwrap()))
                                // },
                            )+

                            _ => None,
                        },

                        _ => None,
                    }
                }
            }
        }
    };
}
