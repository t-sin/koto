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

fn compile_fn(name: &str, args: Vec<Box<Cons>>, ret: Reg, cs: &mut CompileState) {
    match &name[..] {
        "+" => {
            if args.len() == 2 {
                compile_1(args[0].clone(), cs, false);
                compile_1(args[1].clone(), cs, false);
                if let Some(reg1) = find_free_reg(cs) {
                    set_reg(reg1, true, cs);
                    if let Some(reg2) = find_free_reg(cs) {
                        set_reg(reg2, true, cs);
                        cs.program.push(Box::new(Op::POP(reg1)));
                        cs.program.push(Box::new(Op::POP(reg2)));
                        cs.program.push(Box::new(Op::ADD(reg1, reg2, ret)));
                        set_reg(reg1, false, cs);
                        set_reg(reg2, false, cs);
                        set_reg(ret, true, cs);
                    }
                }
            } else {
                panic!("wrong number of args for '+': {:?}", args);
            }
        },
        _ => panic!("unknown operator: {:?}", name),
    }
}

fn compile_1(sexp: Box<Cons>, cs: &mut CompileState, toplevel: bool) {
    if let Some(target) = find_free_reg(&cs) {
        match *sexp {
            Cons::Cons(car, cdr) => {
                let mut reg: Reg;
                if let Cons::Symbol(name) = *car {
                    let args = to_vec(&*cdr);
                    compile_fn(&name, args, target, cs);
                } else {
                    panic!("invalid form: {:?} {:?}", car, cdr);
                }
                if toplevel == true {
                    cs.program.push(Box::new(Op::OUT(target)));
                } else {
                    cs.program.push(Box::new(Op::PUSH(target)));
                }
            },
            Cons::Symbol(name) => (),
            Cons::Number(n) => {
                cs.program.push(Box::new(Op::LOADC(n as u32, target)));
                if toplevel == true {
                    cs.program.push(Box::new(Op::OUT(target)));
                } else {
                    cs.program.push(Box::new(Op::PUSH(target)));
                }
            },
            Cons::Nil => {
                cs.program.push(Box::new(Op::NOP));
            },
        }
        set_reg(target, false, cs);
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
