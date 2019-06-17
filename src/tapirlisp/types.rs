use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use super::super::time::{Measure};
use super::super::event::{Event};
use super::super::units::unit::{AUnit};

#[derive(Debug, PartialEq, Clone)]
pub enum Cons {
    Cons(Box<Cons>, Box<Cons>),
    Symbol(String),
    Number(f64),
    Nil,
}

pub type Name = String;
pub enum Value {
    Pattern(Vec<Box<Event>>),
    Unit(AUnit),
}

pub struct Env {
    measure: Measure,
    binding: HashMap<String, Box<Value>>,
}

#[derive(Debug)]
pub enum ReadError {
    InvalidNumber(String),
    UnexpectedCloseParen,
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReadError::InvalidNumber(s) => write!(f, "Cannot parse '{}' as a number", s),
            ReadError::UnexpectedCloseParen => write!(f, "Unexpected ')'"),
        }
    }
}

impl Error for ReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ReadError::InvalidNumber(_s) => None,
            ReadError::UnexpectedCloseParen => None,
        }
    }
}

#[derive(Debug)]
pub enum EvalError {
    FnWrongParams(String, Vec<Box<Cons>>),
    FnUnknown(String),
    FnMalformedName(Box<Cons>),
    NotANumber(String),
    NotAUnit(Vec<Box<Event>>),
    TodoSearchValueFromBinding,
    Nil
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EvalError::FnWrongParams(name, args) => {
                write!(f, "Wrong params for '{:?}' with args '{:?}'", name, args)
            },
            EvalError::FnUnknown(name) => {
                write!(f, "{:?} is unknown or not implemented.", name)
            },
            EvalError::FnMalformedName(cons) => {
                write!(f, "{:?} is not a symbol.", cons)
            },
            EvalError::NotANumber(s) => {
                write!(f, "{:?} is not a number", s)
            },
            EvalError::NotAUnit(vec) => {
                write!(f, "{:?} is not an unit", vec)
            },
            EvalError::TodoSearchValueFromBinding => {
                write!(f, "TODO: searching from binding.")
            },
            EvalError::Nil => {
                write!(f, "nil.")
            },
        }
    }
}

impl Error for EvalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            EvalError::FnWrongParams(_name, _args) => None,
            EvalError::FnUnknown(_) => None,
            EvalError::FnMalformedName(_) => None,
            EvalError::TodoSearchValueFromBinding => None,
            EvalError::NotANumber(_) => None,
            EvalError::NotAUnit(v) => None,
            EvalError::Nil => None,
        }
    }
}
