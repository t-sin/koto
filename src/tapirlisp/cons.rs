#[derive(Debug, PartialEq)]
pub enum Cons {
    Cons(Box<Cons>, Box<Cons>),
    Symbol(String),
    Number(f64),
    Nil,
}
