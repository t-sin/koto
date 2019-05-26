#[derive(Debug)]
pub enum Cons {
    Cons(String, Box<Cons>),
    Nil,
}

pub fn read(s: String) -> Cons {
    Cons::Nil
}

pub fn print(c: Cons) {
    print!("nil");
}
