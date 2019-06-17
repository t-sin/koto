use std::error::Error;
use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

use super::types::Cons;

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

fn read_symbol(chars: &mut Peekable<Chars>) -> Result<Cons, ReadError> {
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
    Ok(Cons::Symbol(name))
}

fn read_number(chars: &mut Peekable<Chars>) -> Result<Cons, ReadError> {
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
        Ok(n) => Ok(Cons::Number(n)),
        Err(e) => Err(ReadError::InvalidNumber(num)),
    }
}

fn read_list_elem(chars: &mut Peekable<Chars>) -> Result<Cons, ReadError> {
    skip_whitespaces(chars);
    let ch = chars.peek();
    match ch {
        Some(')') => {
            chars.next();
            Ok(Cons::Nil)
        },
        _ => {
            match read_exp(chars) {
                Ok(car) => match read_list_elem(chars) {
                    Ok(cdr) =>  Ok(Cons::Cons(Box::new(car), Box::new(cdr))),
                    e => e,
                },
                e => e,
            }
        },
    }
}

fn read_list(chars: &mut Peekable<Chars>) -> Result<Cons, ReadError> {
    chars.next();
    read_list_elem(chars)
}

fn read_exp(chars: &mut Peekable<Chars>) -> Result<Cons, ReadError> {
    skip_whitespaces(chars);
    let ch = chars.peek();
    match ch {
        None => Ok(Cons::Nil),
        Some(')') => Err(ReadError::UnexpectedCloseParen),
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

pub fn read(s: String) -> Result<Vec<Box<Cons>>, ReadError> {
    let mut chars = s.chars().peekable();
    let mut sexp_vec = Vec::new();
    loop {
        match read_exp(&mut chars) {
            Ok(Cons::Nil) => break,
            Ok(c) => sexp_vec.push(Box::new(c)),
            Err(e) => return Err(e),
        }
    }
    Ok(sexp_vec)
}

fn print_list(car: &Cons, cdr: &Cons) -> String {
    let mut s = String::new();
    s.push_str(print(car).as_str());
    match cdr {
        Cons::Cons(car2, cdr2) => {
            s.push(' ');
            s.push_str(print_list(car2, cdr2).as_str())
        },
        Cons::Nil => (),
        Cons::Number(n) => {
            s.push_str(" . ");
            s.push_str(&n.to_string());
        },
        Cons::Symbol(n) => {
            s.push_str(" . ");
            s.push_str(n);
        },
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
