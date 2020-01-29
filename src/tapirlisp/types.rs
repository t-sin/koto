use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use super::super::mtime::Time;
use super::super::sexp::Cons;
use super::super::ugen::core::Aug;

pub type Name = String;

#[derive(Clone)]
pub enum Value {
    Unit(Aug),
    Nil,
}

#[derive(Clone)]
pub struct Env {
    pub time: Time,
    pub binding: HashMap<Name, Box<Value>>,
}

impl Env {
    pub fn init(time: Time) -> Env {
        Env {
            time: time,
            binding: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum EvalError {
    FnWrongParams(String, Vec<Box<Cons>>),
    FnUnknown(String),
    FnMalformedName(Box<Cons>),
    EvWrongParams(String),
    EvUnknown(String),
    EvMalformedEvent(String),
    UnboundVariable(String),
    AlreadyBound(String),
    NotANumber(String),
    NotASymbol(Box<Cons>),
    NotAug,
    NotAPattern,
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EvalError::FnWrongParams(name, args) => {
                write!(f, "Wrong params for '{:?}' with args '{:?}'", name, args)
            }
            EvalError::FnUnknown(name) => write!(f, "{:?} is unknown or not implemented.", name),
            EvalError::FnMalformedName(cons) => write!(f, "{:?} is not a symbol.", cons),
            EvalError::EvWrongParams(args) => {
                write!(f, "Wrong params for 'pat' with args '{:?}'", args)
            }
            EvalError::EvUnknown(name) => {
                write!(f, "{:?} is unknown or not implemented event.", name)
            }
            EvalError::EvMalformedEvent(s) => write!(f, "{:?} is not an event.", s),
            EvalError::UnboundVariable(name) => write!(f, "Unbound variable: '{:?}'", name),
            EvalError::AlreadyBound(name) => write!(f, "'{:?}' is already bound", name),
            EvalError::NotANumber(s) => write!(f, "{:?} is not a number", s),
            EvalError::NotASymbol(cons) => write!(f, "{:?} is not a symbol.", cons),
            EvalError::NotAug => write!(f, "((serialized unit here)) is not an unit"),
            EvalError::NotAPattern => write!(f, "it's not a pattern"),
        }
    }
}

impl Error for EvalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            EvalError::FnWrongParams(_name, _args) => None,
            EvalError::FnUnknown(_) => None,
            EvalError::FnMalformedName(_) => None,
            EvalError::EvWrongParams(_) => None,
            EvalError::EvUnknown(_) => None,
            EvalError::EvMalformedEvent(_) => None,
            EvalError::UnboundVariable(_) => None,
            EvalError::AlreadyBound(_) => None,
            EvalError::NotANumber(_) => None,
            EvalError::NotASymbol(_) => None,
            EvalError::NotAug => None,
            EvalError::NotAPattern => None,
        }
    }
}
