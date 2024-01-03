use proc_macro::{self, TokenStream};
use quote::quote;
use syn::visit_mut::{self, VisitMut};
use syn::{parse_quote, Expr};

struct TryExprReplace {
    log_function: syn::ExprPath,
}

impl VisitMut for TryExprReplace {
    fn visit_expr_mut(&mut self, node: &mut Expr) {
        if let Expr::Try(try_expr) = &node {
            let expr = &try_expr.expr;
            let logfn = self.log_function.clone();
            *node = parse_quote!(
                match #expr {
                    Ok(expr) => expr,
                    Err(err) => {
                        #logfn! ("Try expression #expr failed with: {}", err);
                        return Err(err.into());

                    }
                }
            );
            return;
        }

        // Delegate to the default impl to visit nested expressions.
        visit_mut::visit_expr_mut(self, node);
    }
}

#[proc_macro]
pub fn try_or_err(tokens: TokenStream) -> TokenStream {
    let expr: syn::Expr = syn::parse_macro_input!(tokens);
    quote!(
        match #expr {
            Ok(expr) => expr,
            Err(err) => {
                error! ("Try expression failed with: {}", err);
                return Err(err.into());

            }
        }

    )
    .into()
}

#[proc_macro_attribute]
pub fn log_tries(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut syntax_tree: syn::ImplItemFn = syn::parse_macro_input!(item);
    let logfn: syn::ExprPath = syn::parse_macro_input!(attr);
    TryExprReplace {
        log_function: logfn,
    }
    .visit_block_mut(&mut syntax_tree.block);
    quote!(#syntax_tree).into()
}
