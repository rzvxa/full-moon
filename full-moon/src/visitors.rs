use crate::{
    ast::{span::ContainedSpan, *},
    private::Sealed,
    tokenizer::{Token, TokenReference},
};

#[cfg(feature = "lua52")]
use crate::ast::lua52::*;
#[cfg(feature = "lua54")]
use crate::ast::lua54::*;
#[cfg(feature = "luau")]
use crate::ast::types::*;
