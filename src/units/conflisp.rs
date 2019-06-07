use std::iter::Peekable;
use std::str::Chars;

use super::unit::Stateful;
use super::unit::Signal;
use super::unit::Unit;

use super::core::Offset;
use super::core::Gain;

use super::oscillator::Sine;


#[derive(Debug, PartialEq)]
pub enum Cons {
    Cons(Box<Cons>, Box<Cons>),
    Symbol(String),
    Number(f64),
    Nil,
}

//// reader

fn skip_whitespaces(chars: &mut Peekable<Chars>) {
    loop {
        let ch = chars.peek();
        match ch {
            Some(c) => {
                if c.is_whitespace() {
                    chars.next();
                } else {
                    break;
                }
            },
            _ => break,
        }
    }
}

fn read_symbol(chars: &mut Peekable<Chars>) -> Cons {
    let mut name = String::new();
    loop {
        let ch = chars.peek();
        match ch {
            Some(c) => {
                if *c == ')' || c.is_whitespace() {
                    break;
                } else {
                    name.push(chars.next().unwrap());
                }
            },
            _ => break,
        }
    }
    Cons::Symbol(name)
}

fn read_number(chars: &mut Peekable<Chars>) -> Cons {
    let mut num = String::new();
    loop {
        let ch = chars.peek();
        match ch {
            Some(c) => {
                if *c == '.' || c.is_digit(10) {
                    num.push(chars.next().unwrap());
                } else {
                    break;
                }
            },
            _ => break,
        }
    }
    match num.parse::<f64>() {
        Ok(n) => Cons::Number(n),
        Err(e) => panic!("cannot parse '{:?}' as a number: {:?}", num, e),
    }
}

fn read_list_elem(chars: &mut Peekable<Chars>) -> Cons {
    skip_whitespaces(chars);
    let ch = chars.peek();
    match ch {
        Some(')') => {
            chars.next();
            Cons::Nil
        },
        _ => {
            Cons::Cons(Box::new(read_exp(chars)), Box::new(read_list_elem(chars)))
        },
    }
}

fn read_list(chars: &mut Peekable<Chars>) -> Cons {
    chars.next();
    read_list_elem(chars)
}

fn read_exp(chars: &mut Peekable<Chars>) -> Cons {
    skip_whitespaces(chars);
    let ch = chars.peek();
    match ch {
        None => Cons::Nil,
        Some(')') => panic!("unexpected ')'"),
        Some('(') => read_list(chars),
        Some(c) => {
            if c.is_digit(10) {
                read_number(chars)
            } else {
                read_symbol(chars)
            }
        },
    }
}

pub fn read(s: String) -> Vec<Cons> {
    let mut chars = s.chars().peekable();
    let mut sexp_vec = Vec::new();
    loop {
        let sexp = read_exp(&mut chars);
        if sexp == Cons::Nil {
            break;
        } else {
            sexp_vec.push(sexp);
        }
    }
    sexp_vec
}

//// printer

fn print_list(car: &Cons, cdr: &Cons) -> String {
    let mut s = String::new();
    s.push_str(print(car).as_str());
    match cdr {
        Cons::Cons(car2, cdr2) => {
            s.push(' ');
            s.push_str(print_list(car2, cdr2).as_str())
        },
        Cons::Nil => (),
        exp => panic!("ill formed S-expression: {:?}", exp),
    }
    s
}

pub fn print(exp: &Cons) -> String {
    let mut s = String::new();
    match exp {
        Cons::Nil => s.push_str("nil"),
        Cons::Number(n) => s.push_str(n.to_string().as_str()),
        Cons::Symbol(n) => s.push_str(n.to_string().as_str()),
        Cons::Cons(car, cdr) => {
            s.push('(');
            s.push_str(print_list(car, cdr).as_str());
            s.push(')');
        },
     }
    s
}

//// unit graph constructor (or eval?)
fn to_vec(list: &Cons) -> Vec<&Cons> {
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

fn construct(name: &str, args: Vec<&Cons>) -> Unit {
    match &name[..] {
        "sine" => {
            if args.len() == 2 {
                Unit::Unit(Box::new(Sine {
                    init_ph: eval_one(args[0]),
                    ph: 0.0,
                    freq: eval_one(args[1]),
                }))
            } else {
                panic!("wrong params");
            }
        },
        "offset" => {
            if args.len() == 2 {
                Unit::Unit(Box::new(Offset {
                    v: match args[0] {
                        Cons::Number(n) => *n,
                        exp => panic!("{:?} is not a number", print(exp)),
                    },
                    src: eval_one(args[1]),
                }))
            } else {
                panic!("wrong params");
            }
        },
        "gain" => {
            if args.len() == 2 {
                Unit::Unit(Box::new(Gain {
                    v: match args[0] {
                        Cons::Number(n) => *n,
                        exp => panic!("{:?} is not a number", print(exp)),
                    },
                    src: eval_one(args[1]),
                }))
            } else {
                panic!("wrong params");
            }
        },
        _ => {
            println!("{:?} is unknown or not implemented.", name);
            Unit::Value(0.0)
        }
    }
}

fn eval_list(name: &Cons, args: &Cons) -> Unit {
    match name {
        Cons::Symbol(n) => construct(&n[..], to_vec(&args)),
        _ => panic!("ill formed form"),
    }
}

pub fn eval_one(sexp: &Cons) -> Unit {
    match sexp {
        Cons::Cons(car, cdr) => eval_list(car, cdr),
        Cons::Symbol(name) => {
            println!("name: {:?}", name);
            Unit::Value(0.0)
        },
        Cons::Number(num) => Unit::Value(*num),
        Cons::Nil => panic!("what should I do?"),
    }
}

// TODO: unit graph serializer
