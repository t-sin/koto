pub mod types;
pub mod rp;
pub mod make;
pub mod eval;

pub use rp::{read, print};
pub use eval::eval;

use types::Cons;

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
