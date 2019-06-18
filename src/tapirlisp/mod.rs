pub mod types;
pub mod rp;
pub mod make;
pub mod eval;
//pub mod dump;

use std::collections::VecDeque;

pub use rp::{read, print};
pub use eval::eval;
//pub use dump::dump;

use types::{Cons, Value, Env, EvalError};

pub fn to_vec(list: &Cons) -> Vec<Box<Cons>> {
    match list {
        Cons::Nil => Vec::new(),
        Cons::Cons(elem, rest) => {
            let mut v: Vec<Box<Cons>> = Vec::new();
            v.push(Box::new((**elem).clone()));
            v.append(&mut to_vec(rest));
            v
        },
        _ => panic!("it's not proper list: {:?}", list),
    }
}

pub fn eval_all(sexp_vec: Vec<Box<Cons>>, env: &mut Env) -> Result<Value, EvalError> {
    let mut q = VecDeque::new();
    for sexp in sexp_vec.iter() {
        match eval(sexp, env) {
            Ok(v) => q.push_back(v),
            Err(err) => return Err(err),
        }
    }
    match q.len() {
        0 => Ok(Value::Nil),
        _ => Ok(q.pop_back().unwrap()),
    }
}
