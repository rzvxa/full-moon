pub mod make_bin_op;
pub mod parser_structs;
pub mod punctuated;
pub mod span;

pub use punctuated::Punctuated;
pub use span::ContainedSpan;

use crate::{
    language::Language,
    symbols::AnySymbol,
    tokenizer::{Position, Token, TokenReference, TokenType},
    util::{
        display_option, display_optional_punctuated, display_optional_punctuated_vec,
        join_iterators, join_vec,
    },
};

use derive_more::Display;
use full_moon_derive::{Node, Visit};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt};

/// An abstract syntax tree, contains all the nodes used in the code
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Ast<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    pub(crate) nodes: Block<S, B, U, R>,
    pub(crate) eof: TokenReference<S>,
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> Ast<S, B, U, R> {
    /// Returns a new Ast with the given nodes
    pub fn with_nodes(self, nodes: Block<S, B, U, R>) -> Self {
        Self { nodes, ..self }
    }

    /// Returns a new Ast with the given EOF token
    pub fn with_eof(self, eof: TokenReference<S>) -> Self {
        Self { eof, ..self }
    }

    /// The entire code of the function
    ///
    /// ```rust
    /// # fn main() -> Result<(), Vec<full_moon::Error>> {
    /// assert_eq!(full_moon::parse("local x = 1; local y = 2")?.nodes().stmts().count(), 2);
    /// # Ok(())
    /// # }
    /// ```
    pub fn nodes(&self) -> &Block<S, B, U, R> {
        &self.nodes
    }

    /// The entire code of the function, but mutable
    pub fn nodes_mut(&mut self) -> &mut Block<S, B, U, R> {
        &mut self.nodes
    }

    /// The EOF token at the end of every Ast
    pub fn eof(&self) -> &TokenReference<S> {
        &self.eof
    }
}

/// A block of statements, such as in if/do/etc block
// #[derive(Clone, Debug, Default, Display, PartialEq, Node, Visit)]
#[derive(Clone, Debug, Default, Display, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[display(
    fmt = "{}{}",
    "display_optional_punctuated_vec(stmts)",
    "display_option(&last_stmt.as_ref().map(display_optional_punctuated))"
)]
pub struct Block<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    stmts: Vec<(Stmt<S, B, U, R>, Option<TokenReference<S>>)>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    last_stmt: Option<(LastStmt<S, B, U, R>, Option<TokenReference<S>>)>,
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> Block<S, B, U, R> {
    /// Creates an empty block
    pub fn new() -> Self {
        Self {
            stmts: Vec::new(),
            last_stmt: None,
        }
    }

    /// An iterator over the statements in the block, such as `local foo = 1`.
    ///
    /// Note that this does not contain the final statement which can be
    /// attained via [`Block::last_stmt`].
    pub fn stmts(&self) -> impl Iterator<Item = &Stmt<S, B, U, R>> {
        self.stmts.iter().map(|(stmt, _)| stmt)
    }

    /// An iterator over the statements in the block, including any optional
    /// semicolon token reference present
    pub fn stmts_with_semicolon(
        &self,
    ) -> impl Iterator<Item = &(Stmt<S, B, U, R>, Option<TokenReference<S>>)> {
        self.stmts.iter()
    }

    /// The last statement of the block if one exists, such as `return foo`
    pub fn last_stmt(&self) -> Option<&LastStmt<S, B, U, R>> {
        Some(&self.last_stmt.as_ref()?.0)
    }

    /// The last statement of the block if on exists, including any optional semicolon token reference present
    pub fn last_stmt_with_semicolon(
        &self,
    ) -> Option<&(LastStmt<S, B, U, R>, Option<TokenReference<S>>)> {
        self.last_stmt.as_ref()
    }

    /// Returns a new block with the given statements
    /// Takes a vector of statements, followed by an optional semicolon token reference
    pub fn with_stmts(self, stmts: Vec<(Stmt<S, B, U, R>, Option<TokenReference<S>>)>) -> Self {
        Self { stmts, ..self }
    }

    /// Returns a new block with the given last statement, if one is given
    /// Takes an optional last statement, with an optional semicolon
    pub fn with_last_stmt(
        self,
        last_stmt: Option<(LastStmt<S, B, U, R>, Option<TokenReference<S>>)>,
    ) -> Self {
        Self { last_stmt, ..self }
    }

    pub(crate) fn merge_blocks(&mut self, other: Self) {
        self.stmts.extend(other.stmts);

        if self.last_stmt.is_none() {
            self.last_stmt = other.last_stmt;
        }
    }
}

/// A statement that stands alone
// #[derive(Clone, Debug, Display, PartialEq, Node, Visit)]
#[derive(Clone, Debug, Display, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[non_exhaustive]
pub enum Stmt<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    /// An assignment, such as `x = 1`
    #[display(fmt = "{_0}")]
    Assignment(Assignment<S, B, U, R>),
    /// A do block, `do end`
    #[display(fmt = "{_0}")]
    Do(Do<S, B, U, R>),
    /// A function call on its own, such as `call()`
    #[display(fmt = "{_0}")]
    FunctionCall(FunctionCall<S, B, U, R>),
    /// A function declaration, such as `function x() end`
    #[display(fmt = "{_0}")]
    FunctionDeclaration(FunctionDeclaration<S, B, U, R>),
    /// A generic for loop, such as `for index, value in pairs(list) do end`
    #[display(fmt = "{_0}")]
    GenericFor(GenericFor<S, B, U, R>),
    /// An if statement
    #[display(fmt = "{_0}")]
    If(If<S, B, U, R>),
    /// A local assignment, such as `local x = 1`
    #[display(fmt = "{_0}")]
    LocalAssignment(LocalAssignment<S, B, U, R>),
    /// A local function declaration, such as `local function x() end`
    #[display(fmt = "{_0}")]
    LocalFunction(LocalFunction<S, B, U, R>),
    /// A numeric for loop, such as `for index = 1, 10 do end`
    #[display(fmt = "{_0}")]
    NumericFor(NumericFor<S, B, U, R>),
    /// A repeat loop
    #[display(fmt = "{_0}")]
    Repeat(Repeat<S, B, U, R>),
    /// A while loop
    #[display(fmt = "{_0}")]
    While(While<S, B, U, R>),

    /// A compound assignment, such as `+=`
    /// Only available when the "luau" feature flag is enabled
    #[cfg(feature = "luau")]
    #[display(fmt = "{_0}")]
    CompoundAssignment(CompoundAssignment),
    /// An exported type declaration, such as `export type Meters = number`
    /// Only available when the "luau" feature flag is enabled.
    #[cfg(feature = "luau")]
    ExportedTypeDeclaration(ExportedTypeDeclaration),
    /// A type declaration, such as `type Meters = number`
    /// Only available when the "luau" feature flag is enabled.
    #[cfg(feature = "luau")]
    TypeDeclaration(TypeDeclaration),

    /// A goto statement, such as `goto label`
    /// Only available when the "lua52" feature flag is enabled.
    #[cfg(feature = "lua52")]
    Goto(Goto),
    /// A label, such as `::label::`
    /// Only available when the "lua52" feature flag is enabled.
    #[cfg(feature = "lua52")]
    Label(Label),
}

/// The last statement of a [`Block`]
// #[derive(Clone, Debug, Display, PartialEq, Node, Visit)]
#[derive(Clone, Debug, Display, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[non_exhaustive]
pub enum LastStmt<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    /// A `break` statement
    Break(TokenReference<S>),
    /// A continue statement
    /// Only available when the "luau" feature flag is enabled.
    #[cfg(feature = "luau")]
    Continue(TokenReference),
    /// A `return` statement
    Return(R),
}

/// A `return` statement
pub trait Return<S: AnySymbol, B: BinOp<S>, U: UnOp<S>> {
    /// The `return` token
    fn token(&self) -> &TokenReference<S>;
    /// The values being returned
    fn returns(&self) -> &Punctuated<Expression<S, B, U, Self>, S>;
    /// Returns a new Return with the given `return` token
    fn with_token(self, token: TokenReference<S>) -> Self;
    /// Returns a new Return with the given punctuated sequence
    fn with_returns(self, returns: Punctuated<Expression<S, B, U, Self>, S>) -> Self;
}

/// Fields of a [`TableConstructor`]
// #[derive(Clone, Debug, Display, PartialEq, Node)]
#[derive(Clone, Debug, Display, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[non_exhaustive]
pub enum Field<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    /// A key in the format of `[expression] = value`
    #[display(
        fmt = "{}{}{}{}{}",
        "brackets.tokens().0",
        "key",
        "brackets.tokens().1",
        "equal",
        "value"
    )]
    ExpressionKey {
        /// The `[...]` part of `[expression] = value`
        brackets: ContainedSpan<S>,
        /// The `expression` part of `[expression] = value`
        key: Expression<S, B, U, R>,
        /// The `=` part of `[expression] = value`
        equal: TokenReference<S>,
        /// The `value` part of `[expression] = value`
        value: Expression<S, B, U, R>,
    },

    /// A key in the format of `name = value`
    #[display(fmt = "{key}{equal}{value}")]
    NameKey {
        /// The `name` part of `name = value`
        key: TokenReference<S>,
        /// The `=` part of `name = value`
        equal: TokenReference<S>,
        /// The `value` part of `name = value`
        value: Expression<S, B, U, R>,
    },

    /// A field with no key, just a value (such as `"a"` in `{ "a" }`)
    #[display(fmt = "{_0}")]
    NoKey(Expression<S, B, U, R>),
}

/// A table being constructed, such as `{ 1, 2, 3 }` or `{ a = 1 }`
// #[derive(Clone, Debug, Display, PartialEq, Node, Visit)]
#[derive(Clone, Debug, Display, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[display(fmt = "{}{}{}", "braces.tokens().0", "fields", "braces.tokens().1")]
pub struct TableConstructor<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    // #[node(full_range)]
    // #[visit(contains = "fields")]
    braces: ContainedSpan<S>,
    fields: Punctuated<Field<S, B, U, R>, S>,
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> TableConstructor<S, B, U, R> {
    /// Creates a new empty TableConstructor
    /// Brace tokens are followed by spaces, such that { `fields` }
    pub fn new() -> Self {
        Self {
            braces: ContainedSpan::new(
                TokenReference::basic_symbol("{ "),
                TokenReference::basic_symbol(" }"),
            ),
            fields: Punctuated::new(),
        }
    }

    /// The braces of the constructor
    pub fn braces(&self) -> &ContainedSpan<S> {
        &self.braces
    }

    /// Returns the [`Punctuated`] sequence of the fields used to create the table
    pub fn fields(&self) -> &Punctuated<Field<S, B, U, R>, S> {
        &self.fields
    }

    /// Returns a new TableConstructor with the given braces
    pub fn with_braces(self, braces: ContainedSpan<S>) -> Self {
        Self { braces, ..self }
    }

    /// Returns a new TableConstructor with the given fields
    pub fn with_fields(self, fields: Punctuated<Field<S, B, U, R>, S>) -> Self {
        Self { fields, ..self }
    }
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> Default for TableConstructor<S, B, U, R> {
    fn default() -> Self {
        Self::new()
    }
}

/// An expression, mostly useful for getting values
// #[derive(Clone, Debug, Display, PartialEq, Node)]
#[derive(Clone, Debug, Display, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[non_exhaustive]
pub enum Expression<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    /// A binary operation, such as `1 + 3`
    #[display(fmt = "{lhs}{binop}{rhs}")]
    BinaryOperator {
        /// The left hand side of the binary operation, the `1` part of `1 + 3`
        lhs: Box<Expression<S, B, U, R>>,
        /// The binary operation used, the `+` part of `1 + 3`
        binop: B,
        /// The right hand side of the binary operation, the `3` part of `1 + 3`
        rhs: Box<Expression<S, B, U, R>>,
    },

    /// A statement in parentheses, such as `(#list)`
    #[display(
        fmt = "{}{}{}",
        "contained.tokens().0",
        "expression",
        "contained.tokens().1"
    )]
    Parentheses {
        /// The parentheses of the expression
        // #[node(full_range)]
        contained: ContainedSpan<S>,
        /// The expression inside the parentheses
        expression: Box<Expression<S, B, U, R>>,
    },

    /// A unary operation, such as `#list`
    #[display(fmt = "{unop}{expression}")]
    UnaryOperator {
        /// The unary operation, the `#` part of `#list`
        unop: U,
        /// The expression the operation is being done on, the `list` part of `#list`
        expression: Box<Expression<S, B, U, R>>,
    },

    /// An anonymous function, such as `function() end)`
    #[display(fmt = "{}{}", "_0.0", "_0.1")]
    Function((TokenReference<S>, FunctionBody<S, B, U, R>)),

    /// A call of a function, such as `call()`
    #[display(fmt = "{_0}")]
    FunctionCall(FunctionCall<S, B, U, R>),

    /// An if expression, such as `if foo then true else false`.
    /// Only available when the "luau" feature flag is enabled.
    #[cfg(feature = "luau")]
    #[display(fmt = "{_0}")]
    IfExpression(IfExpression),

    /// An interpolated string, such as `` `hello {"world"}` ``
    /// Only available when the "luau" feature flag is enabled.
    #[cfg(feature = "luau")]
    #[display(fmt = "{_0}")]
    InterpolatedString(InterpolatedString),

    /// A table constructor, such as `{ 1, 2, 3 }`
    #[display(fmt = "{_0}")]
    TableConstructor(TableConstructor<S, B, U, R>),

    /// A number token, such as `3.3`
    #[display(fmt = "{_0}")]
    Number(TokenReference<S>),

    /// A string token, such as `"hello"`
    #[display(fmt = "{_0}")]
    String(TokenReference<S>),

    /// A symbol, such as `true`
    #[display(fmt = "{_0}")]
    Symbol(TokenReference<S>),

    /// A value that has been asserted for a particular type, for use in Luau.
    /// Only available when the "luau" feature flag is enabled.
    #[cfg(feature = "luau")]
    #[display(fmt = "{expression}{type_assertion}")]
    TypeAssertion {
        /// The expression being asserted
        expression: Box<Expression>,

        /// The type assertion
        type_assertion: TypeAssertion,
    },

    /// A more complex value, such as `call().x`
    #[display(fmt = "{_0}")]
    Var(Var<S, B, U, R>),
}

/// A node used before another in cases such as function calling
/// The `("foo")` part of `("foo"):upper()`
// #[derive(Clone, Debug, Display, PartialEq, Node, Visit)]
#[derive(Clone, Debug, Display, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[non_exhaustive]
pub enum Prefix<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    #[display(fmt = "{_0}")]
    /// A complicated expression, such as `("foo")`
    Expression(Box<Expression<S, B, U, R>>),
    #[display(fmt = "{_0}")]
    /// Just a name, such as `foo`
    Name(TokenReference<S>),
}

/// The indexing of something, such as `x.y` or `x["y"]`
/// Values of variants are the keys, such as `"y"`
#[derive(Clone, Debug, Display, PartialEq)]
// #[derive(Clone, Debug, Display, PartialEq, Node)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[non_exhaustive]
pub enum Index<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    /// Indexing in the form of `x["y"]`
    #[display(
        fmt = "{}{}{}",
        "brackets.tokens().0",
        "expression",
        "brackets.tokens().1"
    )]
    Brackets {
        /// The `[...]` part of `["y"]`
        brackets: ContainedSpan<S>,
        /// The `"y"` part of `["y"]`
        expression: Expression<S, B, U, R>,
    },

    /// Indexing in the form of `x.y`
    #[display(fmt = "{dot}{name}")]
    Dot {
        /// The `.` part of `.y`
        dot: TokenReference<S>,
        /// The `y` part of `.y`
        name: TokenReference<S>,
    },
}

/// Arguments used for a function
// #[derive(Clone, Debug, Display, PartialEq, Node)]
#[derive(Clone, Debug, Display, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[non_exhaustive]
pub enum FunctionArgs<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    /// Used when a function is called in the form of `call(1, 2, 3)`
    #[display(
        fmt = "{}{}{}",
        "parentheses.tokens().0",
        "arguments",
        "parentheses.tokens().1"
    )]
    Parentheses {
        /// The `(...) part of (1, 2, 3)`
        // #[node(full_range)]
        parentheses: ContainedSpan<S>,
        /// The `1, 2, 3` part of `1, 2, 3`
        arguments: Punctuated<Expression<S, B, U, R>, S>,
    },
    /// Used when a function is called in the form of `call "foobar"`
    #[display(fmt = "{_0}")]
    String(TokenReference<S>),
    /// Used when a function is called in the form of `call { 1, 2, 3 }`
    #[display(fmt = "{_0}")]
    TableConstructor(TableConstructor<S, B, U, R>),
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> FunctionArgs<S, B, U, R> {
    pub(crate) fn empty() -> Self {
        FunctionArgs::Parentheses {
            parentheses: ContainedSpan::new(
                TokenReference::basic_symbol("("),
                TokenReference::basic_symbol(")"),
            ),

            arguments: Punctuated::new(),
        }
    }
}

/// A numeric for loop, such as `for index = 1, 10 do end`
// #[derive(Clone, Debug, PartialEq, Node)]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct NumericFor<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    for_token: TokenReference<S>,
    index_variable: TokenReference<S>,
    equal_token: TokenReference<S>,
    start: Expression<S, B, U, R>,
    start_end_comma: TokenReference<S>,
    end: Expression<S, B, U, R>,
    end_step_comma: Option<TokenReference<S>>,
    step: Option<Expression<S, B, U, R>>,
    do_token: TokenReference<S>,
    block: Block<S, B, U, R>,
    end_token: TokenReference<S>,
    #[cfg(feature = "luau")]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    type_specifier: Option<TypeSpecifier>,
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> NumericFor<S, B, U, R> {
    /// Creates a new NumericFor from the given index variable, start, and end expressions
    pub fn new(
        index_variable: TokenReference<S>,
        start: Expression<S, B, U, R>,
        end: Expression<S, B, U, R>,
    ) -> Self {
        Self {
            for_token: TokenReference::basic_symbol("for "),
            index_variable,
            equal_token: TokenReference::basic_symbol(" = "),
            start,
            start_end_comma: TokenReference::basic_symbol(", "),
            end,
            end_step_comma: None,
            step: None,
            do_token: TokenReference::basic_symbol(" do\n"),
            block: Block::new(),
            end_token: TokenReference::basic_symbol("\nend"),
            #[cfg(feature = "luau")]
            type_specifier: None,
        }
    }

    /// The `for` token
    pub fn for_token(&self) -> &TokenReference<S> {
        &self.for_token
    }

    /// The index identity, `index` in the initial example
    pub fn index_variable(&self) -> &TokenReference<S> {
        &self.index_variable
    }

    /// The `=` token
    pub fn equal_token(&self) -> &TokenReference<S> {
        &self.equal_token
    }

    /// The starting point, `1` in the initial example
    pub fn start(&self) -> &Expression<S, B, U, R> {
        &self.start
    }

    /// The comma in between the starting point and end point
    /// for _ = 1, 10 do
    ///          ^
    pub fn start_end_comma(&self) -> &TokenReference<S> {
        &self.start_end_comma
    }

    /// The ending point, `10` in the initial example
    pub fn end(&self) -> &Expression<S, B, U, R> {
        &self.end
    }

    /// The comma in between the ending point and limit, if one exists
    /// for _ = 0, 10, 2 do
    ///              ^
    pub fn end_step_comma(&self) -> Option<&TokenReference<S>> {
        self.end_step_comma.as_ref()
    }

    /// The step if one exists, `2` in `for index = 0, 10, 2 do end`
    pub fn step(&self) -> Option<&Expression<S, B, U, R>> {
        self.step.as_ref()
    }

    /// The `do` token
    pub fn do_token(&self) -> &TokenReference<S> {
        &self.do_token
    }

    /// The code inside the for loop
    pub fn block(&self) -> &Block<S, B, U, R> {
        &self.block
    }

    /// The `end` token
    pub fn end_token(&self) -> &TokenReference<S> {
        &self.end_token
    }

    /// The type specifiers of the index variable
    /// `for i: number = 1, 10 do` returns:
    /// `Some(TypeSpecifier(number))`
    /// Only available when the "luau" feature flag is enabled.
    #[cfg(feature = "luau")]
    pub fn type_specifier(&self) -> Option<&TypeSpecifier> {
        self.type_specifier.as_ref()
    }

    /// Returns a new NumericFor with the given for token
    pub fn with_for_token(self, for_token: TokenReference<S>) -> Self {
        Self { for_token, ..self }
    }

    /// Returns a new NumericFor with the given index variable
    pub fn with_index_variable(self, index_variable: TokenReference<S>) -> Self {
        Self {
            index_variable,
            ..self
        }
    }

    /// Returns a new NumericFor with the given `=` token
    pub fn with_equal_token(self, equal_token: TokenReference<S>) -> Self {
        Self {
            equal_token,
            ..self
        }
    }

    /// Returns a new NumericFor with the given start expression
    pub fn with_start(self, start: Expression<S, B, U, R>) -> Self {
        Self { start, ..self }
    }

    /// Returns a new NumericFor with the given comma between the start and end expressions
    pub fn with_start_end_comma(self, start_end_comma: TokenReference<S>) -> Self {
        Self {
            start_end_comma,
            ..self
        }
    }

    /// Returns a new NumericFor with the given end expression
    pub fn with_end(self, end: Expression<S, B, U, R>) -> Self {
        Self { end, ..self }
    }

    /// Returns a new NumericFor with the given comma between the end and the step expressions
    pub fn with_end_step_comma(self, end_step_comma: Option<TokenReference<S>>) -> Self {
        Self {
            end_step_comma,
            ..self
        }
    }

    /// Returns a new NumericFor with the given step expression
    pub fn with_step(self, step: Option<Expression<S, B, U, R>>) -> Self {
        Self { step, ..self }
    }

    /// Returns a new NumericFor with the given `do` token
    pub fn with_do_token(self, do_token: TokenReference<S>) -> Self {
        Self { do_token, ..self }
    }

    /// Returns a new NumericFor with the given block
    pub fn with_block(self, block: Block<S, B, U, R>) -> Self {
        Self { block, ..self }
    }

    /// Returns a new NumericFor with the given `end` token
    pub fn with_end_token(self, end_token: TokenReference<S>) -> Self {
        Self { end_token, ..self }
    }

    /// Returns a new NumericFor with the given type specifiers
    /// Only available when the "luau" feature flag is enabled.
    #[cfg(feature = "luau")]
    pub fn with_type_specifier(self, type_specifier: Option<TypeSpecifier>) -> Self {
        Self {
            type_specifier,
            ..self
        }
    }
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> fmt::Display
    for NumericFor<S, B, U, R>
{
    #[cfg(feature = "luau")]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}{}{}{}{}{}{}{}{}{}{}{}",
            self.for_token,
            self.index_variable,
            display_option(self.type_specifier()),
            self.equal_token,
            self.start,
            self.start_end_comma,
            self.end,
            display_option(self.end_step_comma()),
            display_option(self.step()),
            self.do_token,
            self.block,
            self.end_token,
        )
    }

    #[cfg(not(feature = "luau"))]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}{}{}{}{}{}{}{}{}{}{}",
            self.for_token,
            self.index_variable,
            self.equal_token,
            self.start,
            self.start_end_comma,
            self.end,
            display_option(self.end_step_comma()),
            display_option(self.step()),
            self.do_token,
            self.block,
            self.end_token,
        )
    }
}

/// A generic for loop, such as `for index, value in pairs(list) do end`
// #[derive(Clone, Debug, PartialEq, Node)]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct GenericFor<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    for_token: TokenReference<S>,
    names: Punctuated<TokenReference<S>, S>,
    in_token: TokenReference<S>,
    expr_list: Punctuated<Expression<S, B, U, R>, S>,
    do_token: TokenReference<S>,
    block: Block<S, B, U, R>,
    end_token: TokenReference<S>,
    #[cfg(feature = "luau")]
    type_specifiers: Vec<Option<TypeSpecifier>>,
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> GenericFor<S, B, U, R> {
    /// Creates a new GenericFor from the given names and expressions
    pub fn new(
        names: Punctuated<TokenReference<S>, S>,
        expr_list: Punctuated<Expression<S, B, U, R>, S>,
    ) -> Self {
        Self {
            for_token: TokenReference::basic_symbol("for "),
            names,
            in_token: TokenReference::basic_symbol(" in "),
            expr_list,
            do_token: TokenReference::basic_symbol(" do\n"),
            block: Block::new(),
            end_token: TokenReference::basic_symbol("\nend"),
            #[cfg(feature = "luau")]
            type_specifiers: Vec::new(),
        }
    }

    /// The `for` token
    pub fn for_token(&self) -> &TokenReference<S> {
        &self.for_token
    }

    /// Returns the punctuated sequence of names
    /// In `for index, value in pairs(list) do`, iterates over `index` and `value`
    pub fn names(&self) -> &Punctuated<TokenReference<S>, S> {
        &self.names
    }

    /// The `in` token
    pub fn in_token(&self) -> &TokenReference<S> {
        &self.in_token
    }

    /// Returns the punctuated sequence of the expressions looped over
    /// In `for index, value in pairs(list) do`, iterates over `pairs(list)`
    pub fn expressions(&self) -> &Punctuated<Expression<S, B, U, R>, S> {
        &self.expr_list
    }

    /// The `do` token
    pub fn do_token(&self) -> &TokenReference<S> {
        &self.do_token
    }

    /// The code inside the for loop
    pub fn block(&self) -> &Block<S, B, U, R> {
        &self.block
    }

    /// The `end` token
    pub fn end_token(&self) -> &TokenReference<S> {
        &self.end_token
    }

    /// The type specifiers of the named variables, in the order that they were assigned.
    /// `for i, v: string in pairs() do` returns an iterator containing:
    /// `None, Some(TypeSpecifier(string))`
    /// Only available when the "luau" feature flag is enabled.
    #[cfg(feature = "luau")]
    pub fn type_specifiers(&self) -> impl Iterator<Item = Option<&TypeSpecifier>> {
        self.type_specifiers.iter().map(Option::as_ref)
    }

    /// Returns a new GenericFor with the given `for` token
    pub fn with_for_token(self, for_token: TokenReference<S>) -> Self {
        Self { for_token, ..self }
    }

    /// Returns a new GenericFor with the given names
    pub fn with_names(self, names: Punctuated<TokenReference<S>, S>) -> Self {
        Self { names, ..self }
    }

    /// Returns a new GenericFor with the given `in` token
    pub fn with_in_token(self, in_token: TokenReference<S>) -> Self {
        Self { in_token, ..self }
    }

    /// Returns a new GenericFor with the given expression list
    pub fn with_expressions(self, expr_list: Punctuated<Expression<S, B, U, R>, S>) -> Self {
        Self { expr_list, ..self }
    }

    /// Returns a new GenericFor with the given `do` token
    pub fn with_do_token(self, do_token: TokenReference<S>) -> Self {
        Self { do_token, ..self }
    }

    /// Returns a new GenericFor with the given block
    pub fn with_block(self, block: Block<S, B, U, R>) -> Self {
        Self { block, ..self }
    }

    /// Returns a new GenericFor with the given `end` token
    pub fn with_end_token(self, end_token: TokenReference<S>) -> Self {
        Self { end_token, ..self }
    }

    /// Returns a new GenericFor with the given type specifiers
    /// Only available when the "luau" feature flag is enabled.
    #[cfg(feature = "luau")]
    pub fn with_type_specifiers(self, type_specifiers: Vec<Option<TypeSpecifier>>) -> Self {
        Self {
            type_specifiers,
            ..self
        }
    }
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> fmt::Display
    for GenericFor<S, B, U, R>
{
    #[cfg(feature = "luau")]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}{}{}{}{}{}{}",
            self.for_token,
            join_type_specifiers(&self.names, self.type_specifiers()),
            self.in_token,
            self.expr_list,
            self.do_token,
            self.block,
            self.end_token
        )
    }

    #[cfg(not(feature = "luau"))]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}{}{}{}{}{}{}",
            self.for_token,
            self.names,
            self.in_token,
            self.expr_list,
            self.do_token,
            self.block,
            self.end_token
        )
    }
}

/// An if statement
// #[derive(Clone, Debug, Display, PartialEq, Node, Visit)]
#[derive(Clone, Debug, Display, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[display(
    fmt = "{}{}{}{}{}{}{}{}",
    "if_token",
    "condition",
    "then_token",
    "block",
    "display_option(else_if.as_ref().map(join_vec))",
    "display_option(else_token)",
    "display_option(r#else)",
    "end_token"
)]
pub struct If<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    if_token: TokenReference<S>,
    condition: Expression<S, B, U, R>,
    then_token: TokenReference<S>,
    block: Block<S, B, U, R>,
    else_if: Option<Vec<ElseIf<S, B, U, R>>>,
    else_token: Option<TokenReference<S>>,
    #[cfg_attr(feature = "serde", serde(rename = "else"))]
    r#else: Option<Block<S, B, U, R>>,
    end_token: TokenReference<S>,
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> If<S, B, U, R> {
    /// Creates a new If from the given condition
    pub fn new(condition: Expression<S, B, U, R>) -> Self {
        Self {
            if_token: TokenReference::basic_symbol("if "),
            condition,
            then_token: TokenReference::basic_symbol(" then"),
            block: Block::new(),
            else_if: None,
            else_token: None,
            r#else: None,
            end_token: TokenReference::basic_symbol("\nend"),
        }
    }

    /// The `if` token
    pub fn if_token(&self) -> &TokenReference<S> {
        &self.if_token
    }

    /// The condition of the if statement, `condition` in `if condition then`
    pub fn condition(&self) -> &Expression<S, B, U, R> {
        &self.condition
    }

    /// The `then` token
    pub fn then_token(&self) -> &TokenReference<S> {
        &self.then_token
    }

    /// The block inside the initial if statement
    pub fn block(&self) -> &Block<S, B, U, R> {
        &self.block
    }

    /// The `else` token if one exists
    pub fn else_token(&self) -> Option<&TokenReference<S>> {
        self.else_token.as_ref()
    }

    /// If there are `elseif` conditions, returns a vector of them
    /// Expression is the condition, block is the code if the condition is true
    // TODO: Make this return an iterator, and remove Option part entirely?
    pub fn else_if(&self) -> Option<&Vec<ElseIf<S, B, U, R>>> {
        self.else_if.as_ref()
    }

    /// The code inside an `else` block if one exists
    pub fn else_block(&self) -> Option<&Block<S, B, U, R>> {
        self.r#else.as_ref()
    }

    /// The `end` token
    pub fn end_token(&self) -> &TokenReference<S> {
        &self.end_token
    }

    /// Returns a new If with the given `if` token
    pub fn with_if_token(self, if_token: TokenReference<S>) -> Self {
        Self { if_token, ..self }
    }

    /// Returns a new If with the given condition
    pub fn with_condition(self, condition: Expression<S, B, U, R>) -> Self {
        Self { condition, ..self }
    }

    /// Returns a new If with the given `then` token
    pub fn with_then_token(self, then_token: TokenReference<S>) -> Self {
        Self { then_token, ..self }
    }

    /// Returns a new If with the given block
    pub fn with_block(self, block: Block<S, B, U, R>) -> Self {
        Self { block, ..self }
    }

    /// Returns a new If with the given list of `elseif` blocks
    pub fn with_else_if(self, else_if: Option<Vec<ElseIf<S, B, U, R>>>) -> Self {
        Self { else_if, ..self }
    }

    /// Returns a new If with the given `else` token
    pub fn with_else_token(self, else_token: Option<TokenReference<S>>) -> Self {
        Self { else_token, ..self }
    }

    /// Returns a new If with the given `else` body
    pub fn with_else(self, r#else: Option<Block<S, B, U, R>>) -> Self {
        Self { r#else, ..self }
    }

    /// Returns a new If with the given `end` token
    pub fn with_end_token(self, end_token: TokenReference<S>) -> Self {
        Self { end_token, ..self }
    }
}

/// An elseif block in a bigger [`If`] statement
// #[derive(Clone, Debug, Display, PartialEq, Node, Visit)]
#[derive(Clone, Debug, Display, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[display(fmt = "{else_if_token}{condition}{then_token}{block}")]
pub struct ElseIf<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    else_if_token: TokenReference<S>,
    condition: Expression<S, B, U, R>,
    then_token: TokenReference<S>,
    block: Block<S, B, U, R>,
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> ElseIf<S, B, U, R> {
    /// Creates a new ElseIf from the given condition
    pub fn new(condition: Expression<S, B, U, R>) -> Self {
        Self {
            else_if_token: TokenReference::basic_symbol("elseif "),
            condition,
            then_token: TokenReference::basic_symbol(" then\n"),
            block: Block::new(),
        }
    }

    /// The `elseif` token
    pub fn else_if_token(&self) -> &TokenReference<S> {
        &self.else_if_token
    }

    /// The condition of the `elseif`, `condition` in `elseif condition then`
    pub fn condition(&self) -> &Expression<S, B, U, R> {
        &self.condition
    }

    /// The `then` token
    pub fn then_token(&self) -> &TokenReference<S> {
        &self.then_token
    }

    /// The body of the `elseif`
    pub fn block(&self) -> &Block<S, B, U, R> {
        &self.block
    }

    /// Returns a new ElseIf with the given `elseif` token
    pub fn with_else_if_token(self, else_if_token: TokenReference<S>) -> Self {
        Self {
            else_if_token,
            ..self
        }
    }

    /// Returns a new ElseIf with the given condition
    pub fn with_condition(self, condition: Expression<S, B, U, R>) -> Self {
        Self { condition, ..self }
    }

    /// Returns a new ElseIf with the given `then` token
    pub fn with_then_token(self, then_token: TokenReference<S>) -> Self {
        Self { then_token, ..self }
    }

    /// Returns a new ElseIf with the given block
    pub fn with_block(self, block: Block<S, B, U, R>) -> Self {
        Self { block, ..self }
    }
}

/// A while loop
// #[derive(Clone, Debug, Display, PartialEq, Node, Visit)]
#[derive(Clone, Debug, Display, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[display(fmt = "{while_token}{condition}{do_token}{block}{end_token}")]
pub struct While<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    while_token: TokenReference<S>,
    condition: Expression<S, B, U, R>,
    do_token: TokenReference<S>,
    block: Block<S, B, U, R>,
    end_token: TokenReference<S>,
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> While<S, B, U, R> {
    /// Creates a new While from the given condition
    pub fn new(condition: Expression<S, B, U, R>) -> Self {
        Self {
            while_token: TokenReference::basic_symbol("while "),
            condition,
            do_token: TokenReference::basic_symbol(" do\n"),
            block: Block::new(),
            end_token: TokenReference::basic_symbol("end\n"),
        }
    }

    /// The `while` token
    pub fn while_token(&self) -> &TokenReference<S> {
        &self.while_token
    }

    /// The `condition` part of `while condition do`
    pub fn condition(&self) -> &Expression<S, B, U, R> {
        &self.condition
    }

    /// The `do` token
    pub fn do_token(&self) -> &TokenReference<S> {
        &self.do_token
    }

    /// The code inside the while loop
    pub fn block(&self) -> &Block<S, B, U, R> {
        &self.block
    }

    /// The `end` token
    pub fn end_token(&self) -> &TokenReference<S> {
        &self.end_token
    }

    /// Returns a new While with the given `while` token
    pub fn with_while_token(self, while_token: TokenReference<S>) -> Self {
        Self {
            while_token,
            ..self
        }
    }

    /// Returns a new While with the given condition
    pub fn with_condition(self, condition: Expression<S, B, U, R>) -> Self {
        Self { condition, ..self }
    }

    /// Returns a new While with the given `do` token
    pub fn with_do_token(self, do_token: TokenReference<S>) -> Self {
        Self { do_token, ..self }
    }

    /// Returns a new While with the given block
    pub fn with_block(self, block: Block<S, B, U, R>) -> Self {
        Self { block, ..self }
    }

    /// Returns a new While with the given `end` token
    pub fn with_end_token(self, end_token: TokenReference<S>) -> Self {
        Self { end_token, ..self }
    }
}

/// A repeat loop
// #[derive(Clone, Debug, Display, PartialEq, Node, Visit)]
#[derive(Clone, Debug, Display, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[display(fmt = "{repeat_token}{block}{until_token}{until}")]
pub struct Repeat<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    repeat_token: TokenReference<S>,
    block: Block<S, B, U, R>,
    until_token: TokenReference<S>,
    until: Expression<S, B, U, R>,
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> Repeat<S, B, U, R> {
    /// Creates a new Repeat from the given expression to repeat until
    pub fn new(until: Expression<S, B, U, R>) -> Self {
        Self {
            repeat_token: TokenReference::basic_symbol("repeat\n"),
            block: Block::new(),
            until_token: TokenReference::basic_symbol("\nuntil "),
            until,
        }
    }

    /// The `repeat` token
    pub fn repeat_token(&self) -> &TokenReference<S> {
        &self.repeat_token
    }

    /// The code inside the `repeat` block
    pub fn block(&self) -> &Block<S, B, U, R> {
        &self.block
    }

    /// The `until` token
    pub fn until_token(&self) -> &TokenReference<S> {
        &self.until_token
    }

    /// The condition for the `until` part
    pub fn until(&self) -> &Expression<S, B, U, R> {
        &self.until
    }

    /// Returns a new Repeat with the given `repeat` token
    pub fn with_repeat_token(self, repeat_token: TokenReference<S>) -> Self {
        Self {
            repeat_token,
            ..self
        }
    }

    /// Returns a new Repeat with the given block
    pub fn with_block(self, block: Block<S, B, U, R>) -> Self {
        Self { block, ..self }
    }

    /// Returns a new Repeat with the given `until` token
    pub fn with_until_token(self, until_token: TokenReference<S>) -> Self {
        Self {
            until_token,
            ..self
        }
    }

    /// Returns a new Repeat with the given `until` block
    pub fn with_until(self, until: Expression<S, B, U, R>) -> Self {
        Self { until, ..self }
    }
}

/// A method call, such as `x:y()`
// #[derive(Clone, Debug, Display, PartialEq, Node, Visit)]
#[derive(Clone, Debug, Display, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[display(fmt = "{colon_token}{name}{args}")]
pub struct MethodCall<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    colon_token: TokenReference<S>,
    name: TokenReference<S>,
    args: FunctionArgs<S, B, U, R>,
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> MethodCall<S, B, U, R> {
    /// Returns a new MethodCall from the given name and args
    pub fn new(name: TokenReference<S>, args: FunctionArgs<S, B, U, R>) -> Self {
        Self {
            colon_token: TokenReference::basic_symbol(":"),
            name,
            args,
        }
    }

    /// The `:` in `x:y()`
    pub fn colon_token(&self) -> &TokenReference<S> {
        &self.colon_token
    }

    /// The arguments of a method call, the `x, y, z` part of `method:call(x, y, z)`
    pub fn args(&self) -> &FunctionArgs<S, B, U, R> {
        &self.args
    }

    /// The method being called, the `call` part of `method:call()`
    pub fn name(&self) -> &TokenReference<S> {
        &self.name
    }

    /// Returns a new MethodCall with the given `:` token
    pub fn with_colon_token(self, colon_token: TokenReference<S>) -> Self {
        Self {
            colon_token,
            ..self
        }
    }

    /// Returns a new MethodCall with the given name
    pub fn with_name(self, name: TokenReference<S>) -> Self {
        Self { name, ..self }
    }

    /// Returns a new MethodCall with the given args
    pub fn with_args(self, args: FunctionArgs<S, B, U, R>) -> Self {
        Self { args, ..self }
    }
}

/// Something being called
// #[derive(Clone, Debug, Display, PartialEq, Node, Visit)]
#[derive(Clone, Debug, Display, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[non_exhaustive]
pub enum Call<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    #[display(fmt = "{_0}")]
    /// A function being called directly, such as `x(1)`
    AnonymousCall(FunctionArgs<S, B, U, R>),
    #[display(fmt = "{_0}")]
    /// A method call, such as `x:y()`
    MethodCall(MethodCall<S, B, U, R>),
}

/// A function body, everything except `function x` in `function x(a, b, c) call() end`
// #[derive(Clone, Debug, PartialEq, Node)]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct FunctionBody<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    #[cfg(feature = "luau")]
    generics: Option<GenericDeclaration>,

    parameters_parentheses: ContainedSpan<S>,
    parameters: Punctuated<Parameter<S>, S>,

    #[cfg(feature = "luau")]
    type_specifiers: Vec<Option<TypeSpecifier>>,

    #[cfg(feature = "luau")]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    return_type: Option<TypeSpecifier>,

    block: Block<S, B, U, R>,
    end_token: TokenReference<S>,
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> FunctionBody<S, B, U, R> {
    /// Returns a new empty FunctionBody
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "luau")]
            generics: None,

            parameters_parentheses: ContainedSpan::new(
                TokenReference::basic_symbol("("),
                TokenReference::basic_symbol(")"),
            ),
            parameters: Punctuated::new(),

            #[cfg(feature = "luau")]
            type_specifiers: Vec::new(),

            #[cfg(feature = "luau")]
            return_type: None,

            block: Block::new(),
            end_token: TokenReference::basic_symbol("\nend"),
        }
    }

    /// The parentheses of the parameters
    pub fn parameters_parentheses(&self) -> &ContainedSpan<S> {
        &self.parameters_parentheses
    }

    /// Returns the [`Punctuated`] sequence of the parameters for the function declaration
    pub fn parameters(&self) -> &Punctuated<Parameter<S>, S> {
        &self.parameters
    }

    /// The code of a function body
    pub fn block(&self) -> &Block<S, B, U, R> {
        &self.block
    }

    /// The `end` token
    pub fn end_token(&self) -> &TokenReference<S> {
        &self.end_token
    }

    /// The generics declared for the function body.
    /// The `<T, U>` part of `function x<T, U>() end`
    /// Only available when the "luau" feature flag is enabled.
    #[cfg(feature = "luau")]
    pub fn generics(&self) -> Option<&GenericDeclaration> {
        self.generics.as_ref()
    }

    /// The type specifiers of the variables, in the order that they were assigned.
    /// `(foo: number, bar, baz: boolean)` returns an iterator containing:
    /// `Some(TypeSpecifier(number)), None, Some(TypeSpecifier(boolean))`
    /// Only available when the "luau" feature flag is enabled.
    #[cfg(feature = "luau")]
    pub fn type_specifiers(&self) -> impl Iterator<Item = Option<&TypeSpecifier>> {
        self.type_specifiers.iter().map(Option::as_ref)
    }

    /// The return type of the function, if one exists.
    /// Only available when the "luau" feature flag is enabled.
    #[cfg(feature = "luau")]
    pub fn return_type(&self) -> Option<&TypeSpecifier> {
        self.return_type.as_ref()
    }

    /// Returns a new FunctionBody with the given parentheses for the parameters
    pub fn with_parameters_parentheses(self, parameters_parentheses: ContainedSpan<S>) -> Self {
        Self {
            parameters_parentheses,
            ..self
        }
    }

    /// Returns a new FunctionBody with the given parameters
    pub fn with_parameters(self, parameters: Punctuated<Parameter<S>, S>) -> Self {
        Self { parameters, ..self }
    }

    /// Returns a new FunctionBody with the given generics declaration
    #[cfg(feature = "luau")]
    pub fn with_generics(self, generics: Option<GenericDeclaration>) -> Self {
        Self { generics, ..self }
    }

    /// Returns a new FunctionBody with the given type specifiers
    #[cfg(feature = "luau")]
    pub fn with_type_specifiers(self, type_specifiers: Vec<Option<TypeSpecifier>>) -> Self {
        Self {
            type_specifiers,
            ..self
        }
    }

    /// Returns a new FunctionBody with the given return type
    #[cfg(feature = "luau")]
    pub fn with_return_type(self, return_type: Option<TypeSpecifier>) -> Self {
        Self {
            return_type,
            ..self
        }
    }

    /// Returns a new FunctionBody with the given block
    pub fn with_block(self, block: Block<S, B, U, R>) -> Self {
        Self { block, ..self }
    }

    /// Returns a new FunctionBody with the given `end` token
    pub fn with_end_token(self, end_token: TokenReference<S>) -> Self {
        Self { end_token, ..self }
    }
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> Default
    for FunctionBody<S, B, U, R>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> fmt::Display
    for FunctionBody<S, B, U, R>
{
    #[cfg(feature = "luau")]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}{}{}{}{}{}{}",
            display_option(self.generics.as_ref()),
            self.parameters_parentheses.tokens().0,
            join_type_specifiers(&self.parameters, self.type_specifiers()),
            self.parameters_parentheses.tokens().1,
            display_option(self.return_type.as_ref()),
            self.block,
            self.end_token
        )
    }

    #[cfg(not(feature = "luau"))]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}{}{}{}{}",
            self.parameters_parentheses.tokens().0,
            self.parameters,
            self.parameters_parentheses.tokens().1,
            self.block,
            self.end_token
        )
    }
}

/// A parameter in a function declaration
// #[derive(Clone, Debug, Display, PartialEq, Eq, Node, Visit)]
#[derive(Clone, Debug, Display, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[non_exhaustive]
pub enum Parameter<S: AnySymbol> {
    /// The `...` vararg syntax, such as `function x(...)`
    Ellipse(TokenReference<S>),
    /// A name parameter, such as `function x(a, b, c)`
    Name(TokenReference<S>),
}

/// A suffix in certain cases, such as `:y()` in `x:y()`
/// Can be stacked on top of each other, such as in `x()()()`
// #[derive(Clone, Debug, Display, PartialEq, Node, Visit)]
#[derive(Clone, Debug, Display, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[non_exhaustive]
pub enum Suffix<S: AnySymbol, B: BinOp<S>, U: BinOp<S>, R: Return<S, B, U>> {
    #[display(fmt = "{_0}")]
    /// A call, including method calls and direct calls
    Call(Call<S, B, U, R>),
    #[display(fmt = "{_0}")]
    /// An index, such as `x.y`
    Index(Index<S, B, U, R>),
}

/// A complex expression used by [`Var`], consisting of both a prefix and suffixes
// #[derive(Clone, Debug, Display, PartialEq, Node, Visit)]
#[derive(Clone, Debug, Display, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[display(fmt = "{}{}", "prefix", "join_vec(suffixes)")]
pub struct VarExpression<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    prefix: Prefix<S, B, U, R>,
    suffixes: Vec<Suffix<S, B, U, R>>,
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> VarExpression<S, B, U, R> {
    /// Returns a new VarExpression from the given prefix
    pub fn new(prefix: Prefix<S, B, U, R>) -> Self {
        Self {
            prefix,
            suffixes: Vec::new(),
        }
    }

    /// The prefix of the expression, such as a name
    pub fn prefix(&self) -> &Prefix<S, B, U, R> {
        &self.prefix
    }

    /// An iter over the suffixes, such as indexing or calling
    pub fn suffixes(&self) -> impl Iterator<Item = &Suffix<S, B, U, R>> {
        self.suffixes.iter()
    }

    /// Returns a new VarExpression with the given prefix
    pub fn with_prefix(self, prefix: Prefix<S, B, U, R>) -> Self {
        Self { prefix, ..self }
    }

    /// Returns a new VarExpression with the given suffixes
    pub fn with_suffixes(self, suffixes: Vec<Suffix<S, B, U, R>>) -> Self {
        Self { suffixes, ..self }
    }
}

/// Used in [`Assignment`s](Assignment) and [`Value`s](Value)
// #[derive(Clone, Debug, Display, PartialEq, Node, Visit)]
#[derive(Clone, Debug, Display, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[non_exhaustive]
pub enum Var<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    /// An expression, such as `x.y.z` or `x()`
    #[display(fmt = "{_0}")]
    Expression(Box<VarExpression<S, B, U, R>>),
    /// A literal identifier, such as `x`
    #[display(fmt = "{_0}")]
    Name(TokenReference<S>),
}

/// An assignment, such as `x = y`. Not used for [`LocalAssignment`s](LocalAssignment)
// #[derive(Clone, Debug, Display, PartialEq, Node, Visit)]
#[derive(Clone, Debug, Display, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[display(fmt = "{var_list}{equal_token}{expr_list}")]
pub struct Assignment<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    var_list: Punctuated<Var<S, B, U, R>, S>,
    equal_token: TokenReference<S>,
    expr_list: Punctuated<Expression<S, B, U, R>, S>,
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> Assignment<S, B, U, R> {
    /// Returns a new Assignment from the given variable and expression list
    pub fn new(
        var_list: Punctuated<Var<S, B, U, R>, S>,
        expr_list: Punctuated<Expression<S, B, U, R>, S>,
    ) -> Self {
        Self {
            var_list,
            equal_token: TokenReference::basic_symbol(" = "),
            expr_list,
        }
    }

    /// Returns the punctuated sequence over the expressions being assigned.
    /// This is the the `1, 2` part of `x, y["a"] = 1, 2`
    pub fn expressions(&self) -> &Punctuated<Expression<S, B, U, R>, S> {
        &self.expr_list
    }

    /// The `=` token in between `x = y`
    pub fn equal_token(&self) -> &TokenReference<S> {
        &self.equal_token
    }

    /// Returns the punctuated sequence over the variables being assigned to.
    /// This is the `x, y["a"]` part of `x, y["a"] = 1, 2`
    pub fn variables(&self) -> &Punctuated<Var<S, B, U, R>, S> {
        &self.var_list
    }

    /// Returns a new Assignment with the given variables
    pub fn with_variables(self, var_list: Punctuated<Var<S, B, U, R>, S>) -> Self {
        Self { var_list, ..self }
    }

    /// Returns a new Assignment with the given `=` token
    pub fn with_equal_token(self, equal_token: TokenReference<S>) -> Self {
        Self {
            equal_token,
            ..self
        }
    }

    /// Returns a new Assignment with the given expressions
    pub fn with_expressions(self, expr_list: Punctuated<Expression<S, B, U, R>, S>) -> Self {
        Self { expr_list, ..self }
    }
}

/// A declaration of a local function, such as `local function x() end`
// #[derive(Clone, Debug, Display, PartialEq, Node, Visit)]
#[derive(Clone, Debug, Display, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    not(feature = "luau"),
    display(fmt = "{local_token}{function_token}{name}{body}")
)]
#[cfg_attr(
    feature = "luau",
    display(fmt = "{local_token}{function_token}{name}{body}")
)]
pub struct LocalFunction<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    local_token: TokenReference<S>,
    function_token: TokenReference<S>,
    name: TokenReference<S>,
    body: FunctionBody<S, B, U, R>,
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> LocalFunction<S, B, U, R> {
    /// Returns a new LocalFunction from the given name
    pub fn new(name: TokenReference<S>) -> Self {
        LocalFunction {
            local_token: TokenReference::basic_symbol("local "),
            function_token: TokenReference::basic_symbol("function "),
            name,
            body: FunctionBody::new(),
        }
    }

    /// The `local` token
    pub fn local_token(&self) -> &TokenReference<S> {
        &self.local_token
    }

    /// The `function` token
    pub fn function_token(&self) -> &TokenReference<S> {
        &self.function_token
    }

    /// The function body, everything except `local function x` in `local function x(a, b, c) call() end`
    pub fn body(&self) -> &FunctionBody<S, B, U, R> {
        &self.body
    }

    /// The name of the function, the `x` part of `local function x() end`
    pub fn name(&self) -> &TokenReference<S> {
        &self.name
    }

    /// Returns a new LocalFunction with the given `local` token
    pub fn with_local_token(self, local_token: TokenReference<S>) -> Self {
        Self {
            local_token,
            ..self
        }
    }

    /// Returns a new LocalFunction with the given `function` token
    pub fn with_function_token(self, function_token: TokenReference<S>) -> Self {
        Self {
            function_token,
            ..self
        }
    }

    /// Returns a new LocalFunction with the given name
    pub fn with_name(self, name: TokenReference<S>) -> Self {
        Self { name, ..self }
    }

    /// Returns a new LocalFunction with the given function body
    pub fn with_body(self, body: FunctionBody<S, B, U, R>) -> Self {
        Self { body, ..self }
    }
}

/// An assignment to a local variable, such as `local x = 1`
// #[derive(Clone, Debug, PartialEq, Node)]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct LocalAssignment<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    local_token: TokenReference<S>,
    #[cfg(feature = "luau")]
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "empty_optional_vector")
    )]
    type_specifiers: Vec<Option<TypeSpecifier>>,
    name_list: Punctuated<TokenReference<S>, S>,
    #[cfg(feature = "lua54")]
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "empty_optional_vector")
    )]
    attributes: Vec<Option<Attribute>>,
    equal_token: Option<TokenReference<S>>,
    expr_list: Punctuated<Expression<S, B, U, R>, S>,
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> LocalAssignment<S, B, U, R> {
    /// Returns a new LocalAssignment from the given name list
    pub fn new(name_list: Punctuated<TokenReference<S>, S>) -> Self {
        Self {
            local_token: TokenReference::basic_symbol("local "),
            #[cfg(feature = "luau")]
            type_specifiers: Vec::new(),
            name_list,
            #[cfg(feature = "lua54")]
            attributes: Vec::new(),
            equal_token: None,
            expr_list: Punctuated::new(),
        }
    }

    /// The `local` token
    pub fn local_token(&self) -> &TokenReference<S> {
        &self.local_token
    }

    /// The `=` token in between `local x = y`, if one exists
    pub fn equal_token(&self) -> Option<&TokenReference<S>> {
        self.equal_token.as_ref()
    }

    /// Returns the punctuated sequence of the expressions being assigned.
    /// This is the `1, 2` part of `local x, y = 1, 2`
    pub fn expressions(&self) -> &Punctuated<Expression<S, B, U, R>, S> {
        &self.expr_list
    }

    /// Returns the punctuated sequence of names being assigned to.
    /// This is the `x, y` part of `local x, y = 1, 2`
    pub fn names(&self) -> &Punctuated<TokenReference<S>, S> {
        &self.name_list
    }

    /// The type specifiers of the variables, in the order that they were assigned.
    /// `local foo: number, bar, baz: boolean` returns an iterator containing:
    /// `Some(TypeSpecifier(number)), None, Some(TypeSpecifier(boolean))`
    /// Only available when the "luau" feature flag is enabled.
    #[cfg(feature = "luau")]
    pub fn type_specifiers(&self) -> impl Iterator<Item = Option<&TypeSpecifier>> {
        self.type_specifiers.iter().map(Option::as_ref)
    }

    /// The attributes specified for the variables, in the order that they were assigned.
    /// `local foo <const>, bar, baz <close>` returns an iterator containing:
    /// `Some(Attribute("const")), None, Some(Attribute("close"))`
    /// Only available when the "lua54" feature flag is enabled.
    #[cfg(feature = "lua54")]
    pub fn attributes(&self) -> impl Iterator<Item = Option<&Attribute>> {
        self.attributes.iter().map(Option::as_ref)
    }

    /// Returns a new LocalAssignment with the given `local` token
    pub fn with_local_token(self, local_token: TokenReference<S>) -> Self {
        Self {
            local_token,
            ..self
        }
    }

    /// Returns a new LocalAssignment with the given type specifiers
    #[cfg(feature = "luau")]
    pub fn with_type_specifiers(self, type_specifiers: Vec<Option<TypeSpecifier>>) -> Self {
        Self {
            type_specifiers,
            ..self
        }
    }

    /// Returns a new LocalAssignment with the given attributes
    #[cfg(feature = "lua54")]
    pub fn with_attributes(self, attributes: Vec<Option<Attribute>>) -> Self {
        Self { attributes, ..self }
    }

    /// Returns a new LocalAssignment with the given name list
    pub fn with_names(self, name_list: Punctuated<TokenReference<S>, S>) -> Self {
        Self { name_list, ..self }
    }

    /// Returns a new LocalAssignment with the given `=` token
    pub fn with_equal_token(self, equal_token: Option<TokenReference<S>>) -> Self {
        Self {
            equal_token,
            ..self
        }
    }

    /// Returns a new LocalAssignment with the given expression list
    pub fn with_expressions(self, expr_list: Punctuated<Expression<S, B, U, R>, S>) -> Self {
        Self { expr_list, ..self }
    }
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> fmt::Display
    for LocalAssignment<S, B, U, R>
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        #[cfg(feature = "lua54")]
        let attributes = self.attributes().chain(std::iter::repeat(None));
        #[cfg(not(feature = "lua54"))]
        let attributes = std::iter::repeat_with(|| None::<TokenReference<S>>);
        #[cfg(feature = "luau")]
        let type_specifiers = self.type_specifiers().chain(std::iter::repeat(None));
        #[cfg(not(feature = "luau"))]
        let type_specifiers = std::iter::repeat_with(|| None::<TokenReference<S>>);

        write!(
            formatter,
            "{}{}{}{}",
            self.local_token,
            join_iterators(&self.name_list, attributes, type_specifiers),
            display_option(&self.equal_token),
            self.expr_list
        )
    }
}

/// A `do` block, such as `do ... end`
/// This is not used for things like `while true do end`, only those on their own
// #[derive(Clone, Debug, Display, PartialEq, Node, Visit)]
#[derive(Clone, Debug, Display, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[display(fmt = "{do_token}{block}{end_token}")]
pub struct Do<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    do_token: TokenReference<S>,
    block: Block<S, B, U, R>,
    end_token: TokenReference<S>,
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> Do<S, B, U, R> {
    /// Creates an empty Do
    pub fn new() -> Self {
        Self {
            do_token: TokenReference::basic_symbol("do\n"),
            block: Block::new(),
            end_token: TokenReference::basic_symbol("\nend"),
        }
    }

    /// The `do` token
    pub fn do_token(&self) -> &TokenReference<S> {
        &self.do_token
    }

    /// The code inside the `do ... end`
    pub fn block(&self) -> &Block<S, B, U, R> {
        &self.block
    }

    /// The `end` token
    pub fn end_token(&self) -> &TokenReference<S> {
        &self.end_token
    }

    /// Returns a new Do with the given `do` token
    pub fn with_do_token(self, do_token: TokenReference<S>) -> Self {
        Self { do_token, ..self }
    }

    /// Returns a new Do with the given block
    pub fn with_block(self, block: Block<S, B, U, R>) -> Self {
        Self { block, ..self }
    }

    /// Returns a new Do with the given `end` token
    pub fn with_end_token(self, end_token: TokenReference<S>) -> Self {
        Self { end_token, ..self }
    }
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> Default for Do<S, B, U, R> {
    fn default() -> Self {
        Self::new()
    }
}

/// A function being called, such as `call()`
// #[derive(Clone, Debug, Display, PartialEq, Node, Visit)]
#[derive(Clone, Debug, Display, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[display(fmt = "{}{}", "prefix", "join_vec(suffixes)")]
pub struct FunctionCall<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    prefix: Prefix<S, B, U, R>,
    suffixes: Vec<Suffix<S, B, U, R>>,
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> FunctionCall<S, B, U, R> {
    /// Creates a new FunctionCall from the given prefix
    /// Sets the suffixes such that the return is `prefixes()`
    pub fn new(prefix: Prefix<S, B, U, R>) -> Self {
        FunctionCall {
            prefix,
            suffixes: vec![Suffix::Call(Call::AnonymousCall(
                FunctionArgs::Parentheses {
                    arguments: Punctuated::new(),
                    parentheses: ContainedSpan::new(
                        TokenReference::basic_symbol("("),
                        TokenReference::basic_symbol(")"),
                    ),
                },
            ))],
        }
    }

    /// The prefix of a function call, the `call` part of `call()`
    pub fn prefix(&self) -> &Prefix<S, B, U, R> {
        &self.prefix
    }

    /// The suffix of a function call, the `()` part of `call()`
    pub fn suffixes(&self) -> impl Iterator<Item = &Suffix<S, B, U, R>> {
        self.suffixes.iter()
    }

    /// Returns a new FunctionCall with the given prefix
    pub fn with_prefix(self, prefix: Prefix<S, B, U, R>) -> Self {
        Self { prefix, ..self }
    }

    /// Returns a new FunctionCall with the given suffixes
    pub fn with_suffixes(self, suffixes: Vec<Suffix<S, B, U, R>>) -> Self {
        Self { suffixes, ..self }
    }
}

/// A function name when being declared as [`FunctionDeclaration`]
// #[derive(Clone, Debug, Display, PartialEq, Eq, Node, Visit)]
#[derive(Clone, Debug, Display, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[display(
    fmt = "{}{}{}",
    "names",
    "display_option(self.method_colon())",
    "display_option(self.method_name())"
)]
pub struct FunctionName<S: AnySymbol> {
    names: Punctuated<TokenReference<S>, S>,
    colon_name: Option<(TokenReference<S>, TokenReference<S>)>,
}

impl<S: AnySymbol> FunctionName<S> {
    /// Creates a new FunctionName from the given list of names
    pub fn new(names: Punctuated<TokenReference<S>, S>) -> Self {
        Self {
            names,
            colon_name: None,
        }
    }

    /// The colon between the name and the method, the `:` part of `function x:y() end`
    pub fn method_colon(&self) -> Option<&TokenReference<S>> {
        Some(&self.colon_name.as_ref()?.0)
    }

    /// A method name if one exists, the `y` part of `function x:y() end`
    pub fn method_name(&self) -> Option<&TokenReference<S>> {
        Some(&self.colon_name.as_ref()?.1)
    }

    /// Returns the punctuated sequence over the names used when defining the function.
    /// This is the `x.y.z` part of `function x.y.z() end`
    pub fn names(&self) -> &Punctuated<TokenReference<S>, S> {
        &self.names
    }

    /// Returns a new FunctionName with the given names
    pub fn with_names(self, names: Punctuated<TokenReference<S>, S>) -> Self {
        Self { names, ..self }
    }

    /// Returns a new FunctionName with the given method name
    /// The first token is the colon, and the second token is the method name itself
    pub fn with_method(self, method: Option<(TokenReference<S>, TokenReference<S>)>) -> Self {
        Self {
            colon_name: method,
            ..self
        }
    }
}

/// A normal function declaration, supports simple declarations like `function x() end`
/// as well as complicated declarations such as `function x.y.z:a() end`
// #[derive(Clone, Debug, Display, PartialEq, Node, Visit)]
#[derive(Clone, Debug, Display, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(not(feature = "luau"), display(fmt = "{function_token}{name}{body}"))]
#[cfg_attr(feature = "luau", display(fmt = "{function_token}{name}{body}"))]
pub struct FunctionDeclaration<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> {
    function_token: TokenReference<S>,
    name: FunctionName<S>,
    body: FunctionBody<S, B, U, R>,
}

impl<S: AnySymbol, B: BinOp<S>, U: UnOp<S>, R: Return<S, B, U>> FunctionDeclaration<S, B, U, R> {
    /// Creates a new FunctionDeclaration from the given name
    pub fn new(name: FunctionName<S>) -> Self {
        Self {
            function_token: TokenReference::basic_symbol("function "),
            name,
            body: FunctionBody::new(),
        }
    }

    /// The `function` token
    pub fn function_token(&self) -> &TokenReference<S> {
        &self.function_token
    }

    /// The body of the function
    pub fn body(&self) -> &FunctionBody<S, B, U, R> {
        &self.body
    }

    /// The name of the function
    pub fn name(&self) -> &FunctionName<S> {
        &self.name
    }

    /// Returns a new FunctionDeclaration with the given `function` token
    pub fn with_function_token(self, function_token: TokenReference<S>) -> Self {
        Self {
            function_token,
            ..self
        }
    }

    /// Returns a new FunctionDeclaration with the given function name
    pub fn with_name(self, name: FunctionName<S>) -> Self {
        Self { name, ..self }
    }

    /// Returns a new FunctionDeclaration with the given function body
    pub fn with_body(self, body: FunctionBody<S, B, U, R>) -> Self {
        Self { body, ..self }
    }
}

/// Operators that require just one operand, such as #X
pub trait UnOp<S> {
    /// The token associated with the operator
    fn token(&self) -> &TokenReference<S>;
}

pub trait BinOp<S> {
    /// The precedence of the operator. The larger the number, the higher the precedence.
    /// See more at <http://www.lua.org/manual/5.1/manual.html#2.5.6>
    fn precedence(&self) -> u8;
    /// Whether the operator is right associative. If not, it is left associative.
    /// See more at <https://www.lua.org/pil/3.5.html>
    fn is_right_associative(&self) -> bool;
    /// Given a token, returns whether it is a right associative binary operator.
    fn is_right_associative_token(token: &TokenReference<S>) -> bool;
}

/// An error that occurs when creating the AST.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct AstError<S: AnySymbol> {
    /// The token that caused the error
    token: Token<S>,

    /// Any additional information that could be provided for debugging
    additional: Cow<'static, str>,

    /// If set, this is the complete range of the error
    #[serde(skip_serializing_if = "Option::is_none")]
    range: Option<(Position, Position)>,
}

impl<S: AnySymbol> AstError<S> {
    /// Returns a human readable error message
    pub fn error_message(&self) -> Cow<'static, str> {
        self.additional.clone()
    }

    /// Returns the range of the error
    pub fn range(&self) -> (Position, Position) {
        self.range
            .or_else(|| Some((self.token.start_position(), self.token.end_position())))
            .unwrap()
    }
}

impl<S: AnySymbol> fmt::Display for AstError<S> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let range = self.range();

        write!(
            formatter,
            "unexpected token `{}`. (starting from line {}, character {} and ending on line {}, character {})\nadditional information: {}",
            self.token,
            range.0.line(),
            range.0.character(),
            range.1.line(),
            range.1.character(),
            self.additional,
        )
    }
}

impl<S: AnySymbol + fmt::Debug> std::error::Error for AstError<S> {}
