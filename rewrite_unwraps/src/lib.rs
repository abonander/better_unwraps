#![feature(proc_macro)]

extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;

use quote::{Tokens, ToTokens};

use syn::fold::*;
use syn::*;

use std::iter;

const CRATE_NAME: &'static str = "better_unwraps";

// Items and expressions annotated with `#[leave_unwrap]` will
// not have their unwrap invocations rewritten.
const LEAVE_UNWRAPS: &'static str = "leave_unwraps";

const CALL_SITE_EXPR: &'static str = "call_site!()";

#[proc_macro_attribute]
pub fn rewrite_unwraps(attr: TokenStream, input: TokenStream) -> TokenStream {
    let inject = match &*attr.to_string() {
        "( inject )" => true,
        "( )" | "" => false,
        _ => panic!("Unsupported invocation: {}", attr),
    };

    let item = syn::parse_item(&input.to_string())
        .expect("Attribute can only be applied to items");

    if let ItemKind::Mod(None) = item.node {
        panic!("`#[rewrite_unwraps]` cannot see inside other modules' files; if you want \
                the attribute to process this module then please add `#![rewrite_unwraps]` \
                inside the module file itself.");
    }

    let mut tokens = Tokens::new();

    RewriteUnwraps { inject, inject_level: -1 }.fold_item(item).to_tokens(&mut tokens);

    tokens.parse().expect("Output should be parsable Rust")
}

struct RewriteUnwraps {
    inject: bool,
    inject_level: i32,
}

impl RewriteUnwraps {
    /// Get the path for any UFCS calls (the `better_unwrap` crate)
    fn base_path(&self) -> Path {
        if self.inject_level < 0 {
            return Path { global: true, segments: vec![CRATE_NAME.into()] };
        }

        let segments = if self.inject_level == 0 {
            vec!["self".into(), CRATE_NAME.into()]
        } else {
            (0 .. self.inject_level)
                .map(|_| "super".into())
                .chain(iter::once(CRATE_NAME.into()))
                .collect()
        };

        Path {
            global: false,
            segments: segments,
        }
    }

    fn extern_crate_decl(&self) -> Item {
        let krate: Ident = CRATE_NAME.into();

        let macro_use_attr = Attribute {
            style: AttrStyle::Inner,
            value: MetaItem::Word("macro_use".into()),
            is_sugared_doc: false,
        };

        Item {
            ident: krate,
            vis: Visibility::Inherited,
            attrs: vec![macro_use_attr],
            node: ItemKind::ExternCrate(None),
        }
    }

    fn rewrite_unwrap(&self, fn_name: &str, mut args: Vec<Expr>, attrs: Vec<Attribute>) -> Expr {
        let (fn_name, trait_name) = match fn_name {
            "unwrap" => ("unwrap_ext", "UnwrapExt"),
            "unwrap_err" => ("unwrap_err_ext", "UnwrapErrExt"),
            _ => panic!("No extension trait for `{}` method.", fn_name),
        };

        let mut ufcs_path = self.base_path();
        ufcs_path.segments.push(trait_name.into());
        ufcs_path.segments.push(fn_name.into());

        let path_expr = Expr {
            node: ExprKind::Path(None, ufcs_path),
            attrs: vec![],
        };

        let call_site_expr = parse_expr(CALL_SITE_EXPR)
            .expect("`CALL_SITE_EXPR` could not be parsed as an expression");

        args.push(call_site_expr);

        Expr {
            node: ExprKind::Call(Box::new(path_expr), args),
            attrs: attrs,
        }
    }
}

impl Folder for RewriteUnwraps {
    fn fold_item(&mut self, mut item: Item) -> Item {
        use syn::ItemKind::*;

        match item.node {
            Mod(Some(ref mut items)) => {
                if self.inject {
                    self.inject = false;
                    items.push(self.extern_crate_decl());

                    // This does mean that inline modules at the crate root will use relative
                    // paths instead of absolute ones, but that's acceptable.
                    self.inject_level = 0;
                } else if self.inject_level >= 0 {
                    self.inject_level += 1;
                }
            }
            _ => (),
        }

        if should_fold(&mut item.attrs) {
            noop_fold_item(self, item)
        } else {
            item
        }
    }

    fn fold_expr(&mut self, mut expr: Expr) -> Expr {
        use syn::ExprKind::*;

        if !should_fold(&mut expr.attrs) {
            return expr;
        }

        expr = match expr.node {
            MethodCall(fn_name, generics, args) => if fn_name == "unwrap" {
                self.rewrite_unwrap("unwrap", args, expr.attrs)
            } else if fn_name == "unwrap_err" {
                self.rewrite_unwrap("unwrap_err", args, expr.attrs)
            } else {
                expr.node = MethodCall(fn_name, generics, args);
                expr
            },
            _ => expr,
        };

        noop_fold_expr(self, expr)
    }
}

fn should_fold(attrs: &mut Vec<Attribute>) -> bool {
    !remove_attr(attrs, LEAVE_UNWRAPS)
}

fn remove_attr(attrs: &mut Vec<Attribute>, attr: &str) -> bool {
    match attrs.iter().position(|maybe| check_attr(maybe, attr)) {
        Some(idx) => { attrs.remove(idx); true }
        _ => false,
    }
}

fn check_attr(attr: &Attribute, name: &str) -> bool {
    match attr.value {
        MetaItem::Word(ref word) => word == name,
        _ => false,
    }
}
