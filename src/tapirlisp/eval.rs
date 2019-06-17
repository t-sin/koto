use std::sync::{Arc, Mutex};

use super::super::tapirlisp::types::{Cons, EvalError};
use super::super::tapirlisp::{to_vec, make_unit};

use super::super::units::unit::{AUnit, UType, UnitGraph};

fn eval_call(name: &Cons, args: &Cons) -> Result<AUnit, EvalError> {
    match name {
        Cons::Symbol(n) => make_unit(&n[..], to_vec(&args)),
        c => Err(EvalError::FnMalformedName(Box::new(c.clone()))),
    }
}

pub fn eval(sexp: &Cons) -> Result<AUnit, EvalError> {
    match sexp {
        Cons::Cons(car, cdr) => eval_call(car, cdr),
        Cons::Symbol(name) => Err(EvalError::TodoSearchValueFromBinding),
        Cons::Number(num) => Ok(Arc::new(Mutex::new(UnitGraph::Value(*num)))),
        Cons::Nil => Err(EvalError::Nil),
    }
}
