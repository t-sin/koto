pub mod io;
pub mod types;

pub use io::{read, print};

use types::Cons;

pub fn to_vec(list: &Cons) -> Vec<&Cons> {
    match list {
        Cons::Nil => Vec::new(),
        Cons::Cons(elem, rest) => {
            let mut v: Vec<&Cons> = Vec::new();
            v.push(elem);
            v.append(&mut to_vec(rest));
            v
        },
        _ => panic!("it's not proper list: {:?}", list),
    }
}
