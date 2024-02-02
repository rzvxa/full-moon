#[macro_export]
macro_rules! create_visitor {
    (ast: {
        $($visit_name:ident => $ast_type:ident,)+

        $(#[$meta:meta] {
            $($meta_visit_name:ident => $meta_ast_type:ident,)+
        })+
    }, token: {
        $($visit_token:ident,)+

        $(#[$token_meta:meta] {
            $($meta_visit_token:ident,)+
        })+
    }) => {
        /// A trait that implements functions to listen for specific nodes/tokens.
        /// Unlike [`VisitorMut`], nodes/tokens passed are immutable.
        ///
        /// ```rust
        /// # use full_moon::ast;
        /// # use full_moon::visitors::*;
        /// # fn main() -> Result<(), Vec<full_moon::Error>> {
        /// // A visitor that logs every local assignment made
        /// #[derive(Default)]
        /// struct LocalVariableVisitor {
        ///     names: Vec<String>,
        /// }
        ///
        /// impl Visitor for LocalVariableVisitor {
        ///     fn visit_local_assignment(&mut self, local_assignment: &ast::LocalAssignment) {
        ///         self.names.extend(&mut local_assignment.names().iter().map(|name| name.token().to_string()));
        ///     }
        /// }
        ///
        /// let mut visitor = LocalVariableVisitor::default();
        /// visitor.visit_ast(&full_moon::parse("local x = 1; local y, z = 2, 3")?);
        /// assert_eq!(visitor.names, vec!["x", "y", "z"]);
        /// # Ok(())
        /// # }
        /// ```
        pub trait Visitor {
            /// Visit the nodes of an [`Ast`](crate::ast::Ast)
            fn visit_ast(&mut self, ast: &Ast) where Self: Sized {
                ast.nodes().visit(self);
                ast.eof().visit(self);
            }

            paste::item! {
                $(
                    #[allow(missing_docs)]
                    fn $visit_name(&mut self, _node: &$ast_type) { }
                    #[allow(missing_docs)]
                    fn [<$visit_name _end>](&mut self, _node: &$ast_type) { }
                )+

                $(
                    $(
                        #[$meta]
                        #[allow(missing_docs)]
                        fn $meta_visit_name(&mut self, _node: &$meta_ast_type) { }
                        #[$meta]
                        #[allow(missing_docs)]
                        fn [<$meta_visit_name _end>](&mut self, _node: &$meta_ast_type) { }
                    )+
                )+
            }

            $(
                #[allow(missing_docs)]
                fn $visit_token(&mut self, _token: &Token) { }
            )+

            $(
                $(
                    #[$token_meta]
                    #[allow(missing_docs)]
                    fn $meta_visit_token(&mut self, _token: &Token) { }
                )+
            )+
        }

        /// A trait that implements functions to listen for specific nodes/tokens.
        /// Unlike [`Visitor`], nodes/tokens passed are mutable.
        pub trait VisitorMut {
            /// Visit the nodes of an [`Ast`](crate::ast::Ast)
            fn visit_ast(&mut self, ast: Ast) -> Ast where Self: Sized {
                // TODO: Visit tokens?
                let eof = ast.eof().to_owned();
                let nodes = ast.nodes.visit_mut(self);

                Ast {
                    nodes,
                    // Everything gets cloned with this visitor, so there's no original tokens
                    eof: self.visit_eof(eof),
                }
            }

            paste::item! {
                $(
                    #[allow(missing_docs)]
                    fn $visit_name(&mut self, node: $ast_type) -> $ast_type {
                        node
                    }

                    #[allow(missing_docs)]
                    fn [<$visit_name _end>](&mut self, node: $ast_type) -> $ast_type {
                        node
                    }
                )+

                $(
                    #[$meta]
                    $(
                        #[$meta]
                        #[allow(missing_docs)]
                        fn $meta_visit_name(&mut self, node: $meta_ast_type) -> $meta_ast_type {
                            node
                        }

                        #[$meta]
                        #[allow(missing_docs)]
                        fn [<$meta_visit_name _end>](&mut self, node: $meta_ast_type) -> $meta_ast_type {
                            node
                        }
                    )+
                )+
            }

            $(
                #[allow(missing_docs)]
                fn $visit_token(&mut self, token: Token) -> Token {
                    token
                }
            )+

            $(
                #[$token_meta]
                $(
                    #[$token_meta]
                    #[allow(missing_docs)]
                    fn $meta_visit_token(&mut self, token: Token) -> Token {
                        token
                    }
                )+
            )+
        }
    };
}

#[doc(hidden)]
pub trait Visit<V> {
    fn visit(&self, visitor: &mut V);
}

#[doc(hidden)]
pub trait VisitMut<V>
where
    Self: Sized,
{
    fn visit_mut(self, visitor: &mut V) -> Self;
}

impl<V, T: Visit<V>> Visit<V> for &T {
    fn visit(&self, visitor: &mut V) {
        (**self).visit(visitor);
    }
}

impl<V, T: Visit<V>> Visit<V> for &mut T {
    fn visit(&self, visitor: &mut V) {
        (**self).visit(visitor);
    }
}

impl<V, T: Visit<V>> Visit<V> for Vec<T> {
    fn visit(&self, visitor: &mut V) {
        for item in self {
            item.visit(visitor);
        }
    }
}

impl<V, T: VisitMut<V>> VisitMut<V> for Vec<T> {
    fn visit_mut(self, visitor: &mut V) -> Self {
        self.into_iter()
            .map(|item| item.visit_mut(visitor))
            .collect()
    }
}

impl<V, T: Visit<V>> Visit<V> for Option<T> {
    fn visit(&self, visitor: &mut V) {
        if let Some(item) = self {
            item.visit(visitor);
        }
    }
}

impl<V, T: VisitMut<V>> VisitMut<V> for Option<T> {
    fn visit_mut(self, visitor: &mut V) -> Self {
        self.map(|item| item.visit_mut(visitor))
    }
}

impl<V, A: Visit<V>, B: Visit<V>> Visit<V> for (A, B) {
    fn visit(&self, visitor: &mut V) {
        self.0.visit(visitor);
        self.1.visit(visitor);
    }
}

impl<V, A: VisitMut<V>, B: VisitMut<V>> VisitMut<V> for (A, B) {
    fn visit_mut(self, visitor: &mut V) -> Self {
        (self.0.visit_mut(visitor), self.1.visit_mut(visitor))
    }
}

impl<V, T: Visit<V>> Visit<V> for Box<T> {
    fn visit(&self, visitor: &mut V) {
        (**self).visit(visitor);
    }
}

impl<V, T: VisitMut<V>> VisitMut<V> for Box<T> {
    fn visit_mut(self, visitor: &mut V) -> Self {
        Box::new((*self).visit_mut(visitor))
    }
}
