use crate::{ast::punctuated::Punctuated, symbols::AnySymbol, tokenizer::TokenReference};
use std::{borrow::Borrow, fmt::Display};

pub fn join_vec<T: Display, V: AsRef<[T]>>(vec: V) -> String {
    let mut string = String::new();

    for item in vec.as_ref() {
        string.push_str(&item.to_string());
    }

    string
}

pub fn display_option<T: Display, O: Borrow<Option<T>>>(option: O) -> String {
    match option.borrow() {
        Some(x) => x.to_string(),
        None => "".to_string(),
    }
}

pub fn display_optional_punctuated<T: Display, S: AnySymbol>(
    pair: &(T, Option<TokenReference<S>>),
) -> String {
    format!("{}{}", pair.0, display_option(&pair.1))
}

pub fn display_optional_punctuated_vec<T: Display, S: AnySymbol>(
    vec: &[(T, Option<TokenReference<S>>)],
) -> String {
    let mut string = String::new();

    for pair in vec {
        string.push_str(&display_optional_punctuated(pair));
    }

    string
}

pub fn join_iterators<
    I1: IntoIterator<Item = Option<T2>>,
    I2: IntoIterator<Item = Option<T3>>,
    T1: Display,
    T2: Display,
    T3: Display,
    S: AnySymbol,
>(
    parameters: &Punctuated<T1, S>,
    first_iterator: I1,
    second_iterator: I2,
) -> String {
    let mut string = String::new();

    for ((name, item1), item2) in parameters
        .pairs()
        .zip(first_iterator.into_iter())
        .zip(second_iterator.into_iter())
    {
        let _ = write!(
            string,
            "{}{}{}{}",
            name.value(),
            display_option(item1),
            display_option(item2),
            display_option(name.punctuation())
        );
    }

    string
}
