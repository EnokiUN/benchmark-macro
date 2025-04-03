use proc_macro::TokenStream;
use quote::{format_ident, ToTokens};
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, spanned::Spanned, Error, Expr, ExprIf,
    ItemFn, LitStr, Meta, Stmt, Token,
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

fn handle_if(benched: &mut Vec<String>, if_expr: &mut ExprIf) -> Result<(), TokenStream> {
    handle_macro(benched, &mut if_expr.then_branch.stmts)?;
    if let Some(else_branch) = &mut if_expr.else_branch {
        match &mut *else_branch.1 {
            Expr::Block(expr_block) => handle_macro(benched, &mut expr_block.block.stmts)?,
            Expr::If(if_expr) => handle_if(benched, if_expr)?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

fn handle_macro(benched: &mut Vec<String>, stmts: &mut Vec<Stmt>) -> Result<(), TokenStream> {
    let mut stmt_idx = 0;
    while stmt_idx < stmts.len() {
        if let Stmt::Expr(expr, _) = &mut stmts[stmt_idx] {
            match expr {
                Expr::Block(expr_block) => handle_macro(benched, &mut expr_block.block.stmts)?,
                Expr::ForLoop(expr_for_loop) => {
                    handle_macro(benched, &mut expr_for_loop.body.stmts)?
                }
                Expr::If(expr_if) => handle_if(benched, expr_if)?,
                Expr::Loop(expr_loop) => handle_macro(benched, &mut expr_loop.body.stmts)?,
                Expr::Match(expr_match) => {
                    for arm in &mut expr_match.arms {
                        if let Expr::Block(block) = &mut *arm.body {
                            handle_macro(benched, &mut block.block.stmts)?
                        }
                    }
                }
                Expr::While(expr_while) => handle_macro(benched, &mut expr_while.body.stmts)?,
                Expr::Async(expr_async) => handle_macro(benched, &mut expr_async.block.stmts)?,
                Expr::Unsafe(expr_unsafe) => handle_macro(benched, &mut expr_unsafe.block.stmts)?,
                _ => {}
            }
        }
        let attrs = match &stmts[stmt_idx] {
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
                                return Err(Error::new(list.span(), "Invalid usage of `bench` macro. Only one argument is supported.")
                                    .to_compile_error()
                                    .into());
                            }
                            let name = list.first().unwrap().clone().value();
                            let var_ident = format_ident!("_bench_{}_time", name);
                            let instant_ident = format_ident!("_bench_{}_instant", name);
                            if !benched.contains(&name) {
                                benched.push(name);
                            }
                            bench_macro_idx = Some(i);
                            stmts.insert(
                                stmt_idx,
                                parse_quote! {let #instant_ident = std::time::Instant::now();},
                            );
                            stmts.insert(
                                stmt_idx + 2,
                                parse_quote! {#var_ident +=  #instant_ident.elapsed().as_micros();},
                            );
                            stmt_idx += 2;
                        }
                        Err(err) => return Err(err.to_compile_error().into()),
                    }
                } else {
                    return Err(Error::new(attr.span(), "Invalid usage of `bench` macro")
                        .to_compile_error()
                        .into());
                }
            }
        }
        if let Some(i) = bench_macro_idx {
            let attrs = match &mut stmts[stmt_idx - 1] {
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
    Ok(())
}

#[proc_macro_attribute]
pub fn bench(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(item as ItemFn);
    let mut benched: Vec<String> = vec![];

    if let Err(err) = handle_macro(&mut benched, &mut item.block.stmts) {
        return err;
    };

    for name in benched {
        let var_ident = format_ident!("_bench_{}_time", name);
        item.block
            .stmts
            .insert(0, parse_quote! {let mut #var_ident: u128 = 0;});
        item.block.stmts.push(
            parse_quote! {log::debug!("Benchmark for {}: {} microseconds", #name, #var_ident);},
        );
    }

    TokenStream::from(item.to_token_stream())
}
