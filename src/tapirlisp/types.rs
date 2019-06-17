use std::collections::HashMap;

use super::super::time::{Measure};

#[derive(Debug, PartialEq, Clone)]
pub enum Cons {
    Cons(Box<Cons>, Box<Cons>),
    Symbol(String),
    Number(f64),
    Nil,
}

pub type Name = String;
pub trait Value {}

pub struct Env {
    measure: Measure,
    binding: HashMap<String, Box<Value>>,
}
