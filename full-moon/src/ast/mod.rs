mod parser_structs;
#[macro_use]
mod parser_util;
mod parsers;
// pub mod punctuated;
pub mod span;
mod update_positions;
mod visitors;

use crate::{
    Language,
    tokenizer::{Position, Symbol, Token, TokenReference, TokenType},
    util::*,
};
use derive_more::Display;
use full_moon_derive::{Node, Visit};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt};

use punctuated::{Pair, Punctuated};
use span::ContainedSpan;

pub use parser_structs::AstResult;

mod versions;
pub use versions::*;

#[cfg(feature = "luau")]
pub mod types;
#[cfg(feature = "luau")]
use types::*;

#[cfg(feature = "luau")]
mod type_visitors;

#[cfg(feature = "lua52")]
pub mod lua52;
#[cfg(feature = "lua52")]
use lua52::*;

#[cfg(feature = "lua54")]
pub mod lua54;
#[cfg(feature = "lua54")]
use lua54::*;












#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse, print, visitors::VisitorMut};

    #[test]
    fn test_with_eof_safety() {
        let new_ast = {
            let ast = parse("local foo = 1").unwrap();
            let eof = ast.eof().clone();
            ast.with_eof(eof)
        };

        print(&new_ast);
    }

    #[test]
    fn test_with_nodes_safety() {
        let new_ast = {
            let ast = parse("local foo = 1").unwrap();
            let nodes = ast.nodes().clone();
            ast.with_nodes(nodes)
        };

        print(&new_ast);
    }

    #[test]
    fn test_with_visitor_safety() {
        let new_ast = {
            let ast = parse("local foo = 1").unwrap();

            struct SyntaxRewriter;
            impl VisitorMut for SyntaxRewriter {
                fn visit_token(&mut self, token: Token) -> Token {
                    token
                }
            }

            SyntaxRewriter.visit_ast(ast)
        };

        print(&new_ast);
    }

    // Tests AST nodes with new methods that call unwrap
    #[test]
    fn test_new_validity() {
        let token: TokenReference = TokenReference::new(
            Vec::new(),
            Token::new(TokenType::Identifier {
                identifier: "foo".into(),
            }),
            Vec::new(),
        );

        let expression = Expression::Var(Var::Name(token.clone()));

        Assignment::new(Punctuated::new(), Punctuated::new());
        Do::new();
        ElseIf::new(expression.clone());
        FunctionBody::new();
        FunctionCall::new(Prefix::Name(token.clone()));
        FunctionDeclaration::new(FunctionName::new(Punctuated::new()));
        GenericFor::new(Punctuated::new(), Punctuated::new());
        If::new(expression.clone());
        LocalAssignment::new(Punctuated::new());
        LocalFunction::new(token.clone());
        MethodCall::new(
            token.clone(),
            FunctionArgs::Parentheses {
                arguments: Punctuated::new(),
                parentheses: ContainedSpan::new(token.clone(), token.clone()),
            },
        );
        NumericFor::new(token, expression.clone(), expression.clone());
        Repeat::new(expression.clone());
        Return::new();
        TableConstructor::new();
        While::new(expression);
    }

    // TODO: Uncomment me later!
    // #[test]
    // fn test_local_assignment_print() {
    //     let block = Block::new().with_stmts(vec![(
    //         Stmt::LocalAssignment(
    //             LocalAssignment::new(
    //                 std::iter::once(Pair::End(TokenReference::new(
    //                     vec![],
    //                     Token::new(TokenType::Identifier {
    //                         identifier: "variable".into(),
    //                     }),
    //                     vec![],
    //                 )))
    //                 .collect(),
    //             )
    //             .with_equal_token(Some(TokenReference::symbol(" = ").unwrap()))
    //             .with_expressions(
    //                 std::iter::once(Pair::End(Expression::Number(TokenReference::new(
    //                     vec![],
    //                     Token::new(TokenType::Number { text: "1".into() }),
    //                     vec![],
    //                 ))))
    //                 .collect(),
    //             ),
    //         ),
    //         None,
    //     )]);
    //
    //     let ast = parse("").unwrap().with_nodes(block);
    //     assert_eq!(print(&ast), "local variable = 1");
    // }
}
