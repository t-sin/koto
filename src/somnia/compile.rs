use super::super::sexp::{Cons, to_vec};
use super::vm::{Op, Reg};

#[derive(Debug)]
struct CompileState {
    program: Vec<Box<Op>>,
    used_regs: [bool; 4],
}

fn find_free_reg(cs: &CompileState) -> Option<Reg> {
    match cs.used_regs.iter().position(|e| *e == false) {
        Some(idx) => {
            match idx {
                0 => Some(Reg::R1),
                1 => Some(Reg::R2),
                2 => Some(Reg::R3),
                3 => Some(Reg::R4),
                _ => None,
            }
        },
        None => None,
    }
}

fn set_reg(reg: Reg, flag: bool, cs: &mut CompileState) {
    match reg {
        Reg::R1 => cs.used_regs[0] = flag,
        Reg::R2 => cs.used_regs[1] = flag,
        Reg::R3 => cs.used_regs[2] = flag,
        Reg::R4 => cs.used_regs[3] = flag,
        _ => (),
    }
}

fn compile_1(sexp: Box<Cons>, cs: &mut CompileState, toplevel: bool) -> Option<Reg> {
    if let Some(target) = find_free_reg(&cs) {
        match *sexp {
            Cons::Cons(car, cdr) => {
                let mut reg: Reg;
                if let Cons::Symbol(name) = *car {
                    let args = to_vec(&*cdr);
                    match &name[..] {
                        "+" => {
                            if args.len() == 2 {
                                let reg1 = compile_1(args[0].clone(), cs, false).unwrap();
                                let reg2 = compile_1(args[1].clone(), cs, false).unwrap();
                                cs.program.push(Box::new(Op::ADD(reg1, reg2, target)));
                                set_reg(target, true, cs);
                                set_reg(reg1, false, cs);
                                set_reg(reg2, false, cs);
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
                    cs.program.push(Box::new(Op::OUT(target)));
                    None
                } else {
                    Some(target)
                }
            },
            Cons::Symbol(name) => None,
            Cons::Number(n) => {
                cs.program.push(Box::new(Op::LOADC(n as u32, target)));
                set_reg(target, true, cs);

                if toplevel == true {
                    cs.program.push(Box::new(Op::OUT(target)));
                    None
                } else {
                    Some(target)
                }
            },
            Cons::Nil => {
                cs.program.push(Box::new(Op::NOP));
                None
            },
        }
    } else {
        println!("state: {:?}", cs);
        panic!("there's no free register!!");
    }
}

pub fn compile(sexp: Vec<Box<Cons>>) -> Vec<Box<Op>> {
    let mut cstate = CompileState {
        program: Vec::new(),
        used_regs: [false, false, false, false],
    };
    for s in sexp {
        compile_1(s, &mut cstate, true);
    }
    cstate.program
}
