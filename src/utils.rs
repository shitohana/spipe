use parse::ParseStream;
use proc_macro2::Ident;
use punctuated::Punctuated;
use spanned::Spanned;
use syn::*;
use token::Comma;

use crate::pipe::{PipeOp, PipeType};

pub fn call_expr<I: IntoIterator<Item=Expr>, C: Into<Expr>>(call: C, args: I) -> Expr {
    let fn_call = ExprCall {
        attrs: vec![],
        func: Box::new(call.into()),
        paren_token: Default::default(),
        args: Punctuated::from_iter(args),
    };
    fn_call.into()
}

pub fn call_method_expr<I: IntoIterator<Item=Expr>>(receiver: Expr, fun: Ident, args: I) -> Expr {
    ExprMethodCall {
        attrs:       vec![],
        receiver:    Box::new(receiver),
        method:      fun,
        turbofish:   None,
        args:        Punctuated::from_iter(args),
        paren_token: Default::default(),
        dot_token:   Default::default(),
    }.into()
}

pub fn create_ident(value: &str) -> Ident {
    Ident::new(value, proc_macro2::Span::call_site())
}

pub fn path_to_expr(path: Path) -> Expr {
    Expr::Path(ExprPath {
        attrs: vec![],
        qself: None,
        path: path.into(),
    })
}

pub fn add_to_path(path: &mut Path, ident: &str) {
    path.segments.push(PathSegment::from(Ident::new(
        ident,
        proc_macro2::Span::call_site(),
    )));
}

pub fn replace_empty_paren_closure(v: &Expr) -> bool {
    match v {
        Expr::Tuple(e) => e.elems.is_empty(),
        _ => false,
    }
}

pub fn substitute_args(
    args: &mut Punctuated<Expr, Comma>,
    sub: Expr,
    which: fn(&Expr) -> bool,
) {
    if let Some(idx) = args.iter().position(which) {
        let _ = std::mem::replace(args.get_mut(idx).unwrap(), sub);
    } else {
        args.insert(0, sub)
    }
}

pub fn try_get_call_ident(expr: &ExprCall) -> Result<Ident> {
    match expr.func.as_ref() {
        Expr::Path(path) => {
            path
                .path
                .get_ident()
                .cloned()
                .ok_or(Error::new(expr.func.span(), "Expected ident"))
        },
        _ => Err(Error::new(expr.func.span(), "not a function")),
    }

}

pub fn parse_reduced_fn(input: ParseStream) -> Result<ExprCall> {
    let path: Path = input.parse()?;
    let args = if input.peek(syn::token::Paren) {
        let args;
        parenthesized!(args in input);
        Punctuated::<Expr, Token![,]>::parse_terminated(&args)?
    }
    else {
        Default::default()
    };

    let expr_call = ExprCall {
        attrs: vec![],
        func: Box::new(Expr::Path(ExprPath {
            attrs: vec![],
            qself: None,
            path,
        })),
        paren_token: Default::default(),
        args,
    };
    Ok(expr_call)
}


pub fn get_apply_block(
    op: PipeOp,
    expr: Expr,
    mutable: bool,
    closure_count: &mut usize
) -> Result<Expr> {
    let var_name = format!("__var_{}", *closure_count);
    let var_ident = Ident::new(var_name.as_str(), proc_macro2::Span::call_site());

    let closure_var_expr: Expr = ExprPath {
        attrs: vec![],
        qself: None,
        path:  Path {
            leading_colon: None,
            segments:      Punctuated::from_iter([PathSegment::from(
                var_ident.clone(),
            )]),
        },
    }
    .into();
    let expr_let = Local {
        attrs:      vec![],
        let_token:  Default::default(),
        pat:        PatIdent {
            attrs:      vec![],
            by_ref:     None,
            mutability: if mutable {
                Some(token::Mut::default())
            }
            else {
                None
            },
            ident:      var_ident,
            subpat:     None,
        }
        .into(),
        init:       Some(LocalInit {
            eq_token: Default::default(),
            expr:     Box::new(expr),
            diverge:  None,
        }),
        semi_token: Default::default(),
    };
    let var_reference = ExprReference {
        attrs: vec![],
        and_token: Default::default(),
        mutability: if mutable {
            Some(token::Mut::default())
        } else { None },
        expr: Box::new(closure_var_expr.clone()),
    };
    let applied = apply_op(op, var_reference.into())?;

    Ok(ExprBlock {
        attrs: vec![],
        label: None,
        block: Block {
            brace_token: Default::default(),
            stmts:       vec![
                Stmt::Local(expr_let),
                Stmt::Expr(applied, Some(token::Semi::default())),
                Stmt::Expr(closure_var_expr, None),
            ],
        },
    }
    .into())
}


pub fn get_fn_closure_call(
    pipe: PipeOp,
    call_on: Expr,
    method_name: &str,
) -> Result<Expr> {
    let closure_var = Path {
        leading_colon: None,
        segments:      Punctuated::from_iter([PathSegment::from(create_ident("__map_var"))]),
    };
    let closure_var_expr = Expr::Path(ExprPath {
        attrs: vec![],
        qself: None,
        path:  closure_var.clone(),
    });
    let applied = apply_op(pipe, closure_var_expr)?;
    let closure_pat = Pat::Path(PatPath {
        attrs: vec![],
        qself: None,
        path:  closure_var,
    });
    let closure = ExprClosure {
        attrs:      vec![],
        lifetimes:  None,
        constness:  None,
        movability: None,
        asyncness:  None,
        capture:    None,
        or1_token:  Default::default(),
        inputs:     Punctuated::from_iter([closure_pat]),
        or2_token:  Default::default(),
        output:     ReturnType::Default,
        body:       Box::new(applied),
    };
    Ok(call_method_expr(
        call_on,
        create_ident(method_name),
        [closure.into()]
    ))
}

pub fn apply_op(pipe: PipeOp, expr: Expr) -> Result<Expr> {
    use PipeOp::*;
    match pipe {
        NoOp => Ok(expr),
        FnCall(mut call) => {
            substitute_args(
                &mut call.args,
                expr,
                replace_empty_paren_closure
            );
            Ok(call.into())
        },
        MethodCall(call) => {
            let ident = try_get_call_ident(&call)?;
            Ok(call_method_expr(expr, ident, call.args))
        },
        Closure(call) => {
            Ok(call_expr(call, [expr]))
        },
        TypeFrom(mut ty) => {
            add_to_path(&mut ty.path, "from");
            Ok(call_expr(path_to_expr(ty.path), vec![expr]))
        },
        TypeTryFrom(mut ty) => {
            add_to_path(&mut ty.path, "try_from");
            Ok(call_expr(path_to_expr(ty.path), vec![expr]))
        },
        TypeAs(ty) => {
            let as_call = ExprCast {
                attrs:    vec![],
                expr:     Box::new(expr),
                as_token: Default::default(),
                ty:       Box::new(ty),
            };
            Ok(Expr::Cast(as_call))
        },
    }
}

pub fn apply_pipe(pipe: PipeType, expr: Expr, closure_count: &mut usize) -> Box<dyn FnOnce(PipeOp) -> Result<Expr> + '_> {
    use PipeType::*;
    match pipe {
        Basic => {
            Box::new(|x| apply_op(x, expr))
        },
        AndThen => {
            Box::new(|x| get_fn_closure_call(x, expr, "and_then"))
        },
        Clone => {
            let method_expr = call_method_expr(
                expr, create_ident("clone"), []
            );
            Box::new(move |x| apply_op(x, method_expr))
        },
        Map => {
            Box::new(|x| get_fn_closure_call(x, expr, "map"))
        },
        Try => {
            let try_expr = ExprTry {
                attrs:          vec![],
                expr:           Box::new(expr),
                question_token: Default::default(),
            };
            Box::new(move |x| apply_op(x, try_expr.into()))
        },
        Unwrap => {
            let method_expr = call_method_expr(
                expr, create_ident("unwrap"), []
            );
            Box::new(move |x| apply_op(x, method_expr))
        },
        Apply => {
            *closure_count += 1;
            Box::new(|x| get_apply_block(x, expr, false, closure_count))
        },
        ApplyMut => {
            *closure_count += 1;
            Box::new(|x| get_apply_block(x, expr, true, closure_count))
        }
    }
}