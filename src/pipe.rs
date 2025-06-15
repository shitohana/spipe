use std::collections::VecDeque;

use syn::punctuated::Punctuated;
use syn::parse::*;
use syn::*;
use crate::utils::*;

macro_rules! pipe_const {
    (and_then) => { Token![&] };
    (clone) => { Token![+] };
    (map) => { Token![@] };
    (try) => { Token![?] };
    (unwrap) => { Token![*] };
    (apply) => { Token![#] };
    (apply_mut) => { Token![$] };
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub enum PipeType {
    // =>
    Basic,
    // =>&
    AndThen,
    // =>+
    Clone,
    // =>@
    Map,
    // =>?
    Try,
    // =>*
    Unwrap,
    // =>#
    Apply,
    // =>$
    ApplyMut,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PipeOp {
    NoOp,
    FnCall(ExprCall),
    MethodCall(ExprCall),
    Closure(ExprClosure),
    TypeFrom(ExprPath),
    TypeTryFrom(ExprPath),
    TypeAs(Type),
}

impl Parse for PipeOp {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![.]) && input.peek2(Token![.]) && input.peek3(Token![.]) {
            input.parse::<Token![.]>()?;
            input.parse::<Token![.]>()?;
            input.parse::<Token![.]>()?;
            return Ok(PipeOp::NoOp);
        }

        let lookahead = input.lookahead1();

        // Method call
        if lookahead.peek(Token![.]) {
            let _: Token![.] = input.parse()?;
            Ok(Self::MethodCall(parse_reduced_fn(input)?))
        }
        // Type cast
        else if lookahead.peek(syn::token::Paren) {
            let inside;
            parenthesized!(inside in input);
            let inside_stream = ParseStream::from(&inside);
            let inside_lookahead = inside.lookahead1();
            // As cast
            if inside_lookahead.peek(Token![as]) {
                let _: Token![as] = inside_stream.parse()?;
                let ty: Type = inside_stream.parse()?;
                Ok(Self::TypeAs(ty))
            }
            // From cast
            else if inside_lookahead.peek(syn::Ident) {
                let ty: ExprPath = inside_stream.parse()?;

                if inside_stream.peek(Token![?]) {
                    let _: Token![?] = inside_stream.parse()?;
                    Ok(Self::TypeTryFrom(ty))
                }
                else {
                    Ok(Self::TypeFrom(ty))
                }
            }
            else {
                Err(inside_lookahead.error())
            }
        }
        else if lookahead.peek(syn::Ident) {
            let expr_call = parse_reduced_fn(input)?;
            Ok(Self::FnCall(expr_call))
        }
        else if lookahead.peek(Token![|]) {
            let closure: ExprClosure = input.parse()?;
            Ok(Self::Closure(closure))
        }
        else {
            return Err(lookahead.error());
        }
    }
}

#[derive(Debug, Clone)]
struct PipeOpPair {
    pipe_type: PipeType,
    operation: PipeOp,
}

impl Parse for PipeOpPair {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();

        let ty = if lookahead.peek(pipe_const!(and_then)) {
            input.parse::<pipe_const!(and_then)>()?;
            PipeType::AndThen
        }
        else if lookahead.peek(pipe_const!(try)) {
            input.parse::<pipe_const!(try)>()?;
            PipeType::Try
        }
        else if lookahead.peek(pipe_const!(clone)) {
            input.parse::<pipe_const!(clone)>()?;
            PipeType::Clone
        }
        else if lookahead.peek(pipe_const!(unwrap)) {
            input.parse::<pipe_const!(unwrap)>()?;
            PipeType::Unwrap
        }
        else if lookahead.peek(pipe_const!(map)) {
            input.parse::<pipe_const!(map)>()?;
            PipeType::Map
        }
        else if lookahead.peek(pipe_const!(apply)) {
            input.parse::<pipe_const!(apply)>()?;
            PipeType::Apply
        }
        else if lookahead.peek(pipe_const!(apply_mut)) {
            input.parse::<pipe_const!(apply_mut)>()?;
            PipeType::ApplyMut
        }
        else {
            PipeType::Basic
        };

        let op: PipeOp = input.parse()?;
        Ok(PipeOpPair {
            pipe_type: ty,
            operation: op
        })
    }
}

pub struct MacroInput {
    initial:    Expr,
    pipe_pairs: VecDeque<PipeOpPair>,
}

impl MacroInput {
    pub fn run(mut self) -> Result<Expr> {
        let mut res = self.initial;
        let mut closure_count: usize = 0;
        while let Some(op) = self.pipe_pairs.pop_front() {
            let pipe_applied_fn = apply_pipe(
                op.pipe_type, res, &mut closure_count
            );
            res = pipe_applied_fn(op.operation)?
        }
        Ok(res)
    }
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let initial: Expr = input.parse()?;

        let parsed = if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            Punctuated::<PipeOpPair, Token![=>]>::parse_separated_nonempty(input)?
        }
        else {
            Default::default()
        };


        Ok(Self {
            initial,
            pipe_pairs: parsed.into_iter().collect(),
        })
    }
}
