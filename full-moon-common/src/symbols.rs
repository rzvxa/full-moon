#[macro_export]
macro_rules! symbol {
    {
        $(#[$symbol_meta:meta])*
        pub enum Symbol {
            $(
                $(#[$meta:meta])*
                $([$($version:ident)|+])? $name:ident => $string:literal,
            )+
        }
    } => {
        paste::paste! {
            $(#[$symbol_meta])*
            pub enum Symbol {
                $(
                    $(#[$meta])*
                    $(
                        #[cfg(any(
                            $(feature = "" $version),+
                        ))]
                    )*
                    #[serde(rename = $string)]
                    $name,
                )+
            }

            impl Symbol {
                /// Given just the symbol text (no whitespace) and the Lua version,
                /// returns the associated symbol, if it exists.
                /// If you want a TokenReference instead, consider [`TokenReference::symbol`].
                // rewrite todo: does this link?
                /// ```rust
                /// # use full_moon::{LuaVersion, tokenizer::Symbol};
                /// assert_eq!(Symbol::from_str("local", LuaVersion::lua51()), Some(Symbol::Local));
                ///
                /// # #[cfg(feature = "lua52")]
                /// assert_eq!(Symbol::from_str("goto", LuaVersion::lua52()), Some(Symbol::Goto));
                /// assert_eq!(Symbol::from_str("goto", LuaVersion::lua51()), None);
                /// ```
                #[allow(unused)] // Without any features, lua_version is unused
                pub fn from_str(symbol: &str) -> Option<Self> {
                    todo!();
                    None
                    // match symbol {
                    //     $(
                    //         $(
                    //             #[cfg(any(
                    //                 $(feature = "" $version),+
                    //             ))]
                    //         )?
                    //         $string => {
                    //             if !crate::has_version!(lua_version, $($($version,)+)?) {
                    //                 return None;
                    //             }
                    //
                    //             Some(Self::$name)
                    //         },
                    //     )+
                    //
                    //     _ => None,
                    // }
                }
            }

            impl Display for Symbol {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    match self {
                        $(
                            $(
                                #[cfg(any(
                                    $(feature = "" $version),+
                                ))]
                            )*
                            Self::$name => f.write_str($string),
                        )+
                    }
                }
            }
        }
    };
}
