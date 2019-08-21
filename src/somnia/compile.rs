use super::super::sexp::{Cons, to_vec};
use super::vm::{Op, Reg};

fn compile_1(sexp: &Cons, prog: &mut Vec<Box<Op>>) {
    match sexp {
        Cons::Cons(car, cdr) => {
            // なんかここで**carしろと言われたり*carにしろといわれたりする。
            // そもそもCons::Cons(<Box<Conx>, Box<Cons>)という型がよくない？
            if let Cons::Symbol(name) = *car {
                let args = to_vec(&*cdr);
                match &name[..] {
                    "+" => {
                        if args.len() == 2 {
                            match *args[0] {
                                Cons::Number(n) => prog.push(Box::new(Op::LOADC(n as u32, Reg::R1))),
                                Cons::Cons(_, _) => compile_1(&args[0], prog),
                                _ => (),
                            }
                            match *args[1] {
                                Cons::Number(n) => prog.push(Box::new(Op::LOADC(n as u32, Reg::R2))),
                                _ => (),
                            }
                            prog.push(Box::new(Op::ADD(Reg::R1, Reg::R2, Reg::R3)));
                        } else {
                            panic!("wrong number of args for '+': {:?}", cdr);
                        }
                    },
                    _ => panic!("unknown operator: {:?}", name),
                }
            } else {
                panic!("invalid form: {:?} {:?}", car, cdr);
            }
        },
        Cons::Symbol(name) => (),
        Cons::Number(n) => {
            prog.push(Box::new(Op::LOADC(*n as u32, Reg::R1)));
            prog.push(Box::new(Op::OUT(Reg::R1)));
        },
        Cons::Nil => prog.push(Box::new(Op::NOP)),
    }
}

pub fn compile(sexp: Vec<Box<Cons>>) -> Vec<Box<Op>> {
    let mut program = Vec::new();
    for s in sexp {
        compile_1(&*s, &mut program);
    }
    program
}
