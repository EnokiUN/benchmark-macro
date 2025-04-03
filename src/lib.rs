use proc_macro::TokenStream;
use quote::{format_ident, ToTokens};
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, spanned::Spanned, Error, Expr, ItemFn,
    LitStr, Meta, Stmt, Token,
};

macro_rules! get_attrs {
    ($expr:ident, $($type:ident),*) => {
        match  $expr {
            $(
                Expr::$type(expr) => expr.attrs.clone(),
             )*
            _ => vec![],
        }
    };
    (mut $expr:ident, $($type:ident),*) => {
        match  $expr {
            $(
                Expr::$type(expr) => &mut expr.attrs,
             )*
            _ => &mut vec![],
        }
    };
}

#[proc_macro_attribute]
pub fn bench(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(item as ItemFn);
    let mut benched: Vec<String> = vec![];

    let _: Vec<&Stmt> = item
        .block
        .stmts
        .iter()
        .inspect(|s| println!("{:#?}", s))
        .collect();

    let mut stmt_idx = 1;
    while stmt_idx < item.block.stmts.len() {
        let attrs = match &item.block.stmts[stmt_idx] {
            Stmt::Local(local) => local.attrs.clone(),
            Stmt::Expr(expr, _) => get_attrs!(
                expr, Array, Assign, Async, Await, Binary, Block, Break, Call, Cast, Closure,
                Const, Continue, Field, ForLoop, Group, If, Index, Infer, Let, Lit, Loop, Macro,
                Match, MethodCall, Paren, Path, Range, RawAddr, Reference, Repeat, Return, Struct,
                Try, TryBlock, Tuple, Unary, Unsafe, While, Yield
            ),
            Stmt::Macro(stmt_macro) => stmt_macro.attrs.clone(),
            _ => vec![],
        };
        let mut bench_macro_idx = None;
        for (i, attr) in attrs.iter().enumerate() {
            if attr.path().is_ident("bench") {
                if let Meta::List(list) = &attr.meta {
                    match list.parse_args_with(Punctuated::<LitStr, Token![,]>::parse_terminated) {
                        Ok(list) => {
                            if list.len() != 1 {
                                return Error::new(list.span(), "Invalid usage of `bench` macro. Only one argument is supported.")
                                    .to_compile_error()
                                    .into();
                            }
                            let name = list.first().unwrap().clone().value();
                            let var_ident = format_ident!("_bench_{}_time", name);
                            let instant_ident = format_ident!("_bench_{}_instant", name);
                            benched.push(name);
                            bench_macro_idx = Some(i);
                            item.block.stmts.insert(
                                stmt_idx,
                                parse_quote! {let #instant_ident = std::time::Instant::now();},
                            );
                            item.block
                                .stmts
                                .insert(stmt_idx + 2, parse_quote! {let #var_ident = #var_ident + #instant_ident.elapsed().as_micros();});
                            stmt_idx += 2;
                        }
                        Err(err) => return err.to_compile_error().into(),
                    }
                } else {
                    return Error::new(attr.span(), "Invalid usage of `bench` macro")
                        .to_compile_error()
                        .into();
                }
            }
        }
        if let Some(i) = bench_macro_idx {
            let attrs = match &mut item.block.stmts[stmt_idx - 1] {
                Stmt::Local(local) => &mut local.attrs,
                Stmt::Expr(expr, _) => get_attrs!(
                    mut expr,
                    Array,
                    Assign,
                    Async,
                    Await,
                    Binary,
                    Block,
                    Break,
                    Call,
                    Cast,
                    Closure,
                    Const,
                    Continue,
                    Field,
                    ForLoop,
                    Group,
                    If,
                    Index,
                    Infer,
                    Let,
                    Lit,
                    Loop,
                    Macro,
                    Match,
                    MethodCall,
                    Paren,
                    Path,
                    Range,
                    RawAddr,
                    Reference,
                    Repeat,
                    Return,
                    Struct,
                    Try,
                    TryBlock,
                    Tuple,
                    Unary,
                    Unsafe,
                    While,
                    Yield
                ),
                Stmt::Macro(stmt_macro) => &mut stmt_macro.attrs,
                _ => &mut vec![],
            };
            attrs.remove(i);
        }
        stmt_idx += 1;
    }

    for name in benched {
        let var_ident = format_ident!("_bench_{}_time", name);
        item.block
            .stmts
            .insert(0, parse_quote! {let #var_ident: u128 = 0;});
        item.block.stmts.push(
            parse_quote! {log::debug!("Benchmark for {}: {} microseconds", #name, #var_ident);},
        );
    }

    TokenStream::from(item.to_token_stream())
}
