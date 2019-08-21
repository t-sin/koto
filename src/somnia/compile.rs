use super::super::sexp::{Cons, to_vec};
use super::vm::{Op, Reg};

fn compile_1(sexp: Box<Cons>, prog: &mut Vec<Box<Op>>, target: Reg, toplevel: bool) {
    match *sexp {
        Cons::Cons(car, cdr) => {
            if let Cons::Symbol(name) = *car {
                let args = to_vec(&*cdr);
                match &name[..] {
                    "+" => {
                        if args.len() == 2 {
                            compile_1(args[0].clone(), prog, Reg::R1, false);
                            compile_1(args[1].clone(), prog, Reg::R2, false);
                            prog.push(Box::new(Op::ADD(Reg::R1, Reg::R2, target)));
                        } else {
                            panic!("wrong number of args for '+': {:?}", cdr);
                        }
                    },
                    _ => panic!("unknown operator: {:?}", name),
                }
            } else {
                panic!("invalid form: {:?} {:?}", car, cdr);
            }

            if toplevel == true {
                prog.push(Box::new(Op::OUT(target)));
            }
        },
        Cons::Symbol(name) => (),
        Cons::Number(n) => {
            prog.push(Box::new(Op::LOADC(n as u32, target)));

            if toplevel == true {
                prog.push(Box::new(Op::OUT(target)));
            }
        },
        Cons::Nil => prog.push(Box::new(Op::NOP)),
    }
}

pub fn compile(sexp: Vec<Box<Cons>>) -> Vec<Box<Op>> {
    let mut program = Vec::new();
    for s in sexp {
        compile_1(s, &mut program, Reg::R3, true);
    }
    program
}
