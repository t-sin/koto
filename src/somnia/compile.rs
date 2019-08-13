use super::super::sexp::Cons;
use super::vm::{Op, Reg};

pub fn compile(sexp: Vec<Box<Cons>>) -> Vec<Box<Op>> {
    let mut program = Vec::new();
    for s in sexp {
        match *s {
            Cons::Cons(car, cdr) => (),
            Cons::Symbol(name) => (),
            Cons::Number(n) => {
                program.push(Box::new(Op::LOADC(n as u32, Reg::R1)));
                program.push(Box::new(Op::OUT(Reg::R1)));
            },
            Cons::Nil => program.push(Box::new(Op::NOP)),
        }
    }
    program
}
