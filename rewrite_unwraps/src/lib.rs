#![feature(proc_macro)]

extern crate proc_macro;

use proc_macro::{quote, Delimiter, Span, Term, TokenNode, TokenTree, TokenTreeIter, TokenStream};

use std::iter;

const CRATE_NAME: &'static str = "better_unwraps";

// Items and expressions annotated with `#[leave_unwrap]` will
// not have their unwrap invocations rewritten.
const LEAVE_UNWRAPS: &'static str = "leave_unwraps";

const CALL_SITE_EXPR: &'static str = "call_site!()";

#[proc_macro_attribute]
pub fn rewrite_unwraps(attr: TokenStream, input: TokenStream) -> TokenStream {
    match &*attr.to_string() {
        "( )" | "" => false,
        _ => panic!("Unsupported invocation: `#[rewrite_unwraps{}]`", attr),
    };

    let rewrite = RewriteUnwraps {
        inject: false,
        inject_level: 0,
    };

    rewrite.map_tokens(input.into_iter())
}

#[proc_macro]
pub fn print_input(input: TokenStream) -> TokenStream {
    let input = input.into_iter().collect::<Vec<_>>();

    panic!("{:?}", input);
}

struct RewriteUnwraps {
    inject: bool,
    inject_level: i32,
}

impl RewriteUnwraps {
    fn map_tokens(&self, mut iter: TokenTreeIter) -> TokenStream {
        let unwrap = Term::intern("unwrap_ext");
        let unwrap_err = Term::intern("unwrap_err_ext");

        let mut out = vec![];

        let mut invoc_span = None;

        for mut token in &mut iter {
            use self::TokenNode::*;

            token.kind = match token.kind {
                Term(ident) => Term(match ident.as_str() {
                    "unwrap" => {invoc_span = Some(token.span); unwrap },
                    "unwrap_err" => {invoc_span = Some(token.span); unwrap_err},
                    _ => {invoc_span = None; ident },
                }),
                Group(Delimiter::Parenthesis, ts) => if let Some(invoc_span) = invoc_span {
                    Group(Delimiter::Parenthesis, self.gen_call_site(invoc_span))
                } else {
                    Group(Delimiter::Parenthesis, self.map_tokens(ts.into_iter()))
                },
                Group(delim, ts) => Group(delim, self.map_tokens(ts.into_iter())),
                kind => { invoc_span = None; kind },
            };

            out.push(token);
        }

        out.into_iter().collect()
    }

    fn gen_call_site(&self, invoc_span: Span) -> TokenStream {
        quote!( (call_site!()) ).into_iter().map(|mut t| { t.span = invoc_span; t}).collect()
    }
}


