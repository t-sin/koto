pub mod rp;
pub mod types;

pub use rp::{read, print};

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
