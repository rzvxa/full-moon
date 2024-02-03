use crate::tokenizer::TokenReference;
use std::{borrow::Borrow, fmt::Display};

use crate::ast::punctuated::Punctuated;
use std::fmt::Write;

// Check if the vector is empty or full of None's
#[cfg(any(feature = "lua54", feature = "luau"))]
#[allow(clippy::ptr_arg)]
pub fn empty_optional_vector<T>(vec: &Vec<Option<T>>) -> bool {
    vec.iter().all(Option::is_none)
}


#[cfg(feature = "luau")]
pub fn join_type_specifiers<I: IntoIterator<Item = Option<T2>>, T1: Display, T2: Display>(
    parameters: &Punctuated<T1>,
    type_specifiers: I,
) -> String {
    let mut string = String::new();

    for (parameter, type_specifier) in parameters.pairs().zip(
        type_specifiers
            .into_iter()
            .chain(std::iter::repeat_with(|| None)),
    ) {
        let _ = write!(
            string,
            "{}{}{}",
            parameter.value(),
            display_option(type_specifier),
            display_option(parameter.punctuation())
        );
    }

    string
}


#[doc(hidden)]
#[macro_export]
macro_rules! has_version {
    ($lua_state:expr, ) => {
        true
    };

    ($lua_version:expr, $($version:ident,)+) => {{
        paste::paste! {
            let mut version_passes = false;

            $(
                #[cfg(feature = "" $version)]
                if $lua_version.[<has_ $version>]() {
                    version_passes = true;
                }
            )+

            version_passes
        }}
    };
}
