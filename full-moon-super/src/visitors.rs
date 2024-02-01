use crate::tokenizer::{Token, TokenReference};
use full_moon_common::{
    create_visitor,
    visitors::{Visit, VisitMut},
};

impl<S> Visit for Token<S> {
    fn visit<V>(&self, visitor: &mut V) {
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
    fn visit_mut<V>(self, visitor: &mut V) -> Self {
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

create_visitor!(ast: {
    visit_anonymous_call => FunctionArgs,
    visit_assignment => Assignment,
    visit_block => Block,
    visit_call => Call,
    visit_contained_span => ContainedSpan,
    visit_do => Do,
    visit_else_if => ElseIf,
    visit_eof => TokenReference,
    visit_expression => Expression,
    visit_field => Field,
    visit_function_args => FunctionArgs,
    visit_function_body => FunctionBody,
    visit_function_call => FunctionCall,
    visit_function_declaration => FunctionDeclaration,
    visit_function_name => FunctionName,
    visit_generic_for => GenericFor,
    visit_if => If,
    visit_index => Index,
    visit_local_assignment => LocalAssignment,
    visit_local_function => LocalFunction,
    visit_last_stmt => LastStmt,
    visit_method_call => MethodCall,
    visit_numeric_for => NumericFor,
    visit_parameter => Parameter,
    visit_prefix => Prefix,
    visit_return => Return,
    visit_repeat => Repeat,
    visit_stmt => Stmt,
    visit_suffix => Suffix,
    visit_table_constructor => TableConstructor,
    visit_token_reference => TokenReference,
    visit_un_op => UnOp,
    visit_var => Var,
    visit_var_expression => VarExpression,
    visit_while => While,

    // Types
    #[cfg(feature = "luau")] {
        visit_compound_assignment => CompoundAssignment,
        visit_compound_op => CompoundOp,
        visit_else_if_expression => ElseIfExpression,
        visit_exported_type_declaration => ExportedTypeDeclaration,
        visit_generic_declaration => GenericDeclaration,
        visit_generic_declaration_parameter => GenericDeclarationParameter,
        visit_generic_parameter_info => GenericParameterInfo,
        visit_if_expression => IfExpression,
        visit_indexed_type_info => IndexedTypeInfo,
        visit_interpolated_string => InterpolatedString,
        visit_type_argument => TypeArgument,
        visit_type_assertion => TypeAssertion,
        visit_type_declaration => TypeDeclaration,
        visit_type_field => TypeField,
        visit_type_field_key => TypeFieldKey,
        visit_type_info => TypeInfo,
        visit_type_specifier => TypeSpecifier,
    }

    // Lua 5.2
    #[cfg(feature = "lua52")] {
        visit_goto => Goto,
        visit_label => Label,
    }

    #[cfg(feature = "lua54")] {
        visit_attribute => Attribute,
    }
}, token: {
    visit_identifier,
    visit_multi_line_comment,
    visit_number,
    visit_single_line_comment,
    visit_string_literal,
    visit_symbol,
    visit_token,
    visit_whitespace,

    #[cfg(feature = "luau")] {
        visit_interpolated_string_segment,
    }
});
